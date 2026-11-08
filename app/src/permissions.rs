//! Kamera/mikrofon/bildirim izinleri yalnızca https://web.whatsapp.com için verilir (aramalar için gerekli).

use tauri::WebviewWindow;

#[cfg(windows)]
pub fn install(window: &WebviewWindow) -> tauri::Result<()> {
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use webview2_com::{take_pwstr, PermissionRequestedEventHandler};
    use whatsapprs_core::policy::{allow_permission, PermKind};

    window.with_webview(|wv| unsafe {
        let result = (|| -> windows::core::Result<()> {
            let core = wv.controller().CoreWebView2()?;
            let handler = PermissionRequestedEventHandler::create(Box::new(|_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut uri = windows::core::PWSTR::null();
                args.Uri(&mut uri)?;
                let origin = take_pwstr(uri);
                let mut raw = COREWEBVIEW2_PERMISSION_KIND::default();
                args.PermissionKind(&mut raw)?;
                let kind = match raw {
                    COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => PermKind::Microphone,
                    COREWEBVIEW2_PERMISSION_KIND_CAMERA => PermKind::Camera,
                    COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => PermKind::Notifications,
                    _ => PermKind::Other,
                };
                let allow = allow_permission(&origin, kind);
                log::info!("izin {kind:?} ({}) {origin} -> {}", raw.0, if allow { "verildi" } else { "reddedildi" });
                args.SetState(if allow { COREWEBVIEW2_PERMISSION_STATE_ALLOW } else { COREWEBVIEW2_PERMISSION_STATE_DENY })?;
                Ok(())
            }));
            let mut token = 0i64;
            core.add_PermissionRequested(&handler, &mut token)?;
            Ok(())
        })();
        if let Err(e) = result {
            log::error!("izin işleyicisi kurulamadı: {e}");
        }
    })
}

#[cfg(not(windows))]
pub fn install(_window: &WebviewWindow) -> tauri::Result<()> {
    Ok(())
}
