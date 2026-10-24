use crate::models::Paper;
use crate::services::papers as svc;
use crate::services::papers::PaperInput;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_papers(state: State<'_, AppState>) -> Result<Vec<Paper>, String> {
    let conn = state.conn()?;
    svc::list(&conn)
}

#[tauri::command]
pub fn create_paper(state: State<'_, AppState>, input: PaperInput) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create(&conn, input)
}

#[tauri::command]
pub fn update_paper(state: State<'_, AppState>, id: i64, input: PaperInput) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update(&conn, id, input)
}

#[tauri::command]
pub fn delete_paper(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete(&conn, id)
}
