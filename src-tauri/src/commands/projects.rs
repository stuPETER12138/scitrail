use crate::models::Project;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, type, role, status, color, start_date, end_date, note
             FROM project
             WHERE deleted_at IS NULL
             ORDER BY status = '归档', id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], Project::from_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| e.to_string())?);
    }
    Ok(out)
}
