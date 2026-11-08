//! Başlık çubuğunu WhatsApp'ın koyu arayüzüyle aynı renge boyar (Windows 11).
//! Windows 10'da renk API'si yok; orada pencere teması koyu olur (Theme::Dark).

use tauri::WebviewWindow;

/// WhatsApp Web koyu tema arka planı: #161717
pub const BACKGROUND: (u8, u8, u8) = (0x16, 0x17, 0x17);
const TEXT: (u8, u8, u8) = (0xe9, 0xed, 0xef);

#[cfg(windows)]
pub fn apply(window: &WebviewWindow) {
    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
    };
    let Ok(hwnd) = window.hwnd() else { return };
    let colorref = |(r, g, b): (u8, u8, u8)| COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16);
    for (attr, color) in [
        (DWMWA_CAPTION_COLOR, colorref(BACKGROUND)),
        (DWMWA_BORDER_COLOR, colorref(BACKGROUND)),
        (DWMWA_TEXT_COLOR, colorref(TEXT)),
    ] {
        let r = unsafe {
            DwmSetWindowAttribute(hwnd, attr, &color as *const COLORREF as *const _, std::mem::size_of::<COLORREF>() as u32)
        };
        if let Err(e) = r {
            log::info!("başlık rengi uygulanamadı ({attr:?}): {e} — Windows 10'da beklenen durum");
            break;
        }
    }
}

#[cfg(not(windows))]
pub fn apply(_window: &WebviewWindow) {}
