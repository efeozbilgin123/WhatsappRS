//! Sistem tepsisi simgesi, menüsü ve okunmamış rozeti.

use crate::{window, AppState};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const ID: &str = "main";
pub const AUTOSTART_ID: &str = "autostart";
pub const PREPARE_UNINSTALL_ID: &str = "prepare-uninstall";

fn icon_normal() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/tray.png")).expect("tepsi ikonu")
}

fn icon_unread() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/tray-unread.png")).expect("tepsi ikonu")
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let installed = app.state::<AppState>().installed;
    let toggle = MenuItem::with_id(app, "toggle", "Göster / Gizle", true, None::<&str>)?;
    let reload = MenuItem::with_id(app, "reload", "Yeniden yükle", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(app, AUTOSTART_ID, "Açılışta başlat", false, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&toggle, &reload, &sep1, &autostart])?;
    if !installed {
        let prepare = MenuItem::with_id(app, PREPARE_UNINSTALL_ID, "Kaldırmaya hazırla", false, None::<&str>)?;
        menu.append(&prepare)?;
    }
    menu.append_items(&[&sep2, &quit])?;

    TrayIconBuilder::with_id(ID)
        .icon(icon_normal())
        .tooltip("WhatsappRS")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
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

pub fn update_unread(app: &AppHandle, count: u32) {
    let (icon, tooltip) = if count > 0 {
        (icon_unread(), format!("WhatsappRS — {count} okunmamış"))
    } else {
        (icon_normal(), "WhatsappRS".to_string())
    };
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_icon(Some(icon.clone()));
        let _ = tray.set_tooltip(Some(tooltip));
    }
    if let Some(w) = app.get_webview_window(window::LABEL) {
        let _ = w.set_overlay_icon(if count > 0 { Some(icon) } else { None });
    }
}
