//! Ana pencere: WhatsApp Web'i yükler, kapatınca tepsiye gizlenir, konumunu hatırlar.

use crate::AppState;
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::window::Color;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_opener::OpenerExt;
use whatsapprs_core::policy::{is_internal_url, new_window_action, NewWindowAction, WHATSAPP_URL};
use whatsapprs_core::settings::{self, geom_is_visible, Rect, WindowGeom};

pub const LABEL: &str = "main";

pub fn create(app: &AppHandle, start_hidden: bool) -> tauri::Result<WebviewWindow> {
    let state = app.state::<AppState>();
    let url = WHATSAPP_URL.parse().expect("sabit URL geçerli");
    let nav_app = app.clone();
    let new_app = app.clone();

    let compat_mode = state.settings.lock().unwrap().compat_mode;
    if compat_mode {
        log::warn!("uyumluluk modu açık: sayfaya betik eklenmiyor (rozet ve bildirimler çalışmaz)");
    }
    let mut builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(url));
    if !compat_mode {
        builder = builder.initialization_script(include_str!("../inject/bridge.js"));
    }
    let window = builder
        .title("WhatsappRS")
        .inner_size(1100.0, 750.0)
        .min_inner_size(600.0, 450.0)
        .visible(false)
        .data_directory(state.paths.webview.clone())
        // Koyu tema: WhatsApp "sistem varsayılanı" temasında koyu açılır, açılışta beyaz parlama olmaz.
        .theme(Some(Theme::Dark))
        .background_color(Color(crate::titlebar::BACKGROUND.0, crate::titlebar::BACKGROUND.1, crate::titlebar::BACKGROUND.2, 255))
        // Tauri'nin dosya sürükle-bırak yakalayıcısı kapalı: dosyalar sohbete sürüklenebilsin.
        .disable_drag_drop_handler()
        .zoom_hotkeys_enabled(true)
        .on_navigation(move |url| {
            if is_internal_url(url.as_str()) {
                log::info!("gezinme: {url}");
                true
            } else {
                open_external(&nav_app, url.as_str());
                false
            }
        })
        .on_new_window(move |url, _features| match new_window_action(url.as_str()) {
            NewWindowAction::OpenInApp => NewWindowResponse::Allow,
            NewWindowAction::OpenExternal => {
                open_external(&new_app, url.as_str());
                NewWindowResponse::Deny
            }
            NewWindowAction::Deny => {
                log::warn!("yeni pencere reddedildi: {url}");
                NewWindowResponse::Deny
            }
        })
        .on_download(|webview, event| {
            match event {
                DownloadEvent::Requested { url, destination } => {
                    log::info!("indirme başladı: {} -> {}", url.scheme(), destination.display());
                }
                DownloadEvent::Finished { path, success, .. } => {
                    crate::notify::download_finished(webview.app_handle(), path.as_deref(), success);
                }
                _ => {}
            }
            true
        })
        .build()?;

    crate::permissions::install(&window)?;
    crate::titlebar::apply(&window);
    restore_geometry(app, &window);

    let ev_app = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            save_geometry(&ev_app);
            if let Some(w) = ev_app.get_webview_window(LABEL) {
                let _ = w.hide();
            }
        }
    });

    if !start_hidden {
        window.show()?;
        let _ = window.set_focus();
    }
    Ok(window)
}

fn open_external(app: &AppHandle, url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        log::warn!("dış bağlantı reddedildi: {url}");
        return;
    }
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        log::warn!("bağlantı açılamadı {url}: {e}");
    }
}

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn toggle(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let visible = w.is_visible().unwrap_or(false);
        let minimized = w.is_minimized().unwrap_or(false);
        if visible && !minimized {
            save_geometry(app);
            let _ = w.hide();
        } else {
            show(app);
        }
    }
}

fn restore_geometry(app: &AppHandle, window: &WebviewWindow) {
    let Some(g) = app.state::<AppState>().settings.lock().unwrap().window else { return };
    let monitors: Vec<Rect> = app
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| Rect { x: m.position().x, y: m.position().y, width: m.size().width, height: m.size().height })
        .collect();
    if !geom_is_visible(&g, &monitors) {
        log::info!("kayıtlı pencere konumu ekran dışında, varsayılan kullanılıyor");
        return;
    }
    let _ = window.set_size(PhysicalSize::new(g.width, g.height));
    let _ = window.set_position(PhysicalPosition::new(g.x, g.y));
    if g.maximized {
        let _ = window.maximize();
    }
}

pub fn save_geometry(app: &AppHandle) {
    let Some(w) = app.get_webview_window(LABEL) else { return };
    if w.is_minimized().unwrap_or(false) {
        return;
    }
    let state = app.state::<AppState>();
    let mut s = state.settings.lock().unwrap();
    if w.is_maximized().unwrap_or(false) {
        // Büyütülmüşken normal boyutu bilinmez; yalnızca bayrağı güncelle.
        match s.window.as_mut() {
            Some(g) => g.maximized = true,
            None => {
                // Hiç normal boyut kaydedilmemiş: geçerli boyutu bayrakla birlikte sakla.
                let (Ok(pos), Ok(size)) = (w.outer_position(), w.inner_size()) else { return };
                s.window = Some(WindowGeom { x: pos.x, y: pos.y, width: size.width, height: size.height, maximized: true });
            }
        }
    } else {
        let (Ok(pos), Ok(size)) = (w.outer_position(), w.inner_size()) else { return };
        s.window = Some(WindowGeom { x: pos.x, y: pos.y, width: size.width, height: size.height, maximized: false });
    }
    // Yalnızca pencere alanını yaz: kullanıcının dosyada elle yaptığı değişiklikler korunur.
    if let Err(e) = settings::update_window(&state.paths.settings, s.window) {
        log::warn!("pencere konumu kaydedilemedi: {e}");
    }
}
