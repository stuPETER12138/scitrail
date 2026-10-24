//! 条目（说明书 §4.2 / P3）：CRUD、修改历史快照、检索

use crate::models::Entry;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tasks::{now_str, today_str};

pub const KINDS: [&str; 5] = ["experiment", "literature", "expense", "meeting", "note"];

#[derive(Clone, Serialize, Deserialize)]
pub struct RefItem {
    pub label: String,
    pub path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryInput {
    pub date: Option<String>,
    pub time: Option<String>,
    pub kind: String,
    pub title: String,
    pub content: Option<String>,
    pub project_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub fields: Option<Value>,
    pub refs: Option<Vec<RefItem>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub snapshot: Value,
    pub saved_at: String,
}

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

const ENTRY_COLUMNS: &str =
    "id, date, time, kind, title, content, project_id, tags, fields, refs, created_at, updated_at";

pub fn get(conn: &Connection, id: i64) -> Result<Entry, String> {
    conn.query_row(
        &format!("SELECT {ENTRY_COLUMNS} FROM entry WHERE id = ?1 AND deleted_at IS NULL"),
        params![id],
        Entry::from_row,
    )
    .map_err(|_| "条目不存在".to_string())
}

pub fn create(conn: &Connection, input: EntryInput) -> Result<i64, String> {
    if !KINDS.contains(&input.kind.as_str()) {
        return Err(format!("未知条目类型：{}", input.kind));
    }
    let title = input.title.trim();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    let tags = serde_json::to_string(&input.tags.unwrap_or_default()).map_err(|e| e.to_string())?;
    let fields = serde_json::to_string(&input.fields.clone().unwrap_or(Value::Null))
        .map_err(|e| e.to_string())?;
    let refs = serde_json::to_string(&input.refs.clone().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO entry(date, time, kind, title, content, project_id, tags, fields, refs)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            clean(input.date).unwrap_or_else(today_str),
            clean(input.time),
            input.kind,
            title,
            input.content.unwrap_or_default(),
            input.project_id,
            tags,
            fields,
            refs
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑；未提供的可选字段保留旧值，修改前自动把旧内容快照进 entry_history
pub fn update(conn: &Connection, id: i64, input: EntryInput) -> Result<(), String> {
    if !KINDS.contains(&input.kind.as_str()) {
        return Err(format!("未知条目类型：{}", input.kind));
    }
    let title = input.title.trim();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    let old = get(conn, id)?;
    let snapshot = serde_json::to_value(&old).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO entry_history(entry_id, snapshot) VALUES(?1, ?2)",
        params![id, snapshot.to_string()],
    )
    .map_err(|e| e.to_string())?;

    let tags = serde_json::to_string(&input.tags.unwrap_or(old.tags)).map_err(|e| e.to_string())?;
    let fields =
        serde_json::to_string(&input.fields.unwrap_or(old.fields)).map_err(|e| e.to_string())?;
    let refs = serde_json::to_string(&input.refs.unwrap_or(old.refs)).map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "UPDATE entry SET date=?2, time=?3, kind=?4, title=?5, content=?6, project_id=?7,
                 tags=?8, fields=?9, refs=?10, updated_at=?11
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                clean(input.date).unwrap_or(old.date),
                clean(input.time).or(old.time),
                input.kind,
                title,
                input.content.unwrap_or(old.content),
                input.project_id,
                tags,
                fields,
                refs,
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("条目不存在".into());
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE entry SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 列表/检索：q 走「FTS5 短语 + LIKE」双通道（中文靠 LIKE，英文数字两者都命中）
pub fn list(
    conn: &Connection,
    from: Option<String>,
    to: Option<String>,
    kind: Option<String>,
    project_id: Option<i64>,
    q: Option<String>,
) -> Result<Vec<Entry>, String> {
    let mut sql = format!("SELECT {ENTRY_COLUMNS} FROM entry WHERE deleted_at IS NULL");
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(f) = clean(from) {
        sql.push_str(" AND date >= ?");
        params_vec.push(Box::new(f));
    }
    if let Some(t) = clean(to) {
        sql.push_str(" AND date <= ?");
        params_vec.push(Box::new(t));
    }
    if let Some(k) = clean(kind) {
        sql.push_str(" AND kind = ?");
        params_vec.push(Box::new(k));
    }
    if let Some(p) = project_id {
        sql.push_str(" AND project_id = ?");
        params_vec.push(Box::new(p));
    }
    if let Some(query) = clean(q) {
        let like = format!("%{query}%");
        let phrase = format!("\"{}\"", query.replace('"', "\"\""));
        sql.push_str(
            " AND (title LIKE ? OR content LIKE ? \
               OR id IN (SELECT rowid FROM entry_fts WHERE entry_fts MATCH ?))",
        );
        params_vec.push(Box::new(like.clone()));
        params_vec.push(Box::new(like));
        params_vec.push(Box::new(phrase));
    }
    sql.push_str(" ORDER BY date DESC, (time IS NULL), time DESC, id DESC LIMIT 500");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter().map(|b| b.as_ref())),
            Entry::from_row,
        )
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn history(conn: &Connection, entry_id: i64) -> Result<Vec<HistoryItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT snapshot, saved_at FROM entry_history WHERE entry_id = ?1 ORDER BY id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![entry_id], |r| {
            let s: String = r.get(0)?;
            Ok(HistoryItem {
                snapshot: serde_json::from_str(&s).unwrap_or(Value::Null),
                saved_at: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// 今日条目数（今日页摘要用）
pub fn today_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM entry WHERE deleted_at IS NULL AND date = ?1",
        params![today_str()],
        |r| r.get(0),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    #[test]
    fn create_update_records_history() {
        let conn = test_conn();
        let id = create(
            &conn,
            EntryInput {
                date: Some("2026-10-24".into()),
                time: Some("09:12".into()),
                kind: "experiment".into(),
                title: "预烧结 1150℃".into(),
                content: None,
                project_id: None,
                tags: Some(vec!["催化".into()]),
                fields: Some(serde_json::json!({"params": "2h"})),
                refs: None,
            },
        )
        .unwrap();
        // 两次修改 → 两条历史
        for t in ["预烧结 1150℃ v2", "预烧结 1150℃ v3"] {
            update(
                &conn,
                id,
                EntryInput {
                    date: Some("2026-10-24".into()),
                    time: Some("10:00".into()),
                    kind: "experiment".into(),
                    title: t.into(),
                    content: Some("正文".into()),
                    project_id: None,
                    tags: None,
                    fields: None,
                    refs: None,
                },
            )
            .unwrap();
        }
        let hist = history(&conn, id).unwrap();
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].snapshot["title"], "预烧结 1150℃ v2");
        assert_eq!(hist[1].snapshot["title"], "预烧结 1150℃");

        let e = get(&conn, id).unwrap();
        assert_eq!(e.title, "预烧结 1150℃ v3");
        assert_eq!(e.tags, vec!["催化".to_string()]);
        assert_eq!(e.fields["params"], "2h"); // 未随更新清空
    }

    #[test]
    fn search_chinese_like_and_english_fts() {
        let conn = test_conn();
        create(
            &conn,
            EntryInput {
                date: Some("2026-10-24".into()),
                time: None,
                kind: "note".into(),
                title: "中文关键词测试".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();
        create(
            &conn,
            EntryInput {
                date: Some("2026-10-24".into()),
                time: None,
                kind: "note".into(),
                title: "sinter gradient analysis".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();
        let zh = list(&conn, None, None, None, None, Some("关键词".into())).unwrap();
        assert_eq!(zh.len(), 1);
        assert_eq!(zh[0].title, "中文关键词测试");
        let en = list(&conn, None, None, None, None, Some("sinter".into())).unwrap();
        assert_eq!(en.len(), 1);
        assert_eq!(en[0].title, "sinter gradient analysis");
    }

    #[test]
    fn soft_delete_hides_everywhere() {
        let conn = test_conn();
        let id = create(
            &conn,
            EntryInput {
                date: None,
                time: None,
                kind: "note".into(),
                title: "将被删除".into(),
                content: None,
                project_id: None,
                tags: None,
                fields: None,
                refs: None,
            },
        )
        .unwrap();
        delete(&conn, id).unwrap();
        assert!(get(&conn, id).is_err());
        assert!(
            list(&conn, None, None, None, None, Some("将被删除".into()))
                .unwrap()
                .is_empty()
        );
    }
}
