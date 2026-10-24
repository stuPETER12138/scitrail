//! 论文投稿流水线（说明书 §4.8）：八阶段流转、放榜日期同步生成 DDL

use crate::models::Paper;
use rusqlite::{Connection, params};

use super::tasks::now_str;

pub const STAGES: [&str; 8] = [
    "构思",
    "撰写中",
    "组内审",
    "投稿中",
    "审稿中",
    "返修",
    "录用",
    "见刊",
];

/// 有放榜提醒意义的阶段
fn reply_stage(stage: &str) -> bool {
    matches!(stage, "投稿中" | "审稿中" | "返修")
}

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperInput {
    pub title: String,
    pub venue: Option<String>,
    pub stage: Option<String>,
    pub submit_date: Option<String>,
    pub expect_reply: Option<String>,
    pub revise_deadline: Option<String>,
    pub project_id: Option<i64>,
    pub note: Option<String>,
}

const PAPER_COLUMNS: &str = "id, title, venue, stage, submit_date, expect_reply, revise_deadline, \
     project_id, ddl_id, note, created_at, updated_at";

/// 按阶段序排列（构思→见刊），同级按更新时间倒序
pub fn list(conn: &Connection) -> Result<Vec<Paper>, String> {
    let sql = format!(
        "SELECT {PAPER_COLUMNS} FROM paper WHERE deleted_at IS NULL
         ORDER BY CASE stage
             WHEN '构思' THEN 1 WHEN '撰写中' THEN 2 WHEN '组内审' THEN 3
             WHEN '投稿中' THEN 4 WHEN '审稿中' THEN 5 WHEN '返修' THEN 6
             WHEN '录用' THEN 7 WHEN '见刊' THEN 8 ELSE 9 END,
             (updated_at IS NULL), updated_at DESC, id DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Paper::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

/// 放榜 DDL 同步：应提醒则建/更新一条 ddl，不应提醒则办结既有 ddl
fn sync_paper_ddl(
    conn: &Connection,
    title: &str,
    venue: Option<&str>,
    stage: &str,
    expect_reply: Option<&str>,
    ddl_id: Option<i64>,
) -> Result<Option<i64>, String> {
    let due = clean(expect_reply.map(|s| s.to_string()));
    if reply_stage(stage)
        && let Some(due) = due
    {
        let name = match venue {
            Some(v) => format!("放榜：{title}（{v}）"),
            None => format!("放榜：{title}"),
        };
        match ddl_id {
            Some(id) => {
                conn.execute(
                    "UPDATE ddl SET name=?2, due_date=?3, status='生效', updated_at=?4
                     WHERE id=?1",
                    params![id, name, due, now_str()],
                )
                .map_err(|e| e.to_string())?;
                Ok(Some(id))
            }
            None => {
                conn.execute(
                    "INSERT INTO ddl(name, category, due_date, project_id, status, note)
                     VALUES(?1, '论文放榜', ?2, NULL, '生效', '由论文流水线自动生成')",
                    params![name, due],
                )
                .map_err(|e| e.to_string())?;
                Ok(Some(conn.last_insert_rowid()))
            }
        }
    } else if let Some(id) = ddl_id {
        conn.execute(
            "UPDATE ddl SET status='已办结', updated_at=?2 WHERE id=?1",
            params![id, now_str()],
        )
        .map_err(|e| e.to_string())?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

fn validate_stage(stage: &str) -> Result<(), String> {
    if STAGES.contains(&stage) {
        Ok(())
    } else {
        Err(format!("未知阶段：{stage}"))
    }
}

pub fn create(conn: &Connection, input: PaperInput) -> Result<i64, String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("论文标题不能为空".into());
    }
    let stage = input.stage.clone().unwrap_or_else(|| "构思".into());
    validate_stage(&stage)?;
    let venue = clean(input.venue.clone());
    let expect = clean(input.expect_reply.clone());

    let ddl_id = if reply_stage(&stage) && expect.is_some() {
        sync_paper_ddl(
            conn,
            &title,
            venue.as_deref(),
            &stage,
            expect.as_deref(),
            None,
        )?
    } else {
        None
    };

    conn.execute(
        "INSERT INTO paper(title, venue, stage, submit_date, expect_reply, revise_deadline, project_id, ddl_id, note)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            title,
            venue,
            stage,
            clean(input.submit_date),
            expect,
            clean(input.revise_deadline),
            input.project_id,
            ddl_id,
            clean(input.note)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑；放榜 DDL 随阶段与日期联动
pub fn update(conn: &Connection, id: i64, input: PaperInput) -> Result<(), String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("论文标题不能为空".into());
    }
    let stage = input.stage.clone().unwrap_or_else(|| "构思".into());
    validate_stage(&stage)?;
    let venue = clean(input.venue.clone());
    let expect = clean(input.expect_reply.clone());

    let (old_ddl, old_stage, old_venue, old_project): (
        Option<i64>,
        String,
        Option<String>,
        Option<i64>,
    ) = conn
        .query_row(
            "SELECT ddl_id, stage, venue, project_id FROM paper WHERE id=?1 AND deleted_at IS NULL",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| "论文不存在".to_string())?;

    let ddl_id = sync_paper_ddl(
        conn,
        &title,
        venue.as_deref(),
        &stage,
        expect.as_deref(),
        old_ddl,
    )?;

    let changed = conn
        .execute(
            "UPDATE paper SET title=?2, venue=?3, stage=?4, submit_date=?5, expect_reply=?6,
                 revise_deadline=?7, project_id=?8, ddl_id=?9, note=?10, updated_at=?11
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                title,
                venue,
                stage,
                clean(input.submit_date),
                expect,
                clean(input.revise_deadline),
                input.project_id,
                ddl_id,
                clean(input.note),
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("论文不存在".into());
    }

    // 流转到「录用」：自动写入成果库（按 paper_id 去重）
    if old_stage != "录用" && stage == "录用" {
        super::achievements::auto_from_paper(conn, id, &title, old_venue.as_deref(), old_project)?;
    }
    Ok(())
}

/// 软删除；关联放榜 DDL 一并办结
pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE paper SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE ddl SET status='已办结', updated_at=?2
         WHERE id = (SELECT ddl_id FROM paper WHERE id=?1) AND status='生效'",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    fn input(stage: &str, expect: Option<&str>) -> PaperInput {
        PaperInput {
            title: "ACSAMI 论文".into(),
            venue: Some("ACS Applied Materials".into()),
            stage: Some(stage.into()),
            submit_date: Some("2026-10-01".into()),
            expect_reply: expect.map(|s| s.to_string()),
            revise_deadline: None,
            project_id: None,
            note: None,
        }
    }

    fn ddl_of(conn: &Connection, id: i64) -> (String, String) {
        conn.query_row(
            "SELECT name, status FROM ddl WHERE id=?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    fn reply_stage_creates_and_settles_ddl() {
        let conn = test_conn();
        let id = create(&conn, input("投稿中", Some("2027-01-15"))).unwrap();

        // 放榜 DDL 已生成并关联
        let did: Option<i64> = conn
            .query_row("SELECT ddl_id FROM paper WHERE id=?1", params![id], |r| {
                r.get(0)
            })
            .unwrap();
        let did = did.expect("应有 ddl");
        let (name, status) = ddl_of(&conn, did);
        assert!(name.contains("放榜"));
        assert!(name.contains("ACSAMI 论文"));
        assert_eq!(status, "生效");

        // 日期变更 → 同步更新
        update(&conn, id, input("审稿中", Some("2027-02-20"))).unwrap();
        let due: String = conn
            .query_row("SELECT due_date FROM ddl WHERE id=?1", params![did], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(due, "2027-02-20");

        // 录用 → DDL 办结
        update(&conn, id, input("录用", None)).unwrap();
        let (_, status2) = ddl_of(&conn, did);
        assert_eq!(status2, "已办结");
    }

    #[test]
    fn idea_stage_no_ddl_and_delete_settles() {
        let conn = test_conn();
        create(&conn, input("构思", Some("2027-01-15"))).unwrap(); // 非提醒阶段：不建 DDL
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM ddl", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);

        let id2 = create(&conn, input("审稿中", Some("2027-03-01"))).unwrap();
        delete(&conn, id2).unwrap();
        let n2: i64 = conn
            .query_row("SELECT COUNT(*) FROM ddl WHERE status='生效'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(n2, 0);
        assert_eq!(list(&conn).unwrap().len(), 1); // 只剩构思那篇
    }

    #[test]
    fn list_orders_by_stage() {
        let conn = test_conn();
        for stage in ["见刊", "构思", "投稿中"] {
            let mut i = input(stage, None);
            i.title = format!("{stage}论文");
            create(&conn, i).unwrap();
        }
        let papers = list(&conn).unwrap();
        let stages: Vec<&str> = papers.iter().map(|p| p.stage.as_str()).collect();
        assert_eq!(stages, vec!["构思", "投稿中", "见刊"]);
    }
}
