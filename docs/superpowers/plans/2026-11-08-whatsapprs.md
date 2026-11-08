# WhatsappRS Uygulama Planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** WhatsApp Web'i Tauri v2 kabuğunda çalıştıran, tepsi/bildirim/kısayol/otomatik başlatma ekleyen, tek klasörde yaşayan Windows uygulaması + kurulum programı + taşınabilir zip.

**Architecture:** Cargo workspace iki crate'ten oluşur: `core/` (Tauri'siz saf mantık: başlık ayrıştırma, izin/gezinme kararları, yollar, ayarlar — Linux'ta test edilir) ve `app/` (Tauri v2 uygulaması; pencere, tepsi, bildirim, kısayol, WebView2 izinleri). `app/inject/bridge.js` sayfaya enjekte edilir ve yalnızca iki Rust komutunu çağırır. Windows derlemesi ve kurulum paketi GitHub Actions'ta yapılır.

**Tech Stack:** Rust 1.97, tauri 2.12 (tauri-build 2.7, tauri-cli 2.12), tauri-plugin-single-instance 2.5, tauri-plugin-global-shortcut 2.4, tauri-plugin-autostart 2.7, tauri-plugin-opener 2.7, webview2-com 0.39 + windows 0.62 (Tauri ile aynı sürüm olmalı), tauri-winrt-notification 0.8, flexi_logger, serde/serde_json, url, NSIS.

**Spec:** `docs/superpowers/specs/2026-11-08-whatsapprs-design.md`

## Global Constraints

- Ürün adı / exe adı: `WhatsappRS` / `WhatsappRS.exe`; Tauri identifier: `com.efeozbilgin.whatsapprs`.
- Yüklenen tek site: `https://web.whatsapp.com`. Resmi olmayan WhatsApp protokol kütüphanesi kullanılmaz.
- **Uygulama hiçbir şeyi exe klasörü dışına yazmaz** (tek istisna: otomatik başlatma için `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` değeri `WhatsappRS`). Bu yüzden `tauri-plugin-window-state` ve varsayılan Tauri app-data/log dizinleri **kullanılmaz**; WebView2 veri klasörü açıkça `data\webview` yapılır.
- Veri düzeni: `<exe klasörü>\data\{settings.json, logs\, webview\}`; loglar dönüşümlü, toplam ≤ 5 MB.
- Uzak sayfaya açık komutlar yalnızca `set_unread` ve `notify` (capability `remote.urls = ["https://web.whatsapp.com"]`).
- Varsayılan global kısayol `Ctrl+Alt+W`; `settings.json`'daki `shortcut` alanıyla değiştirilebilir.
- Otomatik başlatma argümanı `--minimized`.
- Kurulum: NSIS, `currentUser`, varsayılan `%LOCALAPPDATA%\Programs\WhatsappRS`, yönetici izni yok.
- Commit'ler: yazar `efe <79136287+efeozbilgin123@users.noreply.github.com>` (depoda ayarlı), **Co-Authored-By satırı yok**, mesajlar Türkçe. Çalışma dalı: `ilk-surum`.
- Kullanıcıya görünen metinler Türkçe.

## Review Focus

1. **Okunmamışlar okununca rozet sıfırlanmalı:** başlık `(2) WhatsApp` → `WhatsApp` olunca sayı 0'a döner (Task 1 testi `parse_back_to_zero`).
2. **Pencere odaktayken bildirim spam'i olmamalı:** görünür + odakta → toast yok; gizli veya odak dışı → toast var (Task 2 testi `toast_policy`).
3. **Otomatik başlatma ikinci kopyayı tetiklerse pencere aniden açılmamalı:** ikinci kopya argümanlarında `--minimized` varsa pencere gösterilmez (Task 2 testi `second_instance_minimized`).
4. **Monitör söküldükten sonra pencere ekran dışında açılmamalı:** kayıtlı konum hiçbir monitörle kesişmiyorsa varsayılan konum kullanılır (Task 4 testi `geom_offscreen_ignored`).
5. **Elle düzenlenip bozulan ayar dosyası uygulamayı açılmaz hale getirmemeli:** bozuk JSON → `.bak` + varsayılan; eksik alan → o alanın varsayılanı (Task 4 testleri `corrupt_file_backed_up`, `missing_fields_default`).

---

### Task 1: Workspace iskeleti + okunmamış sayısı ayrıştırma

**Files:**
- Create: `Cargo.toml` (workspace: `members = ["core", "app"]`, `resolver = "2"`; `app` Task 5'te eklenir, şimdilik yalnız `core`)
- Create: `core/Cargo.toml` (paket `whatsapprs-core`, lib; bağımlılıklar: `serde` (derive), `serde_json`, `url`)
- Create: `core/src/lib.rs` (`pub mod unread;`)
- Create: `core/src/unread.rs`
- Create: `.gitignore` (`/target`, `app/gen/`)

**Interfaces:**
- Produces: `whatsapprs_core::unread::parse(title: &str) -> u32`

- [ ] **Step 1: Dalı oluştur:** `git checkout -b ilk-surum`
- [ ] **Step 2: Failing testleri yaz** (`core/src/unread.rs` içinde `#[cfg(test)]`):

```rust
#[test] fn parse_count() { assert_eq!(parse("(3) WhatsApp"), 3); }
#[test] fn parse_no_count() { assert_eq!(parse("WhatsApp"), 0); }
#[test] fn parse_plus() { assert_eq!(parse("(99+) WhatsApp"), 99); }
#[test] fn parse_garbage() {
    for t in ["", "(", "()", "(abc) WhatsApp", "WhatsApp (3)", "(-1) WhatsApp"] { assert_eq!(parse(t), 0, "{t}"); }
}
#[test] fn parse_overflow() { assert_eq!(parse("(99999999999) WhatsApp"), 0); }
#[test] fn parse_back_to_zero() { assert_eq!(parse("(2) WhatsApp"), 2); assert_eq!(parse("WhatsApp"), 0); }
#[test] fn parse_leading_space() { assert_eq!(parse("  (4) WhatsApp"), 4); }
```

- [ ] **Step 3: Çalıştır, FAIL gör:** `cargo test -p whatsapprs-core` → derleme hatası (`parse` yok)
- [ ] **Step 4: `pub fn parse(title: &str) -> u32` yaz.** Kural: baştaki boşluk kırpılır; `(` ile başlamalı; ardından 1+ ASCII rakam, isteğe bağlı tek `+`, sonra `)`; rakamlar `u32`'ye sığmazsa 0. Diğer her durum 0.
- [ ] **Step 5: Çalıştır, PASS gör:** `cargo test -p whatsapprs-core` → 7 test geçer
- [ ] **Step 6: Commit:** `git add -A && git commit -m "core: workspace ve okunmamış sayısı ayrıştırma"`

### Task 2: İzin, gezinme ve bildirim kararları

**Files:**
- Create: `core/src/policy.rs`; Modify: `core/src/lib.rs` (`pub mod policy;`)

**Interfaces:**
- Produces:
  - `pub enum PermKind { Microphone, Camera, Notifications, Other }`
  - `policy::allow_permission(origin: &str, kind: PermKind) -> bool`
  - `policy::is_internal_url(url: &str) -> bool`
  - `policy::should_show_toast(window_visible: bool, window_focused: bool) -> bool`
  - `policy::show_on_second_instance(args: &[String]) -> bool`
  - `pub const WHATSAPP_URL: &str = "https://web.whatsapp.com/";`

- [ ] **Step 1: Failing testleri yaz:**

```rust
#[test] fn perm_allows_whatsapp_media() {
    for k in [PermKind::Microphone, PermKind::Camera, PermKind::Notifications] {
        assert!(allow_permission("https://web.whatsapp.com", k));
        assert!(allow_permission("https://web.whatsapp.com/", k));
    }
}
#[test] fn perm_denies_other_kinds() { assert!(!allow_permission("https://web.whatsapp.com", PermKind::Other)); }
#[test] fn perm_denies_lookalikes() {
    for o in ["http://web.whatsapp.com", "https://web.whatsapp.com.evil.com", "https://evil.com/web.whatsapp.com",
              "https://web.whatsapp.com:8443", "https://xweb.whatsapp.com", "https://whatsapp.com", "", "nonsense"] {
        assert!(!allow_permission(o, PermKind::Camera), "{o}");
    }
}
#[test] fn internal_urls() {
    assert!(is_internal_url("https://web.whatsapp.com/"));
    assert!(is_internal_url("https://web.whatsapp.com/send?phone=1"));
    assert!(is_internal_url("about:blank"));
    assert!(!is_internal_url("https://example.com/"));
    assert!(!is_internal_url("http://web.whatsapp.com/"));
    assert!(!is_internal_url("https://web.whatsapp.com.evil.com/"));
}
#[test] fn toast_policy() {
    assert!(!should_show_toast(true, true));
    assert!(should_show_toast(true, false));
    assert!(should_show_toast(false, false));
    assert!(should_show_toast(false, true));
}
#[test] fn second_instance_minimized() {
    assert!(!show_on_second_instance(&["WhatsappRS.exe".into(), "--minimized".into()]));
    assert!(show_on_second_instance(&["WhatsappRS.exe".into()]));
}
```

- [ ] **Step 2: Çalıştır, FAIL gör:** `cargo test -p whatsapprs-core policy`
- [ ] **Step 3: Uygula.** `url::Url::parse` ile: şema tam `https`, host tam `web.whatsapp.com`, `port()` `None`. `is_internal_url` aynı kontrol + `about:blank`. `should_show_toast = !(visible && focused)`.
- [ ] **Step 4: Çalıştır, PASS gör:** `cargo test -p whatsapprs-core`
- [ ] **Step 5: Commit:** `git commit -am "core: izin, gezinme ve bildirim kararları"` (yeni dosya için önce `git add core/src/policy.rs`)

### Task 3: Veri yolları ve kurulum algılama

**Files:**
- Create: `core/src/paths.rs`; Modify: `core/src/lib.rs`

**Interfaces:**
- Produces:
  - `pub struct DataPaths { pub root: PathBuf, pub settings: PathBuf, pub logs: PathBuf, pub webview: PathBuf }`
  - `paths::DataPaths::for_exe_dir(exe_dir: &Path) -> DataPaths` (root = `exe_dir/data`, diğerleri `settings.json`, `logs`, `webview`)
  - `paths::DataPaths::prepare(&self) -> Result<(), PathError>` — üç dizini oluşturur, `root/.write-test` yazıp siler
  - `pub enum PathError { NotWritable { path: PathBuf, source: std::io::Error } }` + `Display` (Türkçe: `"Veri klasörüne yazılamıyor: {path}. Uygulamayı yazma izniniz olan bir klasöre taşıyın."`)
  - `paths::is_installed(exe_dir: &Path) -> bool` — `exe_dir/uninstall.exe` varsa `true` (NSIS bu dosyayı kurulum klasörüne koyar)

- [ ] **Step 1: Failing testleri yaz** (geçici dizin için `std::env::temp_dir()` altında benzersiz klasör; dev-dependency ekleme):
  - `layout`: `for_exe_dir("/x")` → `root == "/x/data"`, `settings == "/x/data/settings.json"`, `logs == "/x/data/logs"`, `webview == "/x/data/webview"`
  - `prepare_creates_dirs`: `prepare()` `Ok`, üç dizin var, `.write-test` yok
  - `prepare_fails_when_unwritable`: `exe_dir` olarak **normal bir dosyanın** yolunu ver (testler root olarak koşabildiği için izin bitleri kullanılmaz) → `Err(PathError::NotWritable{..})`, mesaj `"Veri klasörüne yazılamıyor"` içerir
  - `installed_detection`: `uninstall.exe` yokken `false`, boş dosya oluşturunca `true`
- [ ] **Step 2: FAIL gör:** `cargo test -p whatsapprs-core paths`
- [ ] **Step 3: Uygula** (`std::fs::create_dir_all`, `fs::write`, `fs::remove_file`; ilk hatada `NotWritable` döner).
- [ ] **Step 4: PASS gör:** `cargo test -p whatsapprs-core`
- [ ] **Step 5: Commit:** `"core: veri yolları ve kurulum algılama"`

### Task 4: Ayarlar ve pencere konumu doğrulama

**Files:**
- Create: `core/src/settings.rs`; Modify: `core/src/lib.rs`

**Interfaces:**
- Produces:
  - ```rust
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(default)]
    pub struct Settings { pub shortcut: String, pub window: Option<WindowGeom> }  // Default: shortcut "Ctrl+Alt+W", window None
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
    pub struct WindowGeom { pub x: i32, pub y: i32, pub width: u32, pub height: u32, pub maximized: bool }
    pub struct Rect { pub x: i32, pub y: i32, pub width: u32, pub height: u32 }
    ```
  - `settings::load(path: &Path) -> (Settings, Option<String>)` — dosya yoksa varsayılan + `None`; bozuksa dosyayı `settings.json.bak` olarak yeniden adlandırır, varsayılan + `Some(uyarı)` döner
  - `settings::save(path: &Path, s: &Settings) -> std::io::Result<()>` — önce `settings.json.tmp`'ye yazar, sonra `rename` (yarım yazılmış dosya kalmaz)
  - `settings::geom_is_visible(g: &WindowGeom, monitors: &[Rect]) -> bool` — pencerenin en az 100×100 px'lik bir kısmı bir monitörle kesişiyorsa `true`
- Not: otomatik başlatma durumu ayarlarda tutulmaz; doğruluk kaynağı kayıt defteridir (Task 9).

- [ ] **Step 1: Failing testler:**
  - `missing_file_defaults`: olmayan yol → `(Settings::default(), None)`, `shortcut == "Ctrl+Alt+W"`
  - `roundtrip`: `save` sonra `load` → eşit
  - `missing_fields_default`: dosya içeriği `{"window":null}` → `shortcut == "Ctrl+Alt+W"`
  - `unknown_fields_ignored`: `{"shortcut":"Alt+Q","foo":1}` → `shortcut == "Alt+Q"`, uyarı yok
  - `corrupt_file_backed_up`: içerik `{bozuk` → varsayılan + `Some(_)`, `settings.json.bak` içeriği `{bozuk`, `settings.json` artık yok
  - `geom_visible`: monitör `{0,0,1920,1080}`, pencere `{100,100,800,600}` → `true`
  - `geom_offscreen_ignored`: pencere `{5000,5000,800,600}` → `false`; pencere `{1870,100,800,600}` (yalnız 50 px içeride) → `false`; monitör listesi boş → `false`
- [ ] **Step 2: FAIL gör:** `cargo test -p whatsapprs-core settings`
- [ ] **Step 3: Uygula.**
- [ ] **Step 4: PASS gör:** `cargo test -p whatsapprs-core` (tüm core testleri)
- [ ] **Step 5: Commit:** `"core: ayarlar ve pencere konumu doğrulama"`

### Task 5: Tauri uygulama iskeleti, pencere, loglama + Windows CI

**Files:**
- Modify: `Cargo.toml` (members'a `app`)
- Create: `app/Cargo.toml` — paket `whatsapprs`, `[[bin]] name = "WhatsappRS"`; bağımlılıklar: `whatsapprs-core = { path = "../core" }`, `tauri = { version = "2.12", features = ["tray-icon", "image-png"] }`, `tauri-plugin-single-instance = "2.5"`, `tauri-plugin-opener = "2.7"`, `log`, `flexi_logger`; `[target.'cfg(windows)'.dependencies] windows = { version = "0.62", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }`; build-dep `tauri-build = "2.7"`
- Create: `app/build.rs` — `tauri_build::try_build(Attributes::new().app_manifest(AppManifest::new().commands(&["set_unread", "notify"])))`
- Create: `app/tauri.conf.json` — `productName "WhatsappRS"`, `mainBinaryName "WhatsappRS"`, `identifier "com.efeozbilgin.whatsapprs"`, `version "0.1.0"`, `build.frontendDist "ui"`, `app.windows []` (pencere kodla açılır), `app.withGlobalTauri false`, `bundle.active true`, `bundle.targets ["nsis"]`, `bundle.icon` (oluşturulan ikonlar)
- Create: `app/ui/index.html` (boş yer tutucu; kullanılmaz)
- Create: `app/capabilities/remote.json` — `{"identifier":"remote-whatsapp","windows":["main"],"remote":{"urls":["https://web.whatsapp.com"]},"permissions":["allow-set-unread","allow-notify"]}`
- Create: `app/icons/` — `assets/icon-source.png` (1024×1024, yeşil daire üzerinde beyaz "RS"; Python/Pillow ile üretilip commit edilir) → `cargo tauri icon assets/icon-source.png -o app/icons`; ayrıca `app/icons/tray.png` ve `app/icons/tray-unread.png` (32×32; ikincisi sağ üstte kırmızı nokta)
- Create: `app/src/main.rs` — `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`, `fn main() { whatsapprs::run() }`
- Create: `app/src/lib.rs` — `run()`: yollar → loglama → ayarlar → eklentiler → pencere
- Create: `app/src/fatal.rs` — `pub fn fatal(msg: &str) -> !` : Windows'ta `MessageBoxW` (başlık "WhatsappRS"), diğerlerinde `eprintln!`; sonra `std::process::exit(1)`
- Create: `app/src/logging.rs` — `pub fn init(logs_dir: &Path)` : flexi_logger, dosya `whatsapprs.log`, `Criterion::Size(1_000_000)`, `Naming::Numbers`, `Cleanup::KeepLogFiles(4)`, seviye `info`
- Create: `app/src/window.rs`
- Create: `.github/workflows/build.yml`

**Interfaces:**
- Consumes: `DataPaths`, `settings::{load, save, geom_is_visible}`, `policy::{is_internal_url, show_on_second_instance, WHATSAPP_URL}`
- Produces:
  - `pub struct AppState { pub paths: DataPaths, pub settings: Mutex<Settings>, pub unread: Mutex<u32>, pub installed: bool }` (`app.manage` ile)
  - `window::create(app: &AppHandle, start_hidden: bool) -> tauri::Result<WebviewWindow>` — etiket `"main"`
  - `window::toggle(app: &AppHandle)`, `window::show(app: &AppHandle)` (göster + `unminimize` + `set_focus`)
  - `window::save_geometry(app: &AppHandle)`

- [ ] **Step 1: `window::create` uygula:** `WebviewWindowBuilder::new(app, "main", WebviewUrl::External(WHATSAPP_URL.parse()?))` + `.title("WhatsappRS")`, `.inner_size(1100., 750.)`, `.min_inner_size(600., 450.)`, `.visible(!start_hidden)`, `.data_directory(paths.webview.clone())`, `.on_navigation(|url| { if is_internal_url(url.as_str()) { true } else { opener ile aç; false } })`. Kayıtlı `WindowGeom` `geom_is_visible` (monitörler `app.available_monitors()`) ise konum/boyut/maximize uygula. `on_window_event`: `CloseRequested` → `api.prevent_close()`, `save_geometry`, `hide()`. Yeni pencere (target=_blank) istekleri: Tauri 2.12'deki `on_new_window` ile URL'i opener'a ver ve reddet; bu API derlenmezse Task 6'da `bridge.js` içinde `window.open` ve `a[target=_blank]` tıklamaları yakalanıp aynı amaçla üçüncü bir `open_external(url)` komutu eklenir (komut yalnız `https?://` kabul eder; capability ve `build.rs` listesi güncellenir).
- [ ] **Step 2: `run()` uygula:** `exe_dir = current_exe().parent()`; `DataPaths::for_exe_dir(exe_dir).prepare()` hata → `fatal(&err.to_string())`; `logging::init`; `settings::load` uyarısı varsa `log::warn!`; `start_hidden = args.contains("--minimized")`; `tauri::Builder::default()` + `.plugin(tauri_plugin_single_instance::init(|app, args, _| if show_on_second_instance(&args) { window::show(app) }))` (**ilk eklenen eklenti olmalı**) + `.plugin(tauri_plugin_opener::init())` + `.setup(|app| { window::create(...)?; Ok(()) })`. `run()` hatası (ör. WebView2 yok) → `fatal("WebView2 bulunamadı. https://developer.microsoft.com/microsoft-edge/webview2 adresinden kurun.")`. Çıkışta `save_geometry`.
- [ ] **Step 3: Geçici tepsi:** bu görevde yalnızca `Çıkış` öğeli bir tepsi menüsü ve sol tıkta `toggle` (aksi halde gizlenen pencereye dönülemez). Task 6 genişletir.
- [ ] **Step 4: Yerel kontrol:** `rustup target add x86_64-pc-windows-msvc && cargo check -p whatsapprs --target x86_64-pc-windows-msvc`. Beklenen: hata yok. `build.rs` kaynak derleyici (rc) bulamazsa `apt-get install -y llvm` ile `llvm-rc` kur ve tekrar dene; yine olmazsa bu adımı CI'ya bırak ve not düş.
- [ ] **Step 5: CI yaz** (`.github/workflows/build.yml`): tetikleyiciler `push` (tüm dallar), `pull_request`, etiket `v*`. İş 1 `test` (`ubuntu-latest`): `cargo test -p whatsapprs-core`. İş 2 `windows` (`windows-latest`): `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `cargo test --workspace`, `cargo install tauri-cli --version "^2.12" --locked`, `cargo tauri build` (`app/` içinde), çıktıları `actions/upload-artifact@v4` ile yükle (`target/release/bundle/nsis/*.exe`). Zip ve Release Task 10'da.
- [ ] **Step 6: Commit + push:** `git add -A && git commit -m "app: Tauri iskeleti, pencere, loglama ve Windows CI" && git push -u origin ilk-surum`
- [ ] **Step 7: CI doğrula:** `gh run watch` → iki iş de yeşil; artifact'te `WhatsappRS_0.1.0_x64-setup.exe` var. Kırmızıysa log'u oku, düzelt, tekrar push.

### Task 6: bridge.js + okunmamış rozeti + tam tepsi menüsü

**Files:**
- Create: `app/inject/bridge.js`
- Create: `app/src/commands.rs` (`set_unread`, Task 7'de `notify` eklenir)
- Create: `app/src/tray.rs`
- Modify: `app/src/window.rs` (`.initialization_script(include_str!("../inject/bridge.js"))`), `app/src/lib.rs` (`invoke_handler`, tray kurulumu)

**Interfaces:**
- Consumes: `unread::parse`, `AppState.unread`, `window::{toggle, show}`
- Produces:
  - `#[tauri::command] fn set_unread(app: AppHandle, state: State<AppState>, title: String)`
  - `tray::init(app: &AppHandle) -> tauri::Result<()>` (tepsi kimliği `"main"`)
  - `tray::update_unread(app: &AppHandle, count: u32)`
  - JS: `window.__wrs` nesnesi (Task 7 `click` ekler); `invoke(cmd, args)` yardımcısı

- [ ] **Step 1: `bridge.js` yaz:** IIFE; `window.top !== window || location.host !== "web.whatsapp.com"` ise çık. `invoke = (c, a) => window.__TAURI_INTERNALS__?.invoke(c, a).catch(() => {})`. Başlık: `setInterval` 1000 ms ile `document.title` son gönderilenden farklıysa `invoke("set_unread", { title })` (DOM hazır olmadan da çalışır; başlık gözlemcisi kurmaya gerek yok).
- [ ] **Step 2: `set_unread` uygula:** `parse(&title)`; `state.unread` ile aynıysa dön; değilse kaydet ve `tray::update_unread`.
- [ ] **Step 3: `tray.rs` uygula:** `TrayIconBuilder::with_id("main")`, ikon `tray.png`, ipucu `"WhatsappRS"`. Menü (Türkçe, bu sırayla): `Göster / Gizle`, `Yeniden yükle` (`window.eval("location.reload()")`), ayırıcı, `Açılışta başlat` (CheckMenuItem; Task 9 bağlar, şimdilik devre dışı), `Kaldırmaya hazırla` (yalnız `!installed` iken; Task 9 bağlar), ayırıcı, `Çıkış` (`save_geometry` + `app.exit(0)`). Sol tık → `window::toggle`. `update_unread(n)`: `n > 0` → ikon `tray-unread.png`, ipucu `"WhatsappRS — {n} okunmamış"`, `window.set_overlay_icon(Some(tray-unread))`; `n == 0` → normal ikon, `"WhatsappRS"`, `set_overlay_icon(None)`.
- [ ] **Step 4: Kontrol:** `cargo test --workspace` (Linux'ta core) ve `cargo check -p whatsapprs --target x86_64-pc-windows-msvc`; push → CI yeşil.
- [ ] **Step 5: Commit + push:** `"app: okunmamış rozeti ve tepsi menüsü"`

### Task 7: Bildirimler

**Files:**
- Create: `app/src/notify.rs`
- Modify: `app/src/commands.rs` (`notify`), `app/inject/bridge.js`, `app/Cargo.toml` (`[target.'cfg(windows)'.dependencies] tauri-winrt-notification = "0.8"`)

**Interfaces:**
- Consumes: `policy::should_show_toast`, `paths::is_installed` (`AppState.installed`), `window::show`
- Produces: `#[tauri::command] fn notify(app: AppHandle, state: State<AppState>, id: u32, title: String, body: String)`; JS `window.__wrs.click(id)`

- [ ] **Step 1: `bridge.js`'e Notification yerine geçen sınıf ekle:**

```js
const store = new Map(); let seq = 0;
class WrsNotification extends EventTarget {
  constructor(title, opts = {}) {
    super(); this.id = ++seq; this.title = title; this.body = opts.body || ""; this.onclick = null;
    store.set(this.id, this); if (store.size > 100) store.delete(store.keys().next().value);
    invoke("notify", { id: this.id, title: String(title), body: String(this.body) });
  }
  close() { store.delete(this.id); }
  static get permission() { return "granted"; }
  static requestPermission(cb) { cb && cb("granted"); return Promise.resolve("granted"); }
}
window.Notification = WrsNotification;
if (window.ServiceWorkerRegistration) ServiceWorkerRegistration.prototype.showNotification =
  function (t, o) { new WrsNotification(t, o); return Promise.resolve(); };
window.__wrs = { click(id) { const n = store.get(id); if (!n) return; store.delete(id);
  const e = new Event("click"); n.onclick && n.onclick(e); n.dispatchEvent(e); } };
```

- [ ] **Step 2: `notify` uygula:** `should_show_toast(window.is_visible()?, window.is_focused()?)` false ise dön. Windows'ta `tauri_winrt_notification::Toast::new(app_id)` — `app_id` kurulu ise `"com.efeozbilgin.whatsapprs"` (NSIS Başlat menüsü kısayoluna bu AUMID'i atar), değilse `Toast::POWERSHELL_APP_ID`; `.title(&title).text1(&body).on_activated(move |_| { window::show(&app); window.eval(&format!("window.__wrs&&window.__wrs.click({id})")); Ok(()) }).show()`. Hata → `log::warn!`. Windows dışı: yalnız log (Linux daha sonra).
- [ ] **Step 3: Kontrol:** `cargo check -p whatsapprs --target x86_64-pc-windows-msvc`; push → CI yeşil.
- [ ] **Step 4: Commit + push:** `"app: Windows bildirimleri ve tıklayınca sohbeti açma"`

### Task 8: Kamera / mikrofon izinleri (aramalar)

**Files:**
- Create: `app/src/permissions.rs`
- Modify: `app/src/window.rs` (oluşturduktan sonra `permissions::install(&window)`), `app/Cargo.toml` (`[target.'cfg(windows)'.dependencies] webview2-com = "0.39"`, `windows` özelliklerine `Win32_System_WinRT` gerekirse ekle)

**Interfaces:**
- Consumes: `policy::{allow_permission, PermKind}`
- Produces: `permissions::install(window: &WebviewWindow) -> tauri::Result<()>` (Windows dışında no-op)

- [ ] **Step 1: Uygula:** `window.with_webview(|wv| { let core = wv.controller().CoreWebView2()?; core.add_PermissionRequested(&PermissionRequestedEventHandler::create(Box::new(|_, args| { ... })), &mut token) })`. İşleyicide `args.Uri()` → origin, `args.PermissionKind()` → `COREWEBVIEW2_PERMISSION_KIND_MICROPHONE`→`Microphone`, `_CAMERA`→`Camera`, `_NOTIFICATIONS`→`Notifications`, diğerleri→`Other`; `allow_permission` sonucu → `SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW / _DENY)`. Her karar `log::info!("izin {kind:?} {origin} -> {allow}")`. Ekran paylaşımı WebView2'nin kendi seçim penceresiyle yapılır; ek kod yok.
- [ ] **Step 2: Kontrol:** `cargo check -p whatsapprs --target x86_64-pc-windows-msvc`; push → CI yeşil.
- [ ] **Step 3: Commit + push:** `"app: arama için kamera ve mikrofon izinleri"`

### Task 9: Global kısayol + otomatik başlatma + kaldırmaya hazırla

**Files:**
- Create: `app/src/shortcuts.rs`, `app/src/autostart.rs`
- Modify: `app/src/tray.rs` (menü öğelerini bağla), `app/src/lib.rs`, `app/Cargo.toml` (`tauri-plugin-global-shortcut = "2.4"`, `tauri-plugin-autostart = "2.7"`)

**Interfaces:**
- Consumes: `AppState.settings`, `window::toggle`
- Produces: `shortcuts::init(app: &AppHandle)`; `autostart::{is_enabled(app) -> bool, set(app, on: bool) -> Result<(), String>}`

- [ ] **Step 1: `shortcuts::init`:** `settings.shortcut`'ı `Shortcut::from_str` ile ayrıştır; başarısızsa `log::warn!` ve `"Ctrl+Alt+W"` kullan. `register` hatası (başka uygulama almış) → `log::warn!` + bir kez toast/uyarı: `"Kısayol {s} kullanılamıyor; settings.json'dan değiştirebilirsiniz."`. Basılınca (`ShortcutState::Pressed`) `window::toggle`.
- [ ] **Step 2: `autostart`:** `tauri_plugin_autostart::Builder::new().args(["--minimized"]).app_name("WhatsappRS")` (sürümde `app_name` yoksa varsayılan adın `WhatsappRS` olduğunu kayıt defteri değerinden doğrula — NSIS kaldırıcısı bu değeri adıyla siler). Tepsi `Açılışta başlat` işareti açılışta `is_enabled`'dan okunur; tıklayınca `set(!current)`, hata → işaret geri alınır + log.
- [ ] **Step 3: `Kaldırmaya hazırla`** (yalnız taşınabilir): `set(false)`, sonra bilgi toast'u `"Otomatik başlatma kaydı silindi. Uygulamadan çıkıp klasörü silebilirsiniz."`, ardından `app.exit(0)`.
- [ ] **Step 4: Kontrol:** `cargo check ... --target x86_64-pc-windows-msvc`; push → CI yeşil.
- [ ] **Step 5: Commit + push:** `"app: global kısayol, otomatik başlatma ve kaldırmaya hazırla"`

### Task 10: Kurulum programı, taşınabilir zip, Release ve test listesi

**Files:**
- Create: `app/windows/installer.nsi` — `tauri-bundler` 2.x'in (kullanılan `tauri-cli` sürümüyle aynı) `src/bundle/windows/nsis/installer.nsi` şablonunun kopyası, aşağıdaki değişikliklerle
- Modify: `app/tauri.conf.json` — `bundle.windows.nsis = { "installMode": "currentUser", "template": "windows/installer.nsi", "languages": ["Turkish", "English"], "displayLanguageSelector": false }`, `bundle.windows.webviewInstallMode = { "type": "downloadBootstrapper" }`
- Modify: `.github/workflows/build.yml`
- Create: `docs/TEST.md`, `README.md`

**Interfaces:**
- Consumes: Run değeri adı `WhatsappRS` (Task 9), veri klasörü `data\` (Task 3)

- [ ] **Step 1: Şablon değişiklikleri:**
  - Varsayılan kurulum dizini `$LOCALAPPDATA\Programs\WhatsappRS` (dizin seçme sayfası kalır).
  - Kaldırıcıdaki "uygulama verisini sil" onay kutusu yerine özel bir `nsDialogs` sayfası: soru `"WhatsApp oturumunuz ve verileriniz saklansın mı?"`, iki radyo düğmesi `"Evet, sakla"` / `"Hayır, hepsini sil"`, **ikisi de seçili değil**; seçim yapılana kadar `İleri` düğmesi `EnableWindow 0` ile kapalı, radyo `OnClick` ile açılır. Sessiz kaldırmada (`/S`) varsayılan "sakla" (güncellemeler veriyi silmesin).
  - Kaldırma her durumda: çalışan `WhatsappRS.exe` kapatılır, program dosyaları, kısayollar, `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` → `WhatsappRS` değeri ve "Uygulamalar" kaydı silinir. "Hayır" → `RMDir /r "$INSTDIR"`. "Evet" → `$INSTDIR\data` kalır.
  - Şablonun `%APPDATA%`/`%LOCALAPPDATA%\<identifier>` silen kısımları kalır (uygulama oraya yazmaz ama zararsız temizlik).
  - Kurulum/güncelleme `$INSTDIR\data`'ya dokunmaz.
- [ ] **Step 2: CI genişlet:** Windows işine `Compress-Archive` ile `WhatsappRS-portable.zip` (içinde `WhatsappRS\WhatsappRS.exe`) adımı; artifact'e ekle. `v*` etiketinde `softprops/action-gh-release@v2` ile setup exe + zip'i Release'e yükle (`permissions: contents: write`).
- [ ] **Step 3: `docs/TEST.md`:** spec §6'daki elle test listesi, her madde onay kutusu ve beklenen sonuçla (ör. "Uygulamayı kapatıp aç → QR istenmez"). Kaldırma için üç senaryo: Evet, Hayır, seçim yapmadan İleri'ye basmaya çalışma (basılamamalı).
- [ ] **Step 4: `README.md`:** ne olduğu, indirme (Actions artifact / Release), SmartScreen notu ("Ek bilgi → Yine de çalıştır"), klasör yapısı, kısayol değiştirme, marka notu.
- [ ] **Step 5: Push + CI doğrula:** artifact'te `WhatsappRS_0.1.0_x64-setup.exe` ve `WhatsappRS-portable.zip` var.
- [ ] **Step 6: Commit + push:** `"dağıtım: kurulum programı, taşınabilir zip ve test listesi"`
- [ ] **Step 7: Kullanıcıya teslim:** artifact bağlantısı + `docs/TEST.md`; kullanıcı Windows'ta test eder. Sorun yoksa `ilk-surum` → `main` birleştirme ve `v0.1.0` etiketi kullanıcı onayıyla yapılır.
