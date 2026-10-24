mod commands;
mod db;
mod migration;
mod models;
mod quick;
mod services;
mod state;
mod tray;

use state::AppState;
use std::sync::{Mutex, atomic::AtomicBool};
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            quick::show_main(app);
        }))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        quick::toggle_quick(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let conn = db::init(&dir.join("scitrail.db")).map_err(|e| e.to_string())?;
            app.manage(AppState {
                db: Mutex::new(conn),
                unlocked: AtomicBool::new(false),
            });

            // §6.4：全局速录快捷键。说明书原文默认 Ctrl+K，但全局注册会抢占
            // 浏览器等应用的 Ctrl+K，故实现取低冲突的 Ctrl+Alt+K，将来设置页可改。
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
            let shortcut: Shortcut = "Ctrl+Alt+K"
                .parse()
                .map_err(|e| format!("快捷键解析失败: {e}"))?;
            if let Err(e) = app.global_shortcut().register(shortcut) {
                log::warn!("注册全局快捷键失败（可能被占用）: {e}");
            }

            tray::setup(app)?;
            quick::create_quick(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关闭 = 最小化到托盘；托盘菜单「退出」才真正退出
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    "main" | "quick" => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::has_password,
            commands::auth::unlock,
            commands::auth::lock,
            commands::meta::app_info,
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::complete_task,
            commands::tasks::delete_task,
            commands::quick::parse_quick,
            commands::entries::create_entry,
            commands::stats::stats_week,
            commands::ddls::list_ddls,
            commands::ddls::create_ddl,
            commands::ddls::update_ddl,
            commands::ddls::delete_ddl,
            commands::ddls::rollover_ddl,
            commands::milestones::list_milestones,
            commands::milestones::create_milestone,
            commands::milestones::update_milestone,
            commands::milestones::delete_milestone,
            commands::entries::list_entries,
            commands::entries::update_entry,
            commands::entries::delete_entry,
            commands::entries::entry_history,
            commands::entries::today_entry_count,
            commands::trajectory::get_trajectory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
