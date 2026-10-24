use crate::models::{Achievement, GradRequirement};
use crate::services::achievements as svc;
use crate::services::achievements::{AchievementInput, RequirementInput};
use crate::state::AppState;
use tauri::State;

// ---------- 成果 ----------

#[tauri::command]
pub fn list_achievements(state: State<'_, AppState>) -> Result<Vec<Achievement>, String> {
    let conn = state.conn()?;
    svc::list_achievements(&conn)
}

#[tauri::command]
pub fn create_achievement(
    state: State<'_, AppState>,
    input: AchievementInput,
) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create_achievement(&conn, input)
}

#[tauri::command]
pub fn update_achievement(
    state: State<'_, AppState>,
    id: i64,
    input: AchievementInput,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update_achievement(&conn, id, input)
}

#[tauri::command]
pub fn delete_achievement(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete_achievement(&conn, id)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementExport {
    pub filename: String,
    pub content: String,
}

/// 个人成果清单 Markdown（导出走 export_file）
#[tauri::command]
pub fn achievements_markdown(state: State<'_, AppState>) -> Result<AchievementExport, String> {
    let conn = state.conn()?;
    let (filename, content) = svc::export_markdown(&conn)?;
    Ok(AchievementExport { filename, content })
}

// ---------- 毕业要求 ----------

#[tauri::command]
pub fn list_requirements(state: State<'_, AppState>) -> Result<Vec<GradRequirement>, String> {
    let conn = state.conn()?;
    svc::list_requirements(&conn)
}

#[tauri::command]
pub fn create_requirement(
    state: State<'_, AppState>,
    input: RequirementInput,
) -> Result<i64, String> {
    let conn = state.conn()?;
    svc::create_requirement(&conn, input)
}

#[tauri::command]
pub fn update_requirement(
    state: State<'_, AppState>,
    id: i64,
    input: RequirementInput,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::update_requirement(&conn, id, input)
}

#[tauri::command]
pub fn delete_requirement(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    svc::delete_requirement(&conn, id)
}
