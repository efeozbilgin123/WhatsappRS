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

/// Teşhis: sayfada 50 ms'den uzun süren işlerin 10 saniyelik özeti (bkz. bridge.js).
/// bridge.js alan adlarını snake_case gönderir (total_ms, max_ms).
#[tauri::command(rename_all = "snake_case")]
pub fn perf_report(count: u32, total_ms: u32, max_ms: u32, scrolling: bool, source: String) {
    let source: String = source.chars().take(100).collect();
    log::info!(
        "takılma: {count} kare, toplam {total_ms} ms, en uzun {max_ms} ms{}, kaynak: {source}",
        if scrolling { " (kaydırırken)" } else { "" }
    );
}
