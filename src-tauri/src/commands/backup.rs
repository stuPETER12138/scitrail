use crate::services::backup as svc;
use crate::services::backup::BackupLog;
use crate::state::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn run_backup(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let conn = state.conn()?;
    let name = svc::run_backup(&conn, &dir.join("backups"))?;
    let path = dir.join("backups").join(&name);
    let _ = tauri_plugin_opener::reveal_item_in_dir(&path);
    Ok(name)
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupLog>, String> {
    let conn = state.conn()?;
    svc::list_backups(&conn)
}

/// 恢复：先快照当前状态 → checkpoint → 替换主库 → 重开连接（前端随后整页刷新）
#[tauri::command]
pub fn restore_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    file: String,
) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let backups = dir.join("backups");
    let src = backups.join(&file);
    if !src.exists() {
        return Err(format!("备份文件不存在：{file}"));
    }
    {
        let conn = state.conn()?;
        svc::prepare_restore(&conn, &backups)?;
    } // 连接释放后再动文件
    std::fs::copy(&src, dir.join("scitrail.db")).map_err(|e| e.to_string())?;
    let new_conn = crate::db::init(&dir.join("scitrail.db")).map_err(|e| e.to_string())?;
    *state.conn()? = new_conn;
    Ok(())
}

/// 全量导出（json|md）写入 exports 目录并在资源管理器中定位
#[tauri::command]
pub fn export_data(
    app: AppHandle,
    state: State<'_, AppState>,
    format: String,
) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let conn = state.conn()?;
    let path = svc::export(&conn, &dir.join("exports"), &format)?;
    let _ = tauri_plugin_opener::reveal_item_in_dir(&path);
    Ok(path)
}

/// 通用文件导出（学期总结等）：写 exports 并在资源管理器定位
#[tauri::command]
pub fn export_file(app: AppHandle, name: String, content: String) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = crate::services::backup::export_named(&dir.join("exports"), &name, &content)?;
    let _ = tauri_plugin_opener::reveal_item_in_dir(&path);
    Ok(path)
}
