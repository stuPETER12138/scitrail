use crate::services::trajectory as svc;
use crate::services::trajectory::TrajectoryDay;
use crate::state::AppState;
use tauri::State;

/// 轨迹投影：from/to 为 'YYYY-MM-DD'，返回按日升序分组（旧→新，账本锚定今天）
#[tauri::command]
pub fn get_trajectory(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<TrajectoryDay>, String> {
    let conn = state.conn()?;
    svc::get(&conn, &from, &to)
}
