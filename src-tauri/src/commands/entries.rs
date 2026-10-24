use crate::models::Entry;
use crate::services::entries as svc;
use crate::services::entries::EntryInput;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_entries(
    state: State<'_, AppState>,
    from: Option<String>,
    to: Option<String>,
    kind: Option<String>,
    project_id: Option<i64>,
    q: Option<String>,
) -> Result<Vec<Entry>, String> {
    let conn = state.conn()?;
    svc::list(&conn, from, to, kind, project_id, q)
}

/// 速录小窗与轨迹页 composer 共用的创建入口
#[tauri::command]
pub fn create_entry(state: State<'_, AppState>, entry: EntryInput) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create(&conn, entry)
}

#[tauri::command]
pub fn update_entry(state: State<'_, AppState>, id: i64, entry: EntryInput) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update(&conn, id, entry)
}

#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete(&conn, id)
}

#[tauri::command]
pub fn entry_history(state: State<'_, AppState>, id: i64) -> Result<Vec<svc::HistoryItem>, String> {
    let conn = state.conn()?;
    svc::history(&conn, id)
}

/// 今日条目数（今日页摘要条用）
#[tauri::command]
pub fn today_entry_count(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::today_count(&conn)
}
