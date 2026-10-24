use crate::services::weekly as svc;
use crate::services::weekly::WeeklyReport;
use crate::state::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn generate_weekly(
    state: State<'_, AppState>,
    week_start: Option<String>,
) -> Result<String, String> {
    let conn = state.conn()?;
    let ws = svc::monday_of(week_start.as_deref())?;
    svc::generate(&conn, &ws)
}

#[tauri::command]
pub fn list_weeklies(state: State<'_, AppState>) -> Result<Vec<WeeklyReport>, String> {
    let conn = state.conn()?;
    svc::list(&conn)
}

#[tauri::command]
pub fn save_weekly(
    state: State<'_, AppState>,
    week_start: String,
    content: String,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::save(&conn, &week_start, &content)
}

/// 导出为 .md 文件并在资源管理器中定位
#[tauri::command]
pub fn export_weekly(
    app: AppHandle,
    state: State<'_, AppState>,
    week_start: String,
) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let conn = state.conn()?;
    let path = crate::services::backup::export_weekly(&conn, &dir.join("exports"), &week_start)?;
    let _ = tauri_plugin_opener::reveal_item_in_dir(&path);
    Ok(path)
}
