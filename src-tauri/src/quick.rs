use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// 速录小窗：独立无边框置顶窗口，初始隐藏，由全局快捷键/托盘唤起（§6.4）
pub fn create_quick(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("quick").is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html#/quick".into()))
        .title("速录")
        .inner_size(420.0, 170.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .center()
        .build()?;
    Ok(())
}

pub fn toggle_quick(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quick") {
        match w.is_visible() {
            Ok(true) => {
                let _ = w.hide();
            }
            _ => {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    } else if create_quick(app).is_ok()
        && let Some(w) = app.get_webview_window("quick")
    {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
