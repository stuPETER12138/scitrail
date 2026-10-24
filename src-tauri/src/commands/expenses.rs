use crate::services::expenses as svc;
use crate::services::expenses::ExpenseLedger;
use crate::state::AppState;
use tauri::State;

/// 报销垫付台账：条目列表 + 汇总（垫付中/已提交/已到账）+ 按课题分布
#[tauri::command]
pub fn expense_ledger(state: State<'_, AppState>) -> Result<ExpenseLedger, String> {
    let conn = state.conn()?;
    svc::ledger(&conn)
}

/// 状态流转：垫付中 → 已提交 → 已到账（保留历史快照）
#[tauri::command]
pub fn set_expense_status(
    state: State<'_, AppState>,
    id: i64,
    status: String,
) -> Result<(), String> {
    let conn = state.conn()?;
    svc::set_status(&conn, id, &status)
}
