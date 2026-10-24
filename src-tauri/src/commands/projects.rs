use crate::models::Project;
use crate::services::tasks::now_str;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.conn()?;
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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub color: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub note: Option<String>,
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, input: ProjectInput) -> Result<i64, String> {
    let name = input.name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err("课题名称不能为空".into());
    }
    let conn = state.conn()?;
    conn.execute(
        "INSERT INTO project(name, type, role, status, color, start_date, end_date, note)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            name,
            input.r#type.unwrap_or_else(|| "自选".into()),
            input.role,
            input.status.unwrap_or_else(|| "进行中".into()),
            input.color,
            input.start_date,
            input.end_date,
            input.note
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

/// 全量编辑（名称必填）
#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    id: i64,
    input: ProjectInput,
) -> Result<(), String> {
    let name = input.name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err("课题名称不能为空".into());
    }
    let conn = state.conn()?;
    let changed = conn
        .execute(
            "UPDATE project SET name=?2, type=COALESCE(?3, type), role=?4, status=COALESCE(?5, status),
                 color=?6, start_date=?7, end_date=?8, note=?9, updated_at=?10
             WHERE id=?1 AND deleted_at IS NULL",
            params![
                id,
                name,
                input.r#type,
                input.role,
                input.status,
                input.color,
                input.start_date,
                input.end_date,
                input.note,
                now_str()
            ],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("课题不存在".into());
    }
    Ok(())
}

/// 软删除；其下任务自动移回收件箱（project_id 置空）
#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn()?;
    conn.execute(
        "UPDATE project SET deleted_at=?2, updated_at=?2 WHERE id=?1 AND deleted_at IS NULL",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE task SET project_id=NULL, updated_at=?2 WHERE project_id=?1",
        params![id, now_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
