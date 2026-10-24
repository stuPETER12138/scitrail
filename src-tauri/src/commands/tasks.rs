use crate::models::Task;
use crate::services::tasks as svc;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_tasks(
    state: State<'_, AppState>,
    view: Option<String>,
    project_id: Option<i64>,
) -> Result<Vec<Task>, String> {
    let conn = state.conn()?;
    svc::list(
        &conn,
        view.as_deref().unwrap_or("active"),
        project_id,
        &svc::today_str(),
    )
}

#[tauri::command]
pub fn create_task(state: State<'_, AppState>, task: svc::TaskNew) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create(&conn, task)
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    id: i64,
    task: svc::TaskPatch,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update(&conn, id, task)
}

#[tauri::command]
pub fn complete_task(state: State<'_, AppState>, id: i64) -> Result<Option<i64>, String> {
    let conn = state.conn()?;
    svc::complete(&conn, id)
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete(&conn, id)
}
