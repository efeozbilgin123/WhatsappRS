//! Pencere gizliyken WebView2'den daha az bellek kullanmasını ister (MemoryUsageTargetLevel = Low).
//! Sayfa askıya alınmaz: mesajlar, rozet ve bildirimler çalışmaya devam eder.

use tauri::WebviewWindow;

#[cfg(windows)]
pub fn set_low(window: &WebviewWindow, low: bool) {
    let _ = window.with_webview(move |wv| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::*;
        use windows::core::Interface;
        let Ok(core) = wv.controller().CoreWebView2() else { return };
        let Ok(core19) = core.cast::<ICoreWebView2_19>() else {
            log::info!("bellek hedefi desteklenmiyor (eski WebView2)");
            return;
        };
        let level = if low { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW } else { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL };
        if let Err(e) = core19.SetMemoryUsageTargetLevel(level) {
            log::warn!("bellek hedefi ayarlanamadı: {e}");
        }
    });
}

#[cfg(not(windows))]
pub fn set_low(_window: &WebviewWindow, _low: bool) {}
