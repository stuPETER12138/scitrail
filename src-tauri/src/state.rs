use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard, atomic::AtomicBool};

/// 全局应用状态：单连接（单用户本机场景足够）+ 解锁标记
pub struct AppState {
    pub db: Mutex<Connection>,
    pub unlocked: AtomicBool,
}

impl AppState {
    pub fn conn(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.db.lock().map_err(|e| e.to_string())
    }
}
