use crate::state::AppState;
use chrono::{Datelike, Duration, Local};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekStats {
    pub tasks_completed: i64,
    pub tasks_created: i64,
    pub entries_created: i64,
    pub week_start: String,
}

/// 本周（周一起）统计，今日页小结用
#[tauri::command]
pub fn stats_week(state: State<'_, AppState>) -> Result<WeekStats, String> {
    let today = Local::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let week_start = monday.format("%Y-%m-%d").to_string();
    let conn = state.conn()?;
    let count = |sql: &str| -> Result<i64, String> {
        conn.query_row(sql, [], |r| r.get(0))
            .map_err(|e| e.to_string())
    };
    Ok(WeekStats {
        tasks_completed: count(&format!(
            "SELECT COUNT(*) FROM task WHERE deleted_at IS NULL AND status='完成' \
             AND substr(completed_at,1,10) >= '{week_start}'"
        ))?,
        tasks_created: count(&format!(
            "SELECT COUNT(*) FROM task WHERE deleted_at IS NULL \
             AND substr(created_at,1,10) >= '{week_start}'"
        ))?,
        entries_created: count(&format!(
            "SELECT COUNT(*) FROM entry WHERE deleted_at IS NULL AND date >= '{week_start}'"
        ))?,
        week_start,
    })
}
