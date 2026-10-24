//! 任务领域逻辑（说明书 §4.1/§4.2）：列表视图、增删改、完成与重复任务

use crate::models::Task;
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{Connection, params};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskNew {
    pub title: String,
    pub note: Option<String>,
    pub project_id: Option<i64>,
    pub priority: Option<i64>,
    pub due_date: Option<String>,
    pub repeat_rule: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPatch {
    pub title: String,
    pub note: Option<String>,
    pub project_id: Option<i64>,
    pub priority: Option<i64>,
    pub due_date: Option<String>,
    pub status: Option<String>,
    pub repeat_rule: Option<String>,
}

pub fn now_str() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

pub fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

const TASK_COLUMNS: &str = "t.id, t.title, t.note, t.project_id, p.name, t.milestone_id, \
     t.priority, t.due_date, t.status, t.repeat_rule, t.completed_at, t.created_at";

/// view: today / inbox / active / done / all
pub fn list(
    conn: &Connection,
    view: &str,
    project_id: Option<i64>,
    today: &str,
) -> Result<Vec<Task>, String> {
    let filter = match view {
        "today" => {
            format!(
                "t.status IN ('待办','进行中') AND t.due_date IS NOT NULL AND t.due_date <= '{today}'"
            )
        }
        "inbox" => "t.status IN ('待办','进行中') AND t.project_id IS NULL".to_string(),
        "done" => "t.status IN ('完成','放弃')".to_string(),
        _ => "t.status IN ('待办','进行中')".to_string(),
    };
    let mut sql = format!(
        "SELECT {TASK_COLUMNS} FROM task t \
         LEFT JOIN project p ON p.id = t.project_id AND p.deleted_at IS NULL \
         WHERE t.deleted_at IS NULL AND ({filter})"
    );
    if let Some(pid) = project_id
        && matches!(view, "active" | "done" | "all")
    {
        sql.push_str(&format!(" AND t.project_id = {pid}"));
    }
    sql.push_str(match view {
        "today" => " ORDER BY t.due_date, t.priority, t.id",
        "inbox" => " ORDER BY t.priority, t.id DESC",
        "done" => " ORDER BY t.completed_at DESC LIMIT 300",
        _ => " ORDER BY (t.due_date IS NULL), t.due_date, t.priority, t.id",
    });

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Task::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn create(conn: &Connection, t: TaskNew) -> Result<i64, String> {
    let title = t.title.trim();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    conn.execute(
        "INSERT INTO task(title, note, project_id, priority, due_date, repeat_rule)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            title,
            clean(t.note),
            t.project_id,
            t.priority.unwrap_or(2),
            clean(t.due_date),
            clean(t.repeat_rule)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑（前端从编辑抽屉提交完整对象）
pub fn update(conn: &Connection, id: i64, p: TaskPatch) -> Result<(), String> {
    let title = p.title.trim();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    let (old_status, old_completed): (String, Option<String>) = conn
        .query_row(
            "SELECT status, completed_at FROM task WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "任务不存在".to_string())?;
    let new_status = p.status.clone().unwrap_or_else(|| old_status.clone());
    // 状态迁移时维护 completed_at；完成→完成保留原时刻
    let completed_at = match (old_status.as_str(), new_status.as_str()) {
        ("完成", "完成") => old_completed,
        (_, "完成") => Some(now_str()),
        _ => None,
    };
    conn.execute(
        "UPDATE task SET title=?2, note=?3, project_id=?4, priority=?5, due_date=?6,
             status=?7, repeat_rule=?8, completed_at=?9, updated_at=?10
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            title,
            clean(p.note),
            p.project_id,
            p.priority.unwrap_or(2),
            clean(p.due_date),
            new_status,
            clean(p.repeat_rule),
            completed_at,
            now_str()
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 完成；若为重复任务则生成下一次出现，返回新任务 id
pub fn complete(conn: &Connection, id: i64) -> Result<Option<i64>, String> {
    struct TaskCore {
        title: String,
        note: Option<String>,
        project_id: Option<i64>,
        priority: Option<i64>,
        due_date: Option<String>,
        repeat_rule: Option<String>,
    }
    let core = conn
        .query_row(
            "SELECT title, note, project_id, priority, due_date, repeat_rule
             FROM task WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            |r| {
                Ok(TaskCore {
                    title: r.get(0)?,
                    note: r.get(1)?,
                    project_id: r.get(2)?,
                    priority: r.get(3)?,
                    due_date: r.get(4)?,
                    repeat_rule: r.get(5)?,
                })
            },
        )
        .map_err(|_| "任务不存在".to_string())?;
    conn.execute(
        "UPDATE task SET status='完成', completed_at=?2, updated_at=?2 WHERE id=?1",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;

    if let Some(rule) = clean(core.repeat_rule).as_deref()
        && let Some(next) =
            next_occurrence(rule, core.due_date.as_deref(), Local::now().date_naive())
    {
        conn.execute(
            "INSERT INTO task(title, note, project_id, priority, due_date, status, repeat_rule)
             VALUES(?1, ?2, ?3, ?4, ?5, '待办', ?6)",
            params![
                core.title,
                core.note,
                core.project_id,
                core.priority.unwrap_or(2),
                next.format("%Y-%m-%d").to_string(),
                rule
            ],
        )
        .map_err(|e| e.to_string())?;
        return Ok(Some(conn.last_insert_rowid()));
    }
    Ok(None)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE task SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 重复规则：daily / weekly:N（N=1..7，1=周一）。下一次出现在 base（due 与今天的较大者）之后
pub fn next_occurrence(rule: &str, due: Option<&str>, today: NaiveDate) -> Option<NaiveDate> {
    let due = due.and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let base = due.filter(|d| *d > today).unwrap_or(today);
    match rule {
        "daily" => Some(base + Duration::days(1)),
        r => {
            let n: u32 = r.strip_prefix("weekly:")?.parse().ok()?;
            if !(1..=7).contains(&n) {
                return None;
            }
            let target = (n - 1) as i64;
            let mut delta = (target - base.weekday().num_days_from_monday() as i64).rem_euclid(7);
            if delta == 0 {
                delta = 7;
            }
            Some(base + Duration::days(delta))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;
    use chrono::Weekday;

    #[test]
    fn next_occurrence_rules() {
        let mon = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(); // 周一
        assert_eq!(
            next_occurrence("daily", Some("2026-10-26"), mon),
            Some(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap())
        );
        // 周一完成后，下一个周一 +7 天
        assert_eq!(
            next_occurrence("weekly:1", Some("2026-10-26"), mon),
            Some(NaiveDate::from_ymd_opt(2026, 11, 2).unwrap())
        );
        // 从周一找本周五：+4
        assert_eq!(
            next_occurrence("weekly:5", Some("2026-10-26"), mon),
            Some(NaiveDate::from_ymd_opt(2026, 10, 30).unwrap())
        );
        // 错过的重复（due 在过去）以今天为基准
        assert_eq!(
            next_occurrence("daily", Some("2020-01-01"), mon),
            Some(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap())
        );
        assert_eq!(next_occurrence("weekly:9", None, mon), None);
        assert_eq!(next_occurrence("monthly:1", None, mon), None);
    }

    #[test]
    fn complete_generates_next_weekly_occurrence() {
        let conn = test_conn();
        // 取一个远期的周一，保证不受真实日期影响
        let probe = NaiveDate::from_ymd_opt(2099, 1, 1).unwrap();
        let monday = (0..7)
            .map(|i| probe + Duration::days(i))
            .find(|d| d.weekday() == Weekday::Mon)
            .unwrap();
        conn.execute(
            "INSERT INTO task(title, due_date, repeat_rule, priority) VALUES('组会PPT', ?1, 'weekly:1', 2)",
            params![monday.format("%Y-%m-%d").to_string()],
        )
        .unwrap();

        let new_id = complete(&conn, 1).unwrap().expect("应生成下一次出现");
        let (title, due, status): (String, String, String) = conn
            .query_row(
                "SELECT title, due_date, status FROM task WHERE id = ?1",
                params![new_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(title, "组会PPT");
        assert_eq!(
            due,
            (monday + Duration::days(7)).format("%Y-%m-%d").to_string()
        );
        assert_eq!(status, "待办");

        let (st, comp): (String, String) = conn
            .query_row(
                "SELECT status, completed_at FROM task WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(st, "完成");
        assert!(!comp.is_empty());
    }

    #[test]
    fn complete_without_rule_no_new_task() {
        let conn = test_conn();
        conn.execute("INSERT INTO task(title) VALUES('一次性')", [])
            .unwrap();
        assert!(complete(&conn, 1).unwrap().is_none());
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM task", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn today_view_includes_overdue_excludes_future() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO task(title, due_date) VALUES('逾期','2020-01-01'), ('未来','2999-01-01'), ('今天到期','2026-10-26')",
            [],
        )
        .unwrap();
        let list = list(&conn, "today", None, "2026-10-26").unwrap();
        let titles: Vec<&str> = list.iter().map(|t| t.title.as_str()).collect();
        assert!(titles.contains(&"逾期"));
        assert!(titles.contains(&"今天到期"));
        assert!(!titles.contains(&"未来"));
    }

    #[test]
    fn update_moves_status_and_completed_at() {
        let conn = test_conn();
        conn.execute("INSERT INTO task(title) VALUES('编辑')", [])
            .unwrap();
        update(
            &conn,
            1,
            TaskPatch {
                title: "编辑后".into(),
                note: None,
                project_id: None,
                priority: Some(0),
                due_date: Some("2026-11-01".into()),
                status: Some("进行中".into()),
                repeat_rule: None,
            },
        )
        .unwrap();
        let (title, status, pri): (String, String, i64) = conn
            .query_row(
                "SELECT title, status, priority FROM task WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            (title.as_str(), status.as_str(), pri),
            ("编辑后", "进行中", 0)
        );

        update(
            &conn,
            1,
            TaskPatch {
                title: "编辑后".into(),
                note: None,
                project_id: None,
                priority: Some(0),
                due_date: Some("2026-11-01".into()),
                status: Some("完成".into()),
                repeat_rule: None,
            },
        )
        .unwrap();
        let (status, comp): (String, Option<String>) = conn
            .query_row(
                "SELECT status, completed_at FROM task WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "完成");
        assert!(comp.is_some());
    }
}
