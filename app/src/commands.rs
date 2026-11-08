//! WhatsApp Web sayfasının çağırabildiği iki komut (bkz. capabilities/remote.json).

use crate::{tray, AppState};
use tauri::{AppHandle, State};
use whatsapprs_core::unread;

#[tauri::command]
pub fn set_unread(app: AppHandle, state: State<'_, AppState>, title: String) {
    let count = unread::parse(&title);
    {
        let mut current = state.unread.lock().unwrap();
        if *current == count {
            return;
        }
        *current = count;
    }
    tray::update_unread(&app, count);
}

#[tauri::command]
pub fn notify(app: AppHandle, id: u32, title: String, body: String) {
    crate::notify::show(&app, id, &title, &body);
}
