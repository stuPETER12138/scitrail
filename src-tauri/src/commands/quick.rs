use crate::services::quick_parse::{self, ParsedDraft};
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

/// 速录解析预览：返回结构化草稿；#课题 在库中模糊匹配（精确 → 包含，取最短名）
#[tauri::command]
pub fn parse_quick(
    state: State<'_, AppState>,
    text: String,
    default_kind: Option<String>,
) -> Result<ParsedDraft, String> {
    let mut draft = quick_parse::parse(&text, default_kind.as_deref().unwrap_or("task"));
    if let Some(name) = draft.project_name.clone() {
        let conn = state.conn()?;
        let exact: Option<i64> = conn
            .query_row(
                "SELECT id FROM project WHERE deleted_at IS NULL AND name = ?1 LIMIT 1",
                params![name],
                |r| r.get(0),
            )
            .ok();
        let fuzzy: Option<i64> = conn
            .query_row(
                "SELECT id FROM project WHERE deleted_at IS NULL AND name LIKE ?1
                 ORDER BY LENGTH(name) LIMIT 1",
                params![format!("%{name}%")],
                |r| r.get(0),
            )
            .ok();
        draft.project_matched = exact.or(fuzzy);
    }
    Ok(draft)
}
