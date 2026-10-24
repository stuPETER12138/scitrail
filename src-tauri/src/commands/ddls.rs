use crate::models::Ddl;
use crate::services::time as svc;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_ddls(
    state: State<'_, AppState>,
    active_only: Option<bool>,
) -> Result<Vec<Ddl>, String> {
    let conn = state.conn()?;
    svc::list_ddls(&conn, active_only.unwrap_or(true))
}

#[tauri::command]
pub fn create_ddl(state: State<'_, AppState>, input: svc::DdlInput) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create_ddl(&conn, input)
}

#[tauri::command]
pub fn update_ddl(state: State<'_, AppState>, id: i64, input: svc::DdlInput) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update_ddl(&conn, id, input)
}

#[tauri::command]
pub fn delete_ddl(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete_ddl(&conn, id)
}

#[tauri::command]
pub fn rollover_ddl(state: State<'_, AppState>, id: i64) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::rollover_ddl(&conn, id)
}
