use crate::models::Milestone;
use crate::services::time as svc;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_milestones(state: State<'_, AppState>) -> Result<Vec<Milestone>, String> {
    let conn = state.conn()?;
    svc::list_milestones(&conn)
}

#[tauri::command]
pub fn create_milestone(
    state: State<'_, AppState>,
    input: svc::MilestoneInput,
) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create_milestone(&conn, input)
}

#[tauri::command]
pub fn update_milestone(
    state: State<'_, AppState>,
    id: i64,
    input: svc::MilestoneInput,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update_milestone(&conn, id, input)
}

#[tauri::command]
pub fn delete_milestone(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete_milestone(&conn, id)
}
