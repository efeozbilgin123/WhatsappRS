//! Windows açılışında başlatma (HKCU\...\Run → "WhatsappRS", --minimized ile).

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

pub const RUN_VALUE_NAME: &str = "WhatsappRS";

pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_autostart::Builder::new()
        .app_name(RUN_VALUE_NAME)
        .args(["--minimized"])
        .build()
}

pub fn is_enabled(app: &AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or_else(|e| {
        log::warn!("otomatik başlatma durumu okunamadı: {e}");
        false
    })
}

pub fn set(app: &AppHandle, on: bool) -> Result<(), String> {
    let mgr = app.autolaunch();
    let r = if on { mgr.enable() } else { mgr.disable() };
    r.map_err(|e| e.to_string())
}
