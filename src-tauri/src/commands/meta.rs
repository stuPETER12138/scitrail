use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub db_path: String,
    pub schema_version: i64,
}

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> Result<AppInfo, String> {
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("scitrail.db");
    let schema_version = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        conn.query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?
    };
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        db_path: db_path.to_string_lossy().into_owned(),
        schema_version,
    })
}
