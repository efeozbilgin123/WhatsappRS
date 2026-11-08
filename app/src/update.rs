//! Otomatik güncelleme (GitHub Releases, imzalı).
//! - Kurulu sürüm: imzası doğrulanan kurulum programı sessiz modda çalışır (/UPDATE: oturum ve ayarlar korunur).
//! - Taşınabilir sürüm: imzası doğrulanan yeni exe, çalışan exe ile yer değiştirir ve uygulama yeniden başlar.

use crate::{notify, AppState};
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Taşınabilir sürümün latest.json içindeki platform anahtarı (kurulu sürüm varsayılanı kullanır).
const PORTABLE_TARGET: &str = "windows-x86_64-portable";
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

struct Pending {
    update: Update,
    bytes: Vec<u8>,
}

#[derive(Default)]
pub struct UpdateState {
    pending: Mutex<Option<Pending>>,
    busy: Mutex<bool>,
}

/// Tepsideki güncelleme öğesi (metni duruma göre değişir).
pub struct UpdateMenuItem(pub MenuItem<Wry>);

pub fn start(app: &AppHandle) {
    app.manage(UpdateState::default());
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        loop {
            check(&app, false);
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

/// Tepsi öğesine tıklanınca: bekleyen güncelleme varsa yükler, yoksa hemen denetler.
pub fn on_menu_click(app: &AppHandle) {
    let has_pending = app.state::<UpdateState>().pending.lock().unwrap().is_some();
    if has_pending {
        install(app);
    } else {
        let app = app.clone();
        std::thread::spawn(move || check(&app, true));
    }
}

fn check(app: &AppHandle, manual: bool) {
    let state = app.state::<UpdateState>();
    {
        let mut busy = state.busy.lock().unwrap();
        if *busy || state.pending.lock().unwrap().is_some() {
            return;
        }
        *busy = true;
    }
    let result = tauri::async_runtime::block_on(find_and_download(app));
    *state.busy.lock().unwrap() = false;
    match result {
        Ok(Some(p)) => {
            let version = p.update.version.clone();
            log::info!("güncelleme indirildi: {version}");
            *state.pending.lock().unwrap() = Some(p);
            if let Some(item) = app.try_state::<UpdateMenuItem>() {
                let _ = item.0.set_text(format!("Güncellemeyi yükle ({version})"));
            }
            notify::update_ready(app, &version);
        }
        Ok(None) => {
            log::info!("güncelleme yok");
            if manual {
                notify::info(app, "WhatsappRS güncel.");
            }
        }
        Err(e) => {
            log::warn!("güncelleme denetlenemedi: {e}");
            if manual {
                notify::info(app, "Güncellemeler denetlenemedi. İnternet bağlantınızı kontrol edin.");
            }
        }
    }
}

async fn find_and_download(app: &AppHandle) -> Result<Option<Pending>, String> {
    let installed = app.state::<AppState>().installed;
    let mut builder = app.updater_builder();
    if !installed {
        builder = builder.target(PORTABLE_TARGET);
    }
    let updater = builder.build().map_err(|e| e.to_string())?;
    let Some(update) = updater.check().await.map_err(|e| e.to_string())? else { return Ok(None) };
    // download() imzayı doğrular; imza tutmazsa hata döner ve hiçbir şey yüklenmez.
    let bytes = update.download(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
    Ok(Some(Pending { update, bytes }))
}

pub fn install(app: &AppHandle) {
    let Some(p) = app.state::<UpdateState>().pending.lock().unwrap().take() else { return };
    crate::window::save_geometry(app);
    log::info!("güncelleme yükleniyor: {}", p.update.version);
    if app.state::<AppState>().installed {
        // Kurulum programını sessiz modda başlatır ve bu süreci kapatır.
        if let Err(e) = p.update.install(&p.bytes) {
            log::error!("güncelleme yüklenemedi: {e}");
            notify::info(app, "Güncelleme yüklenemedi.");
        }
        return;
    }
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            log::error!("exe yolu bulunamadı: {e}");
            return;
        }
    };
    match whatsapprs_core::selfupdate::swap_exe(&exe, &p.bytes) {
        Ok(()) => app.restart(),
        Err(e) => {
            log::error!("taşınabilir güncelleme başarısız: {e}");
            notify::info(app, "Güncelleme yüklenemedi; klasöre yazma izni olmayabilir.");
        }
    }
}
