//! Global kısayol (varsayılan Ctrl+Alt+W): pencereyi gösterir/gizler.

use crate::{notify, window, AppState};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use whatsapprs_core::settings::DEFAULT_SHORTCUT;

pub fn init(app: &AppHandle) {
    let configured = app.state::<AppState>().settings.lock().unwrap().shortcut.clone();
    let shortcut: Shortcut = match configured.parse() {
        Ok(s) => s,
        Err(e) => {
            log::warn!("kısayol '{configured}' anlaşılamadı ({e}); {DEFAULT_SHORTCUT} kullanılıyor");
            DEFAULT_SHORTCUT.parse().expect("varsayılan kısayol geçerli")
        }
    };
    let result = app.global_shortcut().on_shortcut(shortcut, |app, _s, event| {
        if event.state == ShortcutState::Pressed {
            window::toggle(app);
        }
    });
    if let Err(e) = result {
        log::warn!("kısayol {shortcut} kaydedilemedi: {e}");
        notify::info(
            app,
            &format!("{shortcut} kısayolu kullanılamıyor. settings.json dosyasından değiştirebilirsiniz."),
        );
    }
}
