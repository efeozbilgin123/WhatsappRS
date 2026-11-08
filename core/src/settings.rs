//! `data\settings.json` okuma/yazma ve pencere konumu doğrulama.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+W";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub shortcut: String,
    pub window: Option<WindowGeom>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { shortcut: DEFAULT_SHORTCUT.to_string(), window: None }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct WindowGeom { pub x: i32, pub y: i32, pub width: u32, pub height: u32, pub maximized: bool }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }

/// Ayarları okur. Dosya yoksa varsayılan; bozuksa `.bak` olarak saklar ve varsayılan döner
/// (ikinci değer bir uyarı mesajıdır).
pub fn load(path: &Path) -> (Settings, Option<String>) {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Settings::default(), None),
        Err(e) => return (Settings::default(), Some(format!("Ayar dosyası okunamadı: {e}"))),
    };
    match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(s) => (s, None),
        Err(e) => {
            let bak = free_backup_path(path);
            let note = match std::fs::rename(path, &bak) {
                Ok(()) => format!("Ayar dosyası bozuk ({e}); {} olarak saklandı, varsayılanlar kullanılıyor", bak.display()),
                Err(re) => format!("Ayar dosyası bozuk ({e}) ve yedeklenemedi ({re}); varsayılanlar kullanılıyor"),
            };
            (Settings::default(), Some(note))
        }
    }
}

/// İlk boş yedek adı: settings.json.bak, .bak1 … .bak9 (hepsi doluysa .bak9'un üzerine yazılır).
fn free_backup_path(path: &Path) -> std::path::PathBuf {
    let first = path.with_extension("json.bak");
    if !first.exists() {
        return first;
    }
    for i in 1..=9 {
        let p = path.with_extension(format!("json.bak{i}"));
        if !p.exists() || i == 9 {
            return p;
        }
    }
    unreachable!()
}

/// Önce geçici dosyaya yazar, sonra yerine taşır; yarım dosya kalmaz.
pub fn save(path: &Path, s: &Settings) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(s).map_err(std::io::Error::other)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)
}

/// Yalnızca pencere konumunu günceller; dosyadaki diğer alanlar (kullanıcının elle yaptığı
/// değişiklikler dahil) korunur. Dosya bozuksa dokunulmaz ve hata döner.
pub fn update_window(path: &Path, window: Option<WindowGeom>) -> std::io::Result<()> {
    let mut current = match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<Settings>(text.trim_start_matches('\u{feff}'))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Settings::default(),
        Err(e) => return Err(e),
    };
    current.window = window;
    save(path, &current)
}

const MIN_VISIBLE: i64 = 100;

/// Pencerenin en az 100×100 px'lik bir kısmı herhangi bir monitörde görünüyor mu?
pub fn geom_is_visible(g: &WindowGeom, monitors: &[Rect]) -> bool {
    let overlap = |a0: i64, a_len: u32, b0: i64, b_len: u32| {
        (a0 + a_len as i64).min(b0 + b_len as i64) - a0.max(b0)
    };
    monitors.iter().any(|m| {
        overlap(g.x as i64, g.width, m.x as i64, m.width) >= MIN_VISIBLE
            && overlap(g.y as i64, g.height, m.y as i64, m.height) >= MIN_VISIBLE
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn tmp() -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let d = std::env::temp_dir().join(format!("wrs-settings-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }
    const MON: Rect = Rect { x: 0, y: 0, width: 1920, height: 1080 };
    fn g(x: i32, y: i32) -> WindowGeom { WindowGeom { x, y, width: 800, height: 600, maximized: false } }

    #[test] fn missing_file_defaults() {
        let (s, w) = load(&tmp().join("settings.json"));
        assert_eq!(s, Settings::default());
        assert_eq!(s.shortcut, "Ctrl+Alt+W");
        assert!(w.is_none());
    }
    #[test] fn roundtrip() {
        let p = tmp().join("settings.json");
        let s = Settings { shortcut: "Alt+Q".into(), window: Some(g(10, 20)) };
        save(&p, &s).unwrap();
        assert_eq!(load(&p), (s, None));
    }
    #[test] fn missing_fields_default() {
        let p = tmp().join("settings.json");
        std::fs::write(&p, r#"{"window":null}"#).unwrap();
        assert_eq!(load(&p).0.shortcut, "Ctrl+Alt+W");
    }
    #[test] fn unknown_fields_ignored() {
        let p = tmp().join("settings.json");
        std::fs::write(&p, r#"{"shortcut":"Alt+Q","foo":1}"#).unwrap();
        let (s, w) = load(&p);
        assert_eq!(s.shortcut, "Alt+Q");
        assert!(w.is_none());
    }
    #[test] fn corrupt_file_backed_up() {
        let d = tmp();
        let p = d.join("settings.json");
        std::fs::write(&p, "{bozuk").unwrap();
        let (s, w) = load(&p);
        assert_eq!(s, Settings::default());
        assert!(w.is_some());
        assert_eq!(std::fs::read_to_string(d.join("settings.json.bak")).unwrap(), "{bozuk");
        assert!(!p.exists());
    }
    #[test] fn update_window_keeps_hand_edits() {
        let p = tmp().join("settings.json");
        save(&p, &Settings::default()).unwrap();
        // kullanıcı uygulama açıkken dosyayı elle düzenledi
        std::fs::write(&p, r#"{"shortcut":"Alt+Q"}"#).unwrap();
        update_window(&p, Some(g(10, 20))).unwrap();
        let (s, _) = load(&p);
        assert_eq!(s.shortcut, "Alt+Q");
        assert_eq!(s.window, Some(g(10, 20)));
    }
    #[test] fn update_window_leaves_corrupt_file_alone() {
        let p = tmp().join("settings.json");
        std::fs::write(&p, "{bozuk").unwrap();
        assert!(update_window(&p, Some(g(1, 2))).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "{bozuk");
    }
    #[test] fn bom_is_accepted() {
        let p = tmp().join("settings.json");
        std::fs::write(&p, "\u{feff}{\"shortcut\":\"Alt+Q\"}").unwrap();
        let (s, w) = load(&p);
        assert_eq!(s.shortcut, "Alt+Q");
        assert!(w.is_none());
    }
    #[test] fn repeated_corruption_keeps_old_backup() {
        let d = tmp();
        let p = d.join("settings.json");
        std::fs::write(&p, "{bir").unwrap();
        load(&p);
        std::fs::write(&p, "{iki").unwrap();
        load(&p);
        assert_eq!(std::fs::read_to_string(d.join("settings.json.bak")).unwrap(), "{bir");
        assert_eq!(std::fs::read_to_string(d.join("settings.json.bak1")).unwrap(), "{iki");
    }
    #[test] fn geom_visible() { assert!(geom_is_visible(&g(100, 100), &[MON])); }
    #[test] fn geom_offscreen_ignored() {
        assert!(!geom_is_visible(&g(5000, 5000), &[MON]));
        assert!(!geom_is_visible(&g(1870, 100), &[MON]));
        assert!(!geom_is_visible(&g(100, 100), &[]));
    }
}
