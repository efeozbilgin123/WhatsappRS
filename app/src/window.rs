//! Ana pencere: WhatsApp Web'i yükler, kapatınca tepsiye gizlenir, konumunu hatırlar.

use crate::AppState;
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_opener::OpenerExt;
use whatsapprs_core::policy::{is_internal_url, WHATSAPP_URL};
use whatsapprs_core::settings::{self, geom_is_visible, Rect, WindowGeom};

pub const LABEL: &str = "main";

pub fn create(app: &AppHandle, start_hidden: bool) -> tauri::Result<WebviewWindow> {
    let state = app.state::<AppState>();
    let url = WHATSAPP_URL.parse().expect("sabit URL geçerli");
    let nav_app = app.clone();
    let new_app = app.clone();

    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(url))
        .title("WhatsappRS")
        .inner_size(1100.0, 750.0)
        .min_inner_size(600.0, 450.0)
        .visible(false)
        .data_directory(state.paths.webview.clone())
        .initialization_script(include_str!("../inject/bridge.js"))
        .on_navigation(move |url| {
            if is_internal_url(url.as_str()) {
                true
            } else {
                open_external(&nav_app, url.as_str());
                false
            }
        })
        .on_new_window(move |url, _features| {
            open_external(&new_app, url.as_str());
            NewWindowResponse::Deny
        })
        .build()?;

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
    let state = app.state::<AppState>();
    let mut s = state.settings.lock().unwrap();
    let maximized = w.is_maximized().unwrap_or(false);
    let minimized = w.is_minimized().unwrap_or(false);
    if minimized {
        return;
    }
    if maximized {
        // Büyütülmüşken normal boyutu bilinmez; yalnızca bayrağı güncelle.
        if let Some(g) = s.window.as_mut() {
            g.maximized = true;
        } else {
            return;
        }
    } else {
        let (Ok(pos), Ok(size)) = (w.outer_position(), w.inner_size()) else { return };
        s.window = Some(WindowGeom { x: pos.x, y: pos.y, width: size.width, height: size.height, maximized: false });
    }
    if let Err(e) = settings::save(&state.paths.settings, &s) {
        log::warn!("ayarlar kaydedilemedi: {e}");
    }
}
