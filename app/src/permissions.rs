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
                    COREWEBVIEW2_PERMISSION_KIND_MULTIPLE_AUTOMATIC_DOWNLOADS => PermKind::MultipleDownloads,
                    _ => PermKind::Other,
                };
                let allow = allow_permission(&origin, kind);
                log::info!("izin {kind:?} ({}) {origin} -> {}", raw.0, if allow { "verildi" } else { "reddedildi" });
                args.SetState(if allow { COREWEBVIEW2_PERMISSION_STATE_ALLOW } else { COREWEBVIEW2_PERMISSION_STATE_DENY })?;
                Ok(())
            }));
            let mut token = 0i64;
            core.add_PermissionRequested(&handler, &mut token)?;

            // Teşhis: WebView2 süreçleri çökerse log'a yaz; sayfa süreci öldüyse yeniden yükle.
            let failed = webview2_com::ProcessFailedEventHandler::create(Box::new(|sender, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                args.ProcessFailedKind(&mut kind)?;
                log::error!("WebView2 süreci çöktü (tür {})", kind.0);
                if kind == COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED {
                    if let Some(core) = sender {
                        log::warn!("sayfa yeniden yükleniyor");
                        core.Reload()?;
                    }
                }
                Ok(())
            }));
            let mut token2 = 0i64;
            core.add_ProcessFailed(&failed, &mut token2)?;
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
