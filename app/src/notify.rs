//! Windows bildirimleri. Tıklanınca pencere öne gelir ve WhatsApp ilgili sohbeti açar.

use crate::{window, AppState};
use tauri::{AppHandle, Manager};
use whatsapprs_core::policy::should_show_toast;

/// Kurulu sürümde NSIS, Başlat menüsü kısayoluna bu AUMID'i atar.
const APP_ID: &str = "com.efeozbilgin.whatsapprs";

pub fn show(app: &AppHandle, id: u32, title: &str, body: &str) {
    let Some(w) = app.get_webview_window(window::LABEL) else { return };
    let visible = w.is_visible().unwrap_or(false);
    let focused = w.is_focused().unwrap_or(false);
    if !should_show_toast(visible, focused) {
        return;
    }
    let installed = app.state::<AppState>().installed;
    if let Err(e) = platform_show(app.clone(), installed, id, title, body) {
        log::warn!("bildirim gösterilemedi: {e}");
    }
}

#[cfg(windows)]
fn platform_show(app: AppHandle, installed: bool, id: u32, title: &str, body: &str) -> Result<(), String> {
    use tauri_winrt_notification::Toast;
    let app_id = if installed { APP_ID } else { Toast::POWERSHELL_APP_ID };
    Toast::new(app_id)
        .title(title)
        .text1(body)
        .on_activated(move |_| {
            window::show(&app);
            if let Some(w) = app.get_webview_window(window::LABEL) {
                let _ = w.eval(format!("window.__wrs && window.__wrs.click({id})"));
            }
            Ok(())
        })
        .show()
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn platform_show(_app: AppHandle, _installed: bool, _id: u32, title: &str, _body: &str) -> Result<(), String> {
    let _ = APP_ID;
    log::info!("bildirim (bu platformda gösterilmiyor): {title}");
    Ok(())
}
