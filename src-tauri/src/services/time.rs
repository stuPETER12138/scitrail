//! DDL 与学位里程碑（说明书 §4.3）：CRUD、办结、复制下一年

use crate::models::{Ddl, Milestone};
use chrono::NaiveDate;
use rusqlite::{Connection, params};

use super::tasks::now_str;

// ---------- DDL ----------

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdlInput {
    pub name: String,
    pub category: Option<String>,
    pub due_date: String, // 'YYYY-MM-DD' 或 'YYYY-MM-DD HH:MM'
    pub lead_days: Option<String>,
    pub project_id: Option<i64>,
    pub repeat_yearly: Option<bool>,
    pub note: Option<String>,
    pub status: Option<String>,
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn valid_due(due: &str) -> bool {
    let date = due.split_whitespace().next().unwrap_or_default();
    NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

pub fn list_ddls(conn: &Connection, active_only: bool) -> Result<Vec<Ddl>, String> {
    let mut sql =
        "SELECT id, name, category, due_date, lead_days, project_id, status, repeat_yearly, note
         FROM ddl WHERE deleted_at IS NULL"
            .to_string();
    if active_only {
        sql.push_str(" AND status = '生效'");
    }
    sql.push_str(" ORDER BY substr(due_date,1,10)");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Ddl::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn create_ddl(conn: &Connection, input: DdlInput) -> Result<i64, String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("名称不能为空".into());
    }
    if !valid_due(&input.due_date) {
        return Err("截止日期格式应为 YYYY-MM-DD（可带 HH:MM）".into());
    }
    conn.execute(
        "INSERT INTO ddl(name, category, due_date, lead_days, project_id, status, repeat_yearly, note)
         VALUES(?1, ?2, ?3, ?4, ?5, '生效', ?6, ?7)",
        params![
            name,
            clean(input.category),
            input.due_date.trim(),
            clean(input.lead_days).unwrap_or_else(|| "7,3,1".into()),
            input.project_id,
            input.repeat_yearly.unwrap_or(false) as i64,
            clean(input.note)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑
pub fn update_ddl(conn: &Connection, id: i64, input: DdlInput) -> Result<(), String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("名称不能为空".into());
    }
    if !valid_due(&input.due_date) {
        return Err("截止日期格式应为 YYYY-MM-DD（可带 HH:MM）".into());
    }
    let changed = conn
        .execute(
            "UPDATE ddl SET name=?2, category=?3, due_date=?4, lead_days=?5, project_id=?6,
                 status=?7, repeat_yearly=?8, note=?9, updated_at=?10
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                name,
                clean(input.category),
                input.due_date.trim(),
                clean(input.lead_days).unwrap_or_else(|| "7,3,1".into()),
                input.project_id,
                input.status.clone().unwrap_or_else(|| "生效".into()),
                input.repeat_yearly.unwrap_or(false) as i64,
                clean(input.note),
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("DDL 不存在".into());
    }
    Ok(())
}

pub fn delete_ddl(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE ddl SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 复制下一年（会议截稿年年有）：原条目办结，生成明年新条目
pub fn rollover_ddl(conn: &Connection, id: i64) -> Result<i64, String> {
    let row: (
        String,
        Option<String>,
        String,
        Option<i64>,
        bool,
        Option<String>,
    ) = conn
        .query_row(
            "SELECT name, category, due_date, project_id, repeat_yearly, note
             FROM ddl WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get::<_, i64>(4)? != 0,
                    r.get(5)?,
                ))
            },
        )
        .map_err(|_| "DDL 不存在".to_string())?;
    let (name, category, due, project_id, _repeat, note) = row;
    let next_due = bump_year(&due)?;

    conn.execute(
        "UPDATE ddl SET status='已办结', updated_at=?2 WHERE id=?1",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO ddl(name, category, due_date, lead_days, project_id, status, repeat_yearly, note)
         VALUES(?1, ?2, ?3, '7,3,1', ?4, '生效', 1, ?5)",
        params![name, category, next_due, project_id, note],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 下一年同月日；2/29 顺延无则落 2/28；保留可选时间部分
fn bump_year(due: &str) -> Result<String, String> {
    let mut parts = due.splitn(2, ' ');
    let date = parts.next().unwrap_or_default();
    let time = parts.next();
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| "日期格式错误".to_string())?;
    use chrono::Datelike;
    let next = NaiveDate::from_ymd_opt(d.year() + 1, d.month(), d.day())
        .or_else(|| NaiveDate::from_ymd_opt(d.year() + 1, d.month(), d.day() - 1))
        .ok_or("日期计算失败")?;
    Ok(match time {
        Some(t) => format!("{next} {t}"),
        None => next.to_string(),
    })
}

// ---------- 里程碑 ----------

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneInput {
    pub name: String,
    pub target_date: Option<String>,
    pub status: Option<String>,
    pub note: Option<String>,
}

pub fn list_milestones(conn: &Connection) -> Result<Vec<Milestone>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, target_date, status, done_at, note
             FROM milestone WHERE deleted_at IS NULL
             ORDER BY (target_date IS NULL), target_date, id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Milestone::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn create_milestone(conn: &Connection, input: MilestoneInput) -> Result<i64, String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("名称不能为空".into());
    }
    conn.execute(
        "INSERT INTO milestone(name, target_date, status, note) VALUES(?1, ?2, ?3, ?4)",
        params![
            name,
            clean(input.target_date),
            input.status.unwrap_or_else(|| "未开始".into()),
            clean(input.note)
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn update_milestone(conn: &Connection, id: i64, input: MilestoneInput) -> Result<(), String> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err("名称不能为空".into());
    }
    let old_status: String = conn
        .query_row(
            "SELECT status FROM milestone WHERE id=?1 AND deleted_at IS NULL",
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| "里程碑不存在".to_string())?;
    let new_status = input.status.clone().unwrap_or(old_status.clone());
    let done_at = match (old_status.as_str(), new_status.as_str()) {
        ("已完成", "已完成") => None, // 保持原值，SQL 里 COALESCE 处理
        (_, "已完成") => Some(now_str()),
        _ => None,
    };
    let changed = conn
        .execute(
            "UPDATE milestone SET name=?2, target_date=?3, status=?4,
                 done_at=CASE WHEN ?5 IS NOT NULL THEN ?5
                              WHEN ?6='已完成' THEN done_at
                              ELSE NULL END,
                 note=?7, updated_at=?8
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                name,
                clean(input.target_date),
                new_status,
                done_at,
                new_status,
                clean(input.note),
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("里程碑不存在".into());
    }
    Ok(())
}

pub fn delete_milestone(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE milestone SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    #[test]
    fn bump_year_handles_feb29_and_time() {
        assert_eq!(bump_year("2028-02-29").unwrap(), "2029-02-28");
        assert_eq!(bump_year("2027-03-01 23:59").unwrap(), "2028-03-01 23:59");
        assert!(bump_year("bad").is_err());
    }

    #[test]
    fn rollover_settles_old_and_creates_next_year() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO ddl(name, due_date, repeat_yearly) VALUES('ACSAMI 截稿','2027-03-01',1)",
            [],
        )
        .unwrap();
        let new_id = rollover_ddl(&conn, 1).unwrap();
        let (old_status,): (String,) = conn
            .query_row("SELECT status FROM ddl WHERE id=1", [], |r| {
                Ok((r.get(0)?,))
            })
            .unwrap();
        assert_eq!(old_status, "已办结");
        let (name, due, status, rp): (String, String, String, i64) = conn
            .query_row(
                "SELECT name, due_date, status, repeat_yearly FROM ddl WHERE id=?1",
                params![new_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(
            (name.as_str(), due.as_str(), status.as_str(), rp),
            ("ACSAMI 截稿", "2028-03-01", "生效", 1)
        );
    }

    #[test]
    fn ddl_list_active_only_ordered() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO ddl(name, due_date, status) VALUES
             ('b','2027-06-01','生效'), ('a','2027-01-01','生效'), ('c','2027-02-01','已办结')",
            [],
        )
        .unwrap();
        let active = list_ddls(&conn, true).unwrap();
        assert_eq!(
            active.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn milestone_complete_sets_done_at() {
        let conn = test_conn();
        conn.execute(
            "INSERT INTO milestone(name, target_date) VALUES('开题','2029-03-01')",
            [],
        )
        .unwrap();
        update_milestone(
            &conn,
            1,
            MilestoneInput {
                name: "开题".into(),
                target_date: Some("2029-03-01".into()),
                status: Some("已完成".into()),
                note: None,
            },
        )
        .unwrap();
        let (status, done): (String, Option<String>) = conn
            .query_row(
                "SELECT status, done_at FROM milestone WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "已完成");
        assert!(done.is_some());

        // 再次保存为已完成：保留原时刻
        update_milestone(
            &conn,
            1,
            MilestoneInput {
                name: "开题".into(),
                target_date: None,
                status: Some("已完成".into()),
                note: None,
            },
        )
        .unwrap();
        let done2: Option<String> = conn
            .query_row("SELECT done_at FROM milestone WHERE id=1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(done, done2);
    }
}
