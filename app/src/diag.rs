//! Teşhis: açılıştan sonra WebView2 sürümünü ve sayfanın hangi grafik sürücüsüyle çizildiğini log'a yazar.
//! "SwiftShader" / "Basic Render" görülürse ekran kartı hızlandırması kapalıdır (kaydırma donmasının olası nedeni).
//! Sayfanın kendisine dokunmaz; uyumluluk modunda da çalışır.

use tauri::WebviewWindow;

const PROBE: &str = r#"(() => {
  let renderer = "webgl yok";
  try {
    const gl = document.createElement("canvas").getContext("webgl");
    if (gl) {
      const ext = gl.getExtension("WEBGL_debug_renderer_info");
      renderer = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER);
    }
  } catch (e) { renderer = "hata: " + e; }
  return { renderer, dpr: devicePixelRatio, w: innerWidth, h: innerHeight, cores: navigator.hardwareConcurrency, ram_gb: navigator.deviceMemory };
})()"#;

#[cfg(windows)]
pub fn start(window: &WebviewWindow) {
    let window = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(15));
        let _ = window.with_webview(|wv| unsafe {
            use webview2_com::ExecuteScriptCompletedHandler;
            use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment;
            let env: ICoreWebView2Environment = wv.environment();
            let mut version = windows::core::PWSTR::null();
            if env.BrowserVersionString(&mut version).is_ok() {
                log::info!("WebView2 sürümü: {}", webview2_com::take_pwstr(version));
            }
            let Ok(core) = wv.controller().CoreWebView2() else { return };
            let handler = ExecuteScriptCompletedHandler::create(Box::new(|_err, result| {
                log::info!("grafik: {result}");
                Ok(())
            }));
            let js = windows::core::HSTRING::from(PROBE);
            if let Err(e) = core.ExecuteScript(&js, &handler) {
                log::warn!("teşhis betiği çalıştırılamadı: {e}");
            }
        });
    });
}

#[cfg(not(windows))]
pub fn start(_window: &WebviewWindow) {
    let _ = PROBE;
}
