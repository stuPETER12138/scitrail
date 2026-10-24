use crate::services::tasks::today_str;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryNew {
    pub date: Option<String>,
    pub time: Option<String>,
    pub kind: String,
    pub title: String,
    pub content: Option<String>,
    pub project_id: Option<i64>,
    pub tags: Option<Vec<String>>,
}

const KINDS: [&str; 5] = ["experiment", "literature", "expense", "meeting", "note"];

/// 最小创建入口（速录小窗用）；条目完整 CRUD 在 P3 轨迹模块
#[tauri::command]
pub fn create_entry(state: State<'_, AppState>, entry: EntryNew) -> Result<i64, String> {
    if !KINDS.contains(&entry.kind.as_str()) {
        return Err(format!("未知条目类型：{}", entry.kind));
    }
    let title = entry.title.trim();
    if title.is_empty() {
        return Err("标题不能为空".into());
    }
    let conn = state.conn()?;
    let tags = serde_json::to_string(&entry.tags.unwrap_or_default()).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO entry(date, time, kind, title, content, project_id, tags)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.date.clone().unwrap_or_else(today_str),
            entry.time,
            entry.kind,
            title,
            entry.content.unwrap_or_default(),
            entry.project_id,
            tags
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
