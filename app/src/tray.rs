//! Sistem tepsisi simgesi, menüsü ve okunmamış rozeti.

use crate::{window, AppState};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const ID: &str = "main";
pub const AUTOSTART_ID: &str = "autostart";
pub const COMPAT_ID: &str = "compat";
pub const UPDATE_ID: &str = "update";
pub const WV_DEFAULTS_ID: &str = "webview-defaults";
pub const CLEAR_CACHE_ID: &str = "clear-cache";

fn icon_normal() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/tray.png")).expect("tepsi ikonu")
}

fn icon_badge(label: &str) -> Image<'static> {
    let bytes: &'static [u8] = match label {
        "1" => include_bytes!("../icons/tray-1.png"),
        "2" => include_bytes!("../icons/tray-2.png"),
        "3" => include_bytes!("../icons/tray-3.png"),
        "4" => include_bytes!("../icons/tray-4.png"),
        "5" => include_bytes!("../icons/tray-5.png"),
        "6" => include_bytes!("../icons/tray-6.png"),
        "7" => include_bytes!("../icons/tray-7.png"),
        "8" => include_bytes!("../icons/tray-8.png"),
        "9" => include_bytes!("../icons/tray-9.png"),
        _ => include_bytes!("../icons/tray-9plus.png"),
    };
    Image::from_bytes(bytes).expect("rozet ikonu")
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Göster / Gizle", true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", "Yeniden yükle", true, None::<&str>)?;
    let autostart_on = crate::autostart::is_enabled(app);
    let autostart = CheckMenuItem::with_id(app, AUTOSTART_ID, "Açılışta başlat", true, autostart_on, None::<&str>)?;
    let compat_on = app.state::<AppState>().settings.lock().unwrap().compat_mode;
    let compat = CheckMenuItem::with_id(app, COMPAT_ID, "Uyumluluk modu (sorun giderme)", true, compat_on, None::<&str>)?;
    let wv_on = app.state::<AppState>().settings.lock().unwrap().webview_defaults;
    let wv_defaults = CheckMenuItem::with_id(app, WV_DEFAULTS_ID, "Deney: WebView2 varsayılan ayarları", true, wv_on, None::<&str>)?;
    let clear_cache = MenuItem::with_id(app, CLEAR_CACHE_ID, "Önbelleği temizle (oturum korunur)", true, None::<&str>)?;
    let update = MenuItem::with_id(app, UPDATE_ID, "Güncellemeleri denetle", true, None::<&str>)?;
    app.manage(crate::update::UpdateMenuItem(update.clone()));
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&toggle, &reload, &sep1, &autostart, &compat, &wv_defaults, &clear_cache, &update, &sep2, &quit])?;

    let autostart_item = autostart.clone();
    TrayIconBuilder::with_id(ID)
        .icon(icon_normal())
        .tooltip("WhatsappRS")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            AUTOSTART_ID => {
                let want = !crate::autostart::is_enabled(app);
                if let Err(e) = crate::autostart::set(app, want) {
                    log::warn!("otomatik başlatma değiştirilemedi: {e}");
                    crate::notify::info(app, "Açılışta başlatma ayarı değiştirilemedi.");
                }
                let _ = autostart_item.set_checked(crate::autostart::is_enabled(app));
            }
            COMPAT_ID => toggle_setting(app, "uyumluluk modu", |s| &mut s.compat_mode, true),
            // WebView2 başlatma bayrakları değişince eski WebView2 süreci kapanmadan yenisi açılamaz:
            // yeniden başlatmak yerine çıkılır, kullanıcı birkaç saniye sonra açar.
            WV_DEFAULTS_ID => toggle_setting(app, "WebView2 varsayılan ayarları", |s| &mut s.webview_defaults, false),
            CLEAR_CACHE_ID => {
                let state = app.state::<AppState>();
                if let Err(e) = whatsapprs_core::settings::update(&state.paths.settings, |s| s.clear_cache_on_start = true) {
                    log::warn!("önbellek temizleme işaretlenemedi: {e}");
                    return;
                }
                log::info!("önbellek bir sonraki açılışta temizlenecek");
                window::save_geometry(app);
                crate::notify::info(app, "Uygulama kapanıyor. Birkaç saniye sonra yeniden açın; önbellek açılışta temizlenecek.");
                app.exit(0);
            }
            UPDATE_ID => crate::update::on_menu_click(app),
            "toggle" => window::toggle(app),
            "reload" => {
                if let Some(w) = app.get_webview_window(window::LABEL) {
                    let _ = w.eval("location.reload()");
                }
            }
            "quit" => {
                window::save_geometry(app);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Bir açık/kapalı ayarı değiştirir, kaydeder; sonra uygulamayı yeniden başlatır ya da kapatır.
fn toggle_setting(app: &AppHandle, name: &str, field: fn(&mut whatsapprs_core::settings::Settings) -> &mut bool, restart: bool) {
    let state = app.state::<AppState>();
    let on = {
        let mut s = state.settings.lock().unwrap();
        let f = field(&mut s);
        *f = !*f;
        *f
    };
    if let Err(e) = whatsapprs_core::settings::update(&state.paths.settings, |s| *field(s) = on) {
        log::warn!("{name} kaydedilemedi: {e}");
        return;
    }
    log::info!("{name} {}", if on { "açıldı" } else { "kapatıldı" });
    window::save_geometry(app);
    if restart {
        app.restart();
    } else {
        crate::notify::info(app, "Ayar kaydedildi. Uygulama kapanıyor; birkaç saniye sonra yeniden açın.");
        app.exit(0);
    }
}

fn icon_overlay(label: &str) -> Image<'static> {
    let bytes: &'static [u8] = match label {
        "1" => include_bytes!("../icons/overlay-1.png"),
        "2" => include_bytes!("../icons/overlay-2.png"),
        "3" => include_bytes!("../icons/overlay-3.png"),
        "4" => include_bytes!("../icons/overlay-4.png"),
        "5" => include_bytes!("../icons/overlay-5.png"),
        "6" => include_bytes!("../icons/overlay-6.png"),
        "7" => include_bytes!("../icons/overlay-7.png"),
        "8" => include_bytes!("../icons/overlay-8.png"),
        "9" => include_bytes!("../icons/overlay-9.png"),
        _ => include_bytes!("../icons/overlay-9plus.png"),
    };
    Image::from_bytes(bytes).expect("kaplama ikonu")
}

pub fn update_unread(app: &AppHandle, count: u32) {
    let label = whatsapprs_core::unread::badge_label(count);
    let (icon, tooltip) = match label {
        Some(l) => (icon_badge(l), format!("WhatsappRS — {count} okunmamış")),
        None => (icon_normal(), "WhatsappRS".to_string()),
    };
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_tooltip(Some(tooltip));
    }
    // Görev çubuğu: logo değişmez, Windows köşeye yalnızca kırmızı sayıyı koyar.
    if let Some(w) = app.get_webview_window(window::LABEL) {
        let _ = w.set_overlay_icon(label.map(icon_overlay));
    }
}
