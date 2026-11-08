//! Sistem tepsisi simgesi ve menüsü.

use crate::window;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::image::Image;
use tauri::AppHandle;

pub const ID: &str = "main";

pub fn icon_normal() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/tray.png")).expect("tepsi ikonu")
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    TrayIconBuilder::with_id(ID)
        .icon(icon_normal())
        .tooltip("WhatsappRS")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit" {
                window::save_geometry(app);
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
