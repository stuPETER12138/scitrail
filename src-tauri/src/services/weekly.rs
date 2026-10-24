//! 周报（说明书 §4.3）：从任务与条目聚合生成本周 Markdown 草稿，存档可编辑

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReport {
    pub week_start: String,
    pub content: String,
    pub generated_at: Option<String>,
    pub edited_at: Option<String>,
}

/// 解析/默认本周周一（week_start）
pub fn monday_of(input: Option<&str>) -> Result<String, String> {
    let base = match input {
        Some(s) => {
            NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| "日期格式应为 YYYY-MM-DD")?
        }
        None => Local::now().date_naive(),
    };
    let monday = base - Duration::days(base.weekday().num_days_from_monday() as i64);
    Ok(monday.format("%Y-%m-%d").to_string())
}

fn add_days(date: &str, n: i64) -> String {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    (d + Duration::days(n)).format("%Y-%m-%d").to_string()
}

/// '2026-10-20' → '10月20日'
fn md(date: &str) -> String {
    format!("{}月{}日", date[5..7].trim_start_matches('0'), &date[8..10])
}

fn now_full() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// 生成并落档（按 week_start UPSERT，重复生成覆盖）
pub fn generate(conn: &Connection, week_start: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(week_start, "%Y-%m-%d").map_err(|_| "日期格式应为 YYYY-MM-DD")?;
    let week_end = add_days(week_start, 6);
    let next_monday = add_days(week_start, 7);
    let next_sunday = add_days(week_start, 13);

    let mut md_doc = format!("# 科迹周报 · {week_start} ~ {week_end}\n\n");

    // 本周完成任务
    let mut stmt = conn
        .prepare(
            "SELECT t.title, p.name, substr(t.completed_at,1,10)
             FROM task t LEFT JOIN project p ON p.id = t.project_id AND p.deleted_at IS NULL
             WHERE t.deleted_at IS NULL AND t.status='完成' AND t.completed_at IS NOT NULL
               AND substr(t.completed_at,1,10) BETWEEN ?1 AND ?2
             ORDER BY t.completed_at",
        )
        .map_err(|e| e.to_string())?;
    let done: Vec<(String, Option<String>, String)> = stmt
        .query_map(params![week_start, week_end], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    md_doc.push_str(&format!("## 本周完成（{}）\n", done.len()));
    if done.is_empty() {
        md_doc.push_str("（本周暂无完成任务）\n");
    }
    for (title, project, day) in &done {
        match project {
            Some(p) => md_doc.push_str(&format!("- [x] {title}（{p}）· {}\n", md(day))),
            None => md_doc.push_str(&format!("- [x] {title} · {}\n", md(day))),
        }
    }

    // 本周条目（按类型分组）
    let mut stmt = conn
        .prepare(
            "SELECT e.kind, e.date, e.time, e.title, p.name
             FROM entry e LEFT JOIN project p ON p.id = e.project_id AND p.deleted_at IS NULL
             WHERE e.deleted_at IS NULL AND e.date BETWEEN ?1 AND ?2
             ORDER BY e.date, (e.time IS NULL), e.time",
        )
        .map_err(|e| e.to_string())?;
    type EntryRow = (String, String, Option<String>, String, Option<String>);
    let entries: Vec<EntryRow> = stmt
        .query_map(params![week_start, week_end], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    md_doc.push_str("\n## 实验与记录\n");
    let kinds = [
        ("experiment", "🧪 实验"),
        ("literature", "📖 文献"),
        ("expense", "💰 报销"),
        ("meeting", "📅 组会"),
        ("note", "📝 笔记"),
    ];
    let mut any = false;
    for (kind, label) in kinds {
        let rows: Vec<_> = entries.iter().filter(|(k, ..)| k == kind).collect();
        if rows.is_empty() {
            continue;
        }
        any = true;
        md_doc.push_str(&format!("\n### {label}（{}）\n", rows.len()));
        for (_, date, time, title, project) in rows {
            match (time, project) {
                (Some(_), Some(p)) => {
                    md_doc.push_str(&format!("- {} {}（{p}）\n", md(date), title))
                }
                (Some(_), None) => md_doc.push_str(&format!("- {} {}\n", md(date), title)),
                (None, Some(p)) => md_doc.push_str(&format!("- {} {}（{p}）\n", md(date), title)),
                (None, None) => md_doc.push_str(&format!("- {} {}\n", md(date), title)),
            }
        }
    }
    if !any {
        md_doc.push_str("（本周暂无记录）\n");
    }

    md_doc.push_str("\n## 问题与风险\n\n<!-- 手动补充：阻塞点、需要导师/同学协调的事项 -->\n");

    // 下周计划
    let mut stmt = conn
        .prepare(
            "SELECT t.title, p.name, t.due_date FROM task t
             LEFT JOIN project p ON p.id = t.project_id AND p.deleted_at IS NULL
             WHERE t.deleted_at IS NULL AND t.status IN ('待办','进行中')
               AND t.due_date BETWEEN ?1 AND ?2 ORDER BY t.due_date",
        )
        .map_err(|e| e.to_string())?;
    let next_tasks: Vec<(String, Option<String>, String)> = stmt
        .query_map(params![next_monday, next_sunday], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT name, substr(due_date,1,10) FROM ddl
             WHERE deleted_at IS NULL AND status='生效'
               AND substr(due_date,1,10) BETWEEN ?1 AND ?2 ORDER BY due_date",
        )
        .map_err(|e| e.to_string())?;
    let next_ddls: Vec<(String, String)> = stmt
        .query_map(params![next_monday, next_sunday], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    md_doc.push_str("\n## 下周计划\n");
    if next_tasks.is_empty() && next_ddls.is_empty() {
        md_doc.push_str("（待规划——可从任务页为下周任务设置截止日期）\n");
    }
    for (title, project, due) in &next_tasks {
        match project {
            Some(p) => md_doc.push_str(&format!("- 📌 {title}（{p}）· 截止 {}\n", md(due))),
            None => md_doc.push_str(&format!("- 📌 {title} · 截止 {}\n", md(due))),
        }
    }
    for (name, due) in &next_ddls {
        md_doc.push_str(&format!("- ⏰ DDL：{name} · {}\n", md(due)));
    }

    let now = now_full();
    conn.execute(
        "INSERT INTO weekly_report(week_start, content, generated_at) VALUES(?1, ?2, ?3)
         ON CONFLICT(week_start) DO UPDATE SET content=excluded.content, generated_at=excluded.generated_at",
        params![week_start, md_doc, now],
    )
    .map_err(|e| e.to_string())?;
    Ok(md_doc)
}

/// 保存编辑（不存在则建档）
pub fn save(conn: &Connection, week_start: &str, content: &str) -> Result<(), String> {
    NaiveDate::parse_from_str(week_start, "%Y-%m-%d").map_err(|_| "日期格式应为 YYYY-MM-DD")?;
    conn.execute(
        "INSERT INTO weekly_report(week_start, content, edited_at) VALUES(?1, ?2, ?3)
         ON CONFLICT(week_start) DO UPDATE SET content=excluded.content, edited_at=excluded.edited_at",
        params![week_start, content, now_full()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<WeeklyReport>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT week_start, content, generated_at, edited_at
             FROM weekly_report WHERE deleted_at IS NULL
             ORDER BY week_start DESC LIMIT 104",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(WeeklyReport {
                week_start: r.get(0)?,
                content: r.get(1)?,
                generated_at: r.get(2)?,
                edited_at: r.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use crate::services::entries::{self, EntryInput};

    #[test]
    fn generate_aggregates_and_upserts() {
        let conn = test_conn();
        let monday = monday_of(Some("2026-10-26")).unwrap(); // 周一
        assert_eq!(monday, "2026-10-26");

        conn.execute(
            "INSERT INTO task(title, status, completed_at, due_date) VALUES
             ('完成的事','完成','2026-10-27 10:00',NULL),
             ('下周的事','待办',NULL,'2026-11-02')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ddl(name, due_date) VALUES('下周截稿','2026-11-03')",
            [],
        )
        .unwrap();
        let mut e = EntryInput {
            date: Some("2026-10-28".into()),
            time: Some("09:12".into()),
            kind: "experiment".into(),
            title: "预烧结实验".into(),
            content: None,
            project_id: None,
            tags: None,
            fields: None,
            refs: None,
        };
        entries::create(&conn, e.clone()).unwrap();
        e.kind = "note".into();
        e.title = "一条笔记".into();
        e.date = Some("2026-10-29".into());
        entries::create(&conn, e).unwrap();

        let md_doc = generate(&conn, &monday).unwrap();
        assert!(md_doc.contains("本周完成（1）"));
        assert!(md_doc.contains("完成的事"));
        assert!(md_doc.contains("### 🧪 实验（1）"));
        assert!(md_doc.contains("10月28日 预烧结实验"));
        assert!(md_doc.contains("### 📝 笔记（1）"));
        assert!(md_doc.contains("下周的事"));
        assert!(md_doc.contains("DDL：下周截稿"));
        assert!(!md_doc.contains("📖 文献")); // 空类型不出现

        // 重复生成覆盖（UPSERT）
        let again = generate(&conn, &monday).unwrap();
        assert_eq!(again, md_doc);
        let reports = list(&conn).unwrap();
        assert_eq!(reports.len(), 1);

        save(&conn, &monday, "手动编辑后的内容").unwrap();
        assert_eq!(list(&conn).unwrap()[0].content, "手动编辑后的内容");
    }

    #[test]
    fn monday_validation() {
        assert_eq!(monday_of(Some("2026-10-28")).unwrap(), "2026-10-26"); // 周三 → 本周一
        assert!(monday_of(Some("bad")).is_err());
    }
}
