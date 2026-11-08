//! Taşınabilir sürümün kendi exe'sini güncellemesi.
//! Windows çalışan exe'nin üzerine yazmaya izin vermez ama adını değiştirmeye izin verir:
//! eski exe `<ad>.exe.old` olur, yeni exe yerine konur; `.old` bir sonraki açılışta silinir.

use std::path::{Path, PathBuf};

fn with_suffix(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    exe.with_file_name(name)
}

pub fn old_path(exe: &Path) -> PathBuf {
    with_suffix(exe, ".old")
}

/// Yeni exe'yi yerine koyar. Herhangi bir adım başarısız olursa mevcut exe çalışır durumda kalır.
pub fn swap_exe(exe: &Path, new_bytes: &[u8]) -> std::io::Result<()> {
    if new_bytes.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "indirilen dosya boş"));
    }
    let new = with_suffix(exe, ".new");
    let old = old_path(exe);
    std::fs::write(&new, new_bytes)?;
    if old.exists() {
        if let Err(e) = std::fs::remove_file(&old) {
            let _ = std::fs::remove_file(&new);
            return Err(e);
        }
    }
    if let Err(e) = std::fs::rename(exe, &old) {
        let _ = std::fs::remove_file(&new);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&new, exe) {
        // geri al: eski exe yerine dönsün
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(&new);
        return Err(e);
    }
    Ok(())
}

/// Önceki güncellemeden kalan `.old` dosyasını siler (çalışırken silinemediği için açılışta).
pub fn cleanup_old(exe: &Path) {
    let _ = std::fs::remove_file(old_path(exe));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn tmp() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let d = std::env::temp_dir().join(format!("wrs-upd-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test] fn swaps_and_keeps_old() {
        let d = tmp();
        let exe = d.join("WhatsappRS.exe");
        std::fs::write(&exe, b"v1").unwrap();
        swap_exe(&exe, b"v2").unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"v2");
        assert_eq!(std::fs::read(old_path(&exe)).unwrap(), b"v1");
        assert!(!d.join("WhatsappRS.exe.new").exists());
    }
    #[test] fn second_update_replaces_stale_old() {
        let d = tmp();
        let exe = d.join("WhatsappRS.exe");
        std::fs::write(&exe, b"v2").unwrap();
        std::fs::write(old_path(&exe), b"v1").unwrap();
        swap_exe(&exe, b"v3").unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"v3");
        assert_eq!(std::fs::read(old_path(&exe)).unwrap(), b"v2");
    }
    #[test] fn failure_leaves_current_exe_untouched() {
        let d = tmp();
        let exe = d.join("WhatsappRS.exe");
        std::fs::write(&exe, b"v1").unwrap();
        // .new yolunda bir klasör varsa yazma başarısız olur
        std::fs::create_dir(d.join("WhatsappRS.exe.new")).unwrap();
        assert!(swap_exe(&exe, b"v2").is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"v1");
    }
    #[test] fn empty_download_rejected() {
        let d = tmp();
        let exe = d.join("WhatsappRS.exe");
        std::fs::write(&exe, b"v1").unwrap();
        assert!(swap_exe(&exe, b"").is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"v1");
    }
    #[test] fn cleanup_removes_old() {
        let d = tmp();
        let exe = d.join("WhatsappRS.exe");
        std::fs::write(old_path(&exe), b"v1").unwrap();
        cleanup_old(&exe);
        assert!(!old_path(&exe).exists());
        cleanup_old(&exe); // yoksa sorun çıkarmaz
    }
}
