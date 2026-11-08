//! Windows açılışında başlatma: yalnızca HKCU\...\Run → "WhatsappRS" = "<exe>" --minimized.
//! (Hazır eklenti yerine kendi kodumuz: HKLM'ye hiç yazmaz ve yolu tırnak içine alır.)

use tauri::AppHandle;

pub const RUN_VALUE_NAME: &str = "WhatsappRS";

#[cfg(windows)]
mod imp {
    use super::RUN_VALUE_NAME;
    use whatsapprs_core::policy::autostart_command;
    use windows_registry::CURRENT_USER;

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    pub fn is_enabled() -> Result<bool, String> {
        match CURRENT_USER.options().read().open(RUN) {
            Ok(key) => Ok(key.get_string(RUN_VALUE_NAME).is_ok()),
            Err(_) => Ok(false),
        }
    }

    pub fn set(on: bool) -> Result<(), String> {
        let key = CURRENT_USER.create(RUN).map_err(|e| e.to_string())?;
        if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            key.set_string(RUN_VALUE_NAME, autostart_command(&exe.display().to_string()))
                .map_err(|e| e.to_string())
        } else {
            let _ = key.remove_value(RUN_VALUE_NAME);
            // Görev Yöneticisi'nden açılıp kapatıldıysa Windows bu değeri oluşturur; iz bırakmayalım.
            if let Ok(approved) = CURRENT_USER.options().read().write().open(APPROVED) {
                let _ = approved.remove_value(RUN_VALUE_NAME);
            }
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn is_enabled() -> Result<bool, String> {
        Ok(false)
    }
    pub fn set(_on: bool) -> Result<(), String> {
        Err("bu platformda desteklenmiyor".into())
    }
}

pub fn is_enabled(_app: &AppHandle) -> bool {
    imp::is_enabled().unwrap_or_else(|e| {
        log::warn!("otomatik başlatma durumu okunamadı: {e}");
        false
    })
}

pub fn set(_app: &AppHandle, on: bool) -> Result<(), String> {
    imp::set(on)
}
