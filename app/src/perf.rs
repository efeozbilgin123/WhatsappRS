//! Teşhis + deney: WebView2 süreçlerinin Windows güç kısıtlaması (EcoQoS / "verimlilik modu") durumunu
//! log'a yazar; "yüksek performans" açıksa kısıtlamayı kaldırmasını Windows'tan ister.
//! Hibrit işlemcilerde kısıtlanan süreçler verimli (yavaş) çekirdeklere itilebilir.

use tauri::WebviewWindow;

#[cfg(windows)]
pub fn start(window: &WebviewWindow, high_performance: bool) {
    let window = window.clone();
    std::thread::spawn(move || {
        let mut last = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(if last.is_empty() { 20 } else { 60 }));
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = window.with_webview(move |wv| {
                let _ = tx.send(unsafe { process_ids(&wv) });
            });
            let Ok(pids) = rx.recv_timeout(std::time::Duration::from_secs(5)) else { continue };
            let report: Vec<String> = pids.iter().map(|(pid, kind)| unsafe { handle(*pid, kind, high_performance) }).collect();
            let report = report.join(", ");
            if report != last {
                log::info!("WebView2 süreçleri{}: {report}", if high_performance { " (yüksek performans açık)" } else { "" });
                last = report;
            }
        }
    });
}

#[cfg(windows)]
unsafe fn process_ids(wv: &tauri::webview::PlatformWebview) -> Vec<(u32, String)> {
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use windows::core::Interface;
    let mut out = Vec::new();
    let Ok(env8) = wv.environment().cast::<ICoreWebView2Environment8>() else { return out };
    let Ok(infos) = env8.GetProcessInfos() else { return out };
    let mut count = 0u32;
    let _ = infos.Count(&mut count);
    for i in 0..count {
        let Ok(info) = infos.GetValueAtIndex(i) else { continue };
        let (mut pid, mut kind) = (0i32, COREWEBVIEW2_PROCESS_KIND::default());
        let _ = info.ProcessId(&mut pid);
        let _ = info.Kind(&mut kind);
        let name = match kind {
            COREWEBVIEW2_PROCESS_KIND_BROWSER => "tarayıcı",
            COREWEBVIEW2_PROCESS_KIND_RENDERER => "sayfa",
            COREWEBVIEW2_PROCESS_KIND_GPU => "gpu",
            COREWEBVIEW2_PROCESS_KIND_UTILITY => "yardımcı",
            _ => "diğer",
        };
        out.push((pid as u32, name.to_string()));
    }
    out
}

/// Sürecin güç kısıtlaması durumunu okur; istenirse kaldırır. "sayfa:kısıtlı" / "sayfa:normal" döner.
#[cfg(windows)]
unsafe fn handle(pid: u32, kind: &str, high_performance: bool) -> String {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::*;
    let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION;
    let Ok(h) = OpenProcess(access, false, pid) else { return format!("{kind}:erişilemedi") };
    let mut state = PROCESS_POWER_THROTTLING_STATE { Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION, ..Default::default() };
    let size = std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32;
    let throttled = GetProcessInformation(h, ProcessPowerThrottling, &mut state as *mut _ as *mut _, size).is_ok()
        && state.ControlMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED != 0
        && state.StateMask & PROCESS_POWER_THROTTLING_EXECUTION_SPEED != 0;
    let mut result = format!("{kind}:{}", if throttled { "kısıtlı" } else { "normal" });
    if high_performance {
        // ControlMask = hız kısıtlamasını biz yönetiyoruz, StateMask = 0 → kısıtlama kapalı
        let off = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: 0,
        };
        if SetProcessInformation(h, ProcessPowerThrottling, &off as *const _ as *const _, size).is_err() {
            result.push_str("(kaldırılamadı)");
        }
    }
    let _ = CloseHandle(h);
    result
}

#[cfg(not(windows))]
pub fn start(_window: &WebviewWindow, _high_performance: bool) {}
