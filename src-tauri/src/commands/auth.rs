use crate::services::settings;
use crate::state::AppState;
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::OsRng;
use std::sync::atomic::Ordering;
use tauri::State;

const KEY_PASSWORD_HASH: &str = "password_hash";

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// 是否已设置口令（false = 首次运行，unlock 将执行「创建」语义）
#[tauri::command]
pub fn has_password(state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(settings::get(&conn, KEY_PASSWORD_HASH)?.is_some())
}

/// 解锁；未设口令时首次调用即创建口令（前端配合「再输一次」确认）
#[tauri::command]
pub fn unlock(state: State<'_, AppState>, password: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match settings::get(&conn, KEY_PASSWORD_HASH)? {
        None => {
            let hash = hash_password(&password)?;
            settings::set(&conn, KEY_PASSWORD_HASH, &hash)?;
            state.unlocked.store(true, Ordering::SeqCst);
            Ok(true)
        }
        Some(hash) => {
            let ok = verify_password(&hash, &password);
            if ok {
                state.unlocked.store(true, Ordering::SeqCst);
            }
            Ok(ok)
        }
    }
}

#[tauri::command]
pub fn lock(state: State<'_, AppState>) -> Result<(), String> {
    state.unlocked.store(false, Ordering::SeqCst);
    Ok(())
}
