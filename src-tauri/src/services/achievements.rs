//! 成果库与毕业要求（说明书 §4.9）：CRUD、论文录用自动生成成果、清单导出 Markdown

use crate::models::{Achievement, GradRequirement};
use rusqlite::{Connection, params};

use super::tasks::{now_str, today_str};

pub const KINDS: [&str; 7] = [
    "期刊论文",
    "会议论文",
    "发明专利",
    "实用新型",
    "软件著作权",
    "竞赛获奖",
    "其他",
];

/// 满足毕业口径的默认状态集（在投不计）
pub const SATISFYING_STATUSES: &str = "已录用,已发表,已授权,已获奖,已登记";

fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

// ---------- 成果 ----------

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementInput {
    pub kind: String,
    pub title: String,
    pub author_rank: Option<String>,
    pub status: Option<String>,
    pub date: Option<String>,
    pub venue: Option<String>,
    pub doi: Option<String>,
    pub project_id: Option<i64>,
    pub note: Option<String>,
}

const ACH_COLUMNS: &str = "id, kind, title, author_rank, status, date, venue, doi, \
     project_id, paper_id, note, created_at";

pub fn list_achievements(conn: &Connection) -> Result<Vec<Achievement>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {ACH_COLUMNS} FROM achievement WHERE deleted_at IS NULL
             ORDER BY (date IS NULL), date DESC, id DESC"
        ))
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Achievement::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

fn validate(kind: &str) -> Result<(), String> {
    if KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(format!("未知成果类型：{kind}"))
    }
}

pub fn create_achievement(conn: &Connection, input: AchievementInput) -> Result<i64, String> {
    validate(&input.kind)?;
    let title = input.title.trim();
    if title.is_empty() {
        return Err("成果标题不能为空".into());
    }
    conn.execute(
        "INSERT INTO achievement(kind, title, author_rank, status, date, venue, doi, project_id, note)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.kind,
            title,
            clean(input.author_rank),
            input.status.unwrap_or_else(|| "已发表".into()),
            clean(input.date),
            clean(input.venue),
            clean(input.doi),
            input.project_id,
            clean(input.note)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑（paper_id 不变）
pub fn update_achievement(
    conn: &Connection,
    id: i64,
    input: AchievementInput,
) -> Result<(), String> {
    validate(&input.kind)?;
    let title = input.title.trim();
    if title.is_empty() {
        return Err("成果标题不能为空".into());
    }
    let changed = conn
        .execute(
            "UPDATE achievement SET kind=?2, title=?3, author_rank=?4, status=?5, date=?6,
                 venue=?7, doi=?8, project_id=?9, note=?10, updated_at=?11
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                input.kind,
                title,
                clean(input.author_rank),
                input.status.unwrap_or_else(|| "已发表".into()),
                clean(input.date),
                clean(input.venue),
                clean(input.doi),
                input.project_id,
                clean(input.note),
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("成果不存在".into());
    }
    Ok(())
}

pub fn delete_achievement(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE achievement SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 论文流转到「录用」时自动生成成果（按 paper_id 去重），返回新成果 id
pub fn auto_from_paper(
    conn: &Connection,
    paper_id: i64,
    title: &str,
    venue: Option<&str>,
    project_id: Option<i64>,
) -> Result<Option<i64>, String> {
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM achievement WHERE paper_id = ?1 AND deleted_at IS NULL",
            params![paper_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exists > 0 {
        return Ok(None);
    }
    let kind = match venue {
        Some(v) if v.contains("会议") || v.to_lowercase().contains("conference") => "会议论文",
        _ => "期刊论文",
    };
    conn.execute(
        "INSERT INTO achievement(kind, title, status, date, venue, project_id, paper_id, note)
         VALUES(?1, ?2, '已录用', ?3, ?4, ?5, ?6, '论文录用后自动生成')",
        params![kind, title, today_str(), venue, project_id, paper_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(conn.last_insert_rowid()))
}

/// 个人成果清单 Markdown（奖助申报 / 年度总结用）
pub fn export_markdown(conn: &Connection) -> Result<(String, String), String> {
    let list = list_achievements(conn)?;
    let today = today_str();
    let mut md = format!("# 个人成果清单（{today}）\n\n共 {} 项\n", list.len());
    for kind in KINDS {
        let rows: Vec<&Achievement> = list.iter().filter(|a| a.kind == kind).collect();
        if rows.is_empty() {
            continue;
        }
        md.push_str(&format!("\n## {kind}（{}）\n", rows.len()));
        for a in rows {
            let mut line = format!("- {}", a.title);
            if let Some(r) = &a.author_rank {
                line.push_str(&format!("（{r}）"));
            }
            if let Some(v) = &a.venue {
                line.push_str(&format!("· {v}"));
            }
            if let Some(d) = &a.date {
                line.push_str(&format!(" · {}", &d[..4]));
            }
            if let Some(doi) = &a.doi {
                line.push_str(&format!(" · DOI: {doi}"));
            }
            md.push_str(&line);
            md.push('\n');
        }
    }
    Ok((format!("成果清单-{today}.md"), md))
}

// ---------- 毕业要求 ----------

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementInput {
    pub item: String,
    pub need_count: Option<i64>,
    pub match_kinds: Option<String>,
    pub match_status: Option<String>,
    pub note: Option<String>,
    pub sort: Option<i64>,
}

fn normalize_csv(v: Option<String>) -> String {
    v.unwrap_or_default()
        .split(['，', ','])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn list_requirements(conn: &Connection) -> Result<Vec<GradRequirement>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, item, need_count, match_kinds, match_status, note, sort
             FROM grad_requirement WHERE deleted_at IS NULL
             ORDER BY sort, id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], GradRequirement::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn create_requirement(conn: &Connection, input: RequirementInput) -> Result<i64, String> {
    let item = input.item.trim();
    if item.is_empty() {
        return Err("要求条目不能为空".into());
    }
    let match_status = match normalize_csv(input.match_status) {
        s if s.is_empty() => SATISFYING_STATUSES.to_string(),
        s => s,
    };
    conn.execute(
        "INSERT INTO grad_requirement(item, need_count, match_kinds, match_status, note, sort)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            item,
            input.need_count.unwrap_or(1).max(1),
            normalize_csv(input.match_kinds),
            match_status,
            clean(input.note),
            input.sort.unwrap_or(0)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_requirement(
    conn: &Connection,
    id: i64,
    input: RequirementInput,
) -> Result<(), String> {
    let item = input.item.trim();
    if item.is_empty() {
        return Err("要求条目不能为空".into());
    }
    let match_status = match normalize_csv(input.match_status) {
        s if s.is_empty() => SATISFYING_STATUSES.to_string(),
        s => s,
    };
    let changed = conn
        .execute(
            "UPDATE grad_requirement SET item=?2, need_count=?3, match_kinds=?4, match_status=?5,
                 note=?6, sort=?7, updated_at=?8
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                item,
                input.need_count.unwrap_or(1).max(1),
                normalize_csv(input.match_kinds),
                match_status,
                clean(input.note),
                input.sort.unwrap_or(0),
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("要求条目不存在".into());
    }
    Ok(())
}

pub fn delete_requirement(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE grad_requirement SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    fn ach(kind: &str, status: &str) -> AchievementInput {
        AchievementInput {
            kind: kind.into(),
            title: "某成果".into(),
            author_rank: Some("第一作者".into()),
            status: Some(status.into()),
            date: Some("2026-10-01".into()),
            venue: Some("某期刊".into()),
            doi: None,
            project_id: None,
            note: None,
        }
    }

    #[test]
    fn auto_from_paper_dedupes_and_kinds() {
        let conn = test_conn();
        let paper_input = |title: &str, venue: &str| crate::services::papers::PaperInput {
            title: title.into(),
            venue: Some(venue.into()),
            stage: Some("投稿中".into()),
            submit_date: None,
            expect_reply: None,
            revise_deadline: None,
            project_id: None,
            note: None,
        };
        let pid1 =
            crate::services::papers::create(&conn, paper_input("论文A", "某国际会议")).unwrap();
        let pid2 = crate::services::papers::create(&conn, paper_input("论文B", "ACSAMI")).unwrap();

        let a = auto_from_paper(&conn, pid1, "论文A", Some("某国际会议"), None)
            .unwrap()
            .unwrap();
        // 同一论文再录用不重复生成
        assert!(
            auto_from_paper(&conn, pid1, "论文A", Some("某国际会议"), None)
                .unwrap()
                .is_none()
        );
        let kind: String = conn
            .query_row(
                "SELECT kind FROM achievement WHERE id=?1",
                params![a],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kind, "会议论文");
        let b = auto_from_paper(&conn, pid2, "论文B", Some("ACSAMI"), None)
            .unwrap()
            .unwrap();
        let kind2: String = conn
            .query_row(
                "SELECT kind FROM achievement WHERE id=?1",
                params![b],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kind2, "期刊论文");
    }

    #[test]
    fn export_markdown_groups_by_kind() {
        let conn = test_conn();
        create_achievement(&conn, ach("期刊论文", "已发表")).unwrap();
        create_achievement(&conn, ach("发明专利", "已授权")).unwrap();
        let (name, md) = export_markdown(&conn).unwrap();
        assert!(name.starts_with("成果清单-"));
        assert!(md.contains("共 2 项"));
        assert!(md.contains("## 期刊论文（1）"));
        assert!(md.contains("## 发明专利（1）"));
        assert!(md.contains("第一作者"));
    }
}
