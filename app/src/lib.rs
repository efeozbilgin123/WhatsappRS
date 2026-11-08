//! WhatsappRS: WhatsApp Web için hafif Tauri kabuğu.

mod autostart;
mod commands;
mod fatal;
mod logging;
mod notify;
mod permissions;
mod shortcuts;
mod titlebar;
mod tray;
mod window;

use std::sync::Mutex;
use whatsapprs_core::paths::{is_installed, DataPaths};
use whatsapprs_core::policy::show_on_second_instance;
use whatsapprs_core::settings::{self, Settings};

pub struct AppState {
    pub paths: DataPaths,
    pub settings: Mutex<Settings>,
    pub unread: Mutex<u32>,
    pub installed: bool,
}

pub fn run() {
    let exe = std::env::current_exe().unwrap_or_else(|e| fatal::fatal(&format!("Uygulama konumu bulunamadı: {e}")));
    let exe_dir = exe.parent().expect("exe bir klasörde").to_path_buf();
    let paths = DataPaths::for_exe_dir(&exe_dir);
    if let Err(e) = paths.prepare() {
        fatal::fatal(&e.to_string());
    }
    logging::init(&paths.logs);
    log::info!("WhatsappRS {} başlıyor ({})", env!("CARGO_PKG_VERSION"), exe_dir.display());

    let (loaded, warning) = settings::load(&paths.settings);
    if let Some(w) = warning {
        log::warn!("{w}");
    }
    let start_hidden = std::env::args().any(|a| a == "--minimized");
    let state = AppState {
        paths,
        settings: Mutex::new(loaded),
        unread: Mutex::new(0),
        installed: is_installed(&exe_dir),
    };

    let result = tauri::Builder::default()
        // single-instance ilk eklenti olmalı
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if show_on_second_instance(&args) {
                window::show(app);
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![commands::set_unread, commands::notify])
        .setup(move |app| {
            let handle = app.handle();
            tray::init(handle)?;
            window::create(handle, start_hidden)?;
            shortcuts::init(handle);
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        log::error!("uygulama hatası: {e}");
        fatal::fatal(
            "WhatsappRS başlatılamadı. Microsoft Edge WebView2 eksik olabilir: \
             https://developer.microsoft.com/microsoft-edge/webview2 adresinden kurun.",
        );
    }
}
