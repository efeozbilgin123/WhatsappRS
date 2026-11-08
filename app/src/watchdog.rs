//! Teşhis: uygulamanın ana iş parçacığı (pencere ve WebView2 olayları) takılırsa süresini log'a yazar.
//! Donma şikâyetlerinde sorunun bizim tarafta mı yoksa WebView2/WhatsApp'ta mı olduğunu ayırır.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::AppHandle;

const INTERVAL: Duration = Duration::from_secs(2);
const WARN_AFTER: Duration = Duration::from_millis(500);

pub fn start(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(INTERVAL);
        let done = Arc::new(AtomicBool::new(false));
        let done2 = done.clone();
        let sent = Instant::now();
        if app.run_on_main_thread(move || done2.store(true, Ordering::SeqCst)).is_err() {
            return; // uygulama kapanıyor
        }
        let mut warned = false;
        while !done.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(50));
            if !warned && sent.elapsed() > Duration::from_secs(5) {
                log::warn!("ana iş parçacığı 5 saniyedir yanıt vermiyor");
                warned = true;
            }
        }
        let waited = sent.elapsed();
        if waited > WARN_AFTER {
            log::warn!("ana iş parçacığı {} ms takıldı", waited.as_millis());
        }
    });
}
