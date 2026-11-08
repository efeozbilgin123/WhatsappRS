//! Tüm veriler exe'nin yanındaki `data\` klasöründe tutulur.

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct DataPaths {
    pub root: PathBuf,
    pub settings: PathBuf,
    pub logs: PathBuf,
    pub webview: PathBuf,
}

#[derive(Debug)]
pub enum PathError {
    NotWritable { path: PathBuf, source: std::io::Error },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::NotWritable { path, source } => write!(
                f,
                "Veri klasörüne yazılamıyor: {}. Uygulamayı yazma izniniz olan bir klasöre taşıyın. ({source})",
                path.display()
            ),
        }
    }
}

impl std::error::Error for PathError {}

impl DataPaths {
    pub fn for_exe_dir(exe_dir: &Path) -> DataPaths {
        let root = exe_dir.join("data");
        DataPaths {
            settings: root.join("settings.json"),
            logs: root.join("logs"),
            webview: root.join("webview"),
            root,
        }
    }

    /// Klasörleri oluşturur ve gerçekten yazılabildiğini bir deneme dosyasıyla doğrular.
    pub fn prepare(&self) -> Result<(), PathError> {
        let fail = |path: &Path| { let path = path.to_path_buf(); move |source| PathError::NotWritable { path, source } };
        for dir in [&self.root, &self.logs, &self.webview] {
            std::fs::create_dir_all(dir).map_err(fail(dir))?;
        }
        let probe = self.root.join(".write-test");
        std::fs::write(&probe, b"ok").map_err(fail(&self.root))?;
        std::fs::remove_file(&probe).map_err(fail(&self.root))?;
        Ok(())
    }
}

/// NSIS kurulumu `uninstall.exe` dosyasını kurulum klasörüne koyar.
pub fn is_installed(exe_dir: &Path) -> bool {
    exe_dir.join("uninstall.exe").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn tmp() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let d = std::env::temp_dir().join(format!("wrs-paths-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test] fn layout() {
        let p = DataPaths::for_exe_dir(Path::new("/x"));
        assert_eq!(p.root, PathBuf::from("/x/data"));
        assert_eq!(p.settings, PathBuf::from("/x/data/settings.json"));
        assert_eq!(p.logs, PathBuf::from("/x/data/logs"));
        assert_eq!(p.webview, PathBuf::from("/x/data/webview"));
    }
    #[test] fn prepare_creates_dirs() {
        let d = tmp();
        let p = DataPaths::for_exe_dir(&d);
        p.prepare().unwrap();
        assert!(p.root.is_dir() && p.logs.is_dir() && p.webview.is_dir());
        assert!(!p.root.join(".write-test").exists());
    }
    #[test] fn prepare_fails_when_unwritable() {
        let d = tmp();
        let file = d.join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();
        let err = DataPaths::for_exe_dir(&file).prepare().unwrap_err();
        assert!(matches!(err, PathError::NotWritable { .. }));
        assert!(err.to_string().contains("Veri klasörüne yazılamıyor"));
    }
    #[test] fn installed_detection() {
        let d = tmp();
        assert!(!is_installed(&d));
        std::fs::write(d.join("uninstall.exe"), b"").unwrap();
        assert!(is_installed(&d));
    }
}
