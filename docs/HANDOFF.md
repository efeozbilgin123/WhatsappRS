# WhatsappRS — Devir Belgesi (yapay zekâ ajanları ve geliştiriciler için)

> Son güncelleme: 2026-11-09 · Son yayınlanan sürüm: **0.2.8** · Çalışma dalı: `ilk-surum` (her sürümde `main`'e birleştirilir)
>
> Bu belge, kullanıcıyla yapılan uzun bir sohbetin (tasarım, uygulama, 9 sürüm, hata ayıklama) özetidir.
> İşe başlamadan önce **tamamını** oku. Ayrıntılı tasarım: `docs/superpowers/specs/2026-11-08-whatsapprs-design.md`,
> ilk uygulama planı: `docs/superpowers/plans/2026-11-08-whatsapprs.md`, elle test listesi: `docs/TEST.md`.

---

## 1. Proje nedir

**WhatsappRS**, Windows için WhatsApp masaüstü istemcisi. Resmi WhatsApp masaüstü uygulamasını beğenmeyen
kullanıcı için yazıldı (yavaş, çok RAM, hatalı, eksik özellik, Linux'ta yok).

- **Yaklaşım:** Resmi `https://web.whatsapp.com` sayfası **Tauri v2** (Rust) penceresinde, WebView2 (Chromium) ile çalışır.
  Resmi olmayan WhatsApp protokol kütüphaneleri (whatsmeow, Baileys vb.) **bilerek kullanılmadı**: hesap ban riski +
  arama (sesli/görüntülü) desteği yok. Arama kullanıcı için zorunlu.
- **WhatsApp Web aramaları:** 2026 Şubat'ta betada başladı, 28 Temmuz 2026'da genel kullanıma açıldı (bire bir + grup,
  ekran paylaşımı). Kamera/mikrofon izinleri yalnızca `https://web.whatsapp.com` kaynağına verilir.
- **Eklenen masaüstü özellikleri:** sistem tepsisi (kapatınca tepsiye), okunmamış göstergesi (turuncu nokta),
  Windows bildirimleri (tıklayınca sohbet açılır), global kısayol `Ctrl+Alt+W`, açılışta başlatma, koyu tema +
  WhatsApp rengindeki başlık çubuğu, indirme bildirimi, dosya sürükle-bırak, yakınlaştırma, otomatik güncelleme
  (kurulu **ve** taşınabilir sürüm), tek kopya.
- **Depo:** https://github.com/efeozbilgin123/WhatsappRS (herkese açık, GPL-3.0)
- **Sürümler:** https://github.com/efeozbilgin123/WhatsappRS/releases

## 2. Kullanıcı hakkında (mutlaka uy)

- Türkçe, samimi konuşur ("kanka"). **Yanıtları Türkçe ver**, teknik olmayan kişiye anlatır gibi sade yaz.
- Windows kullanıyor (Windows 11, Intel 12. nesil hibrit CPU, 12 çekirdek, 16 GB RAM, Intel UHD Graphics,
  ekran ölçeği %125, görev çubuğunda **küçük simgeler**).
- **Edge'i CTT WinUtil ile silmiş, Edge kurmayı reddediyor.** WebView2 Runtime kurulu ve sağlam (157.0.4341.42;
  Microsoft kurulum programı "zaten yüklü" dedi). Kayıt defterinde HKLM/HKCU `Policies\Microsoft\Edge` ve
  `...\Edge\WebView2` anahtarları **yok**.
- Uygulama kurulu: `C:\Users\Efe\AppData\Local\Programs\WhatsappRS` (kurulum programıyla).
- **PWA / PWAsForFirefox önermeyi reddetti** ("pwa kurmayacağız"). Çözümleri biz geliştirmeliyiz.
- Uzun test istemez ("üşeniyorum"); kısa, net, tek adımlı istekler işe yarar. Log dosyasını gönderir.
- **Git kuralları (kullanıcının açık isteği):**
  - Commit'ler **kullanıcı adına**: yazar `efe <79136287+efeozbilgin123@users.noreply.github.com>` (depoda `git config` ile ayarlı).
  - **`Co-Authored-By: Claude` satırı EKLEME**, commit mesajlarında Claude'dan söz etme. Commit mesajları Türkçe.
  - Bir oturum kancası (stop hook) commit'leri Claude adına yeniden yazmayı önerebilir — **yapma**, kullanıcı reddetti.

## 3. Sürüm geçmişi (özet)

| Sürüm | Öne çıkan |
|---|---|
| 0.1.0 | İlk sürüm: Tauri iskeleti, tepsi, bildirim, kısayol, otomatik başlatma, NSIS kurulum + taşınabilir zip |
| (inceleme) | Ayar dosyası elle düzenlemeleri koruma, kendi HKCU otomatik başlatma kodu (eklenti yerine), kaldırıcıda `data\.whatsapprs` işareti, bildirimlerde WebView2 izni reddi |
| 0.2.0 | Otomatik güncelleme, uyumluluk modu, yeni logo, koyu tema/başlık rengi, indirme bildirimi, sürükle-bırak, yakınlaştırma |
| 0.2.1 | Bağlantılar için gizli yardımcı pencere (about:blank ile açılan bağlantılar), ana iş parçacığı izleyicisi |
| 0.2.2 | WebView2 sürümü + grafik sürücüsü teşhisi, büyük rozetler |
| 0.2.3 | Deney: WebView2 varsayılan başlatma ayarları; gizliyken düşük bellek hedefi |
| 0.2.4 | Görev çubuğu kaplaması ayrı ikon, takılma (long-animation-frame) ölçümü |
| 0.2.5 | Tepsi: önbelleği temizle (oturum korunur) |
| 0.2.6 | Okunmamış göstergesi: sayı yerine **turuncu nokta** (kullanıcı isteği) |
| 0.2.7 | WebView2 süreçlerinin güç kısıtlaması (EcoQoS) teşhisi; deney: yüksek performans |
| 0.2.8 | Teşhis: JavaScript JIT hız testi (`js_bench_ms`) ve WebAssembly kontrolü (`wasm`) |

Otomatik güncelleme gerçek kullanımda doğrulandı (0.2.0 → 0.2.1 kendi kendine güncellendi).

## 4. ⚠️ AÇIK SORUN: Sohbet listesi kaydırırken kasma (öncelik 1)

### Belirti
- Sol paneldeki **sohbet listesini** kaydırmak kasıyor/donuyor (sohbet içi değil).
- Cihaz **sıfırdan bağlandığında akıcı**, **birkaç dakika içinde** kasma başlıyor ve artıyor.
- **Resmi WhatsApp uygulaması** (o da WebView2/Chromium) da yavaş ama bizimkinden akıcı.
- **Zen Browser (Firefox/Gecko tabanlı)** ile WhatsApp Web: **hiç kasma yok** (kullanıcı bildirimi; yeni bağlanmıştı).

### Elenenler (kanıtla)
| Şüpheli | Sonuç | Kanıt |
|---|---|---|
| Bizim enjekte betik (`bridge.js`) | Elendi | Uyumluluk modunda (betik yok) da kasıyor |
| Ana iş parçacığının takılması | Elendi | `watchdog.rs` hiç uyarı yazmadı |
| GPU kullanılmaması | Elendi | `ANGLE (Intel UHD Graphics … D3D11)` |
| Tauri'nin ağ isteklerine karışması | Elendi | wry yalnızca kendi özel protokol önekleri için filtre ekliyor |
| WebView2 başlatma bayrakları (`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`) | Elendi | "Deney: WebView2 varsayılan ayarları" ile de kasıyor |
| Windows güç kısıtlaması (EcoQoS) | Elendi | Tüm süreçler `normal`; "Deney: yüksek performans" işe yaramadı |
| Bozuk önbellek | Elendi | "Önbelleği temizle" işe yaramadı |
| PDF açma tetikleyicisi | Zayıf | Yeniden bağlayınca geçti ama zamanla yine geldi → veri büyümesi |

### Ölçülenler (`takılma:` log satırları)
- Açılışta: ~35–38 uzun kare, toplam ~5 sn, en uzun ~0,8–1,1 sn, kaynak `static.whatsapp.net/…js IDBTransaction.oncomplete`.
- Kaydırırken: 10 sn'de 8–17 uzun kare, en uzun 237–377 ms, kaynak `static.whatsapp.net/…js MessagePort.onmessage`
  (WhatsApp'ın kendi React zamanlayıcısı). Yani süre **WhatsApp'ın JavaScript'inde** geçiyor.

### Bekleyen
- **0.2.8 log'u henüz gelmedi.** Bakılacak satır: `grafik: {"js_bench_ms": …, "wasm": …}`.
  - `wasm: false` → V8 JIT kapalı (jitless). Sebep bulunmuş olur; nedenini (WinUtil, güvenlik yazılımı vb.) araştır.
  - Referans: bu geliştirme sunucusunda `node` ile JIT açık 123 ms, `--jitless` 461 ms (~4 kat). Kullanıcının CPU'su daha hızlı.
- Henüz denenmeyen Chromium tarafı fikirler:
  1. WebView2 veri klasörünü `Programs` dışına taşıma deneyi (Defender taraması şüphesi; resmi uygulama/Zen verisini başka yerde tutuyor).
  2. long-animation-frame girdisinde `scripts[].duration` ile `styleAndLayoutStart`/`forcedStyleAndLayoutDuration` ayrımını log'lamak (JS mi, yerleşim mi?).

### Önerilen büyük çözüm: Firefox (Gecko) motorlu WhatsappRS — **kullanıcı onayı bekleniyor**
Kullanıcı "biz niye Firefox tabanlı yapmıyoz… düşün ve bi yol bul" dedi; aşağıdaki tasarım sunuldu, son soru
"Deneme sürümüne başlayayım mı?" idi. **Kullanıcı yanıt vermeden oturum limiti doldu ve "limit yenilenince bu işe devam
etme, devir belgesi yaz" dedi.** Yani bu işe kendiliğinden başlama; kullanıcıdan açık onay al.

- Gecko Windows'ta gömülemez (embedding API yok, GeckoView yalnız Android). Servo WhatsApp Web'i/WebRTC'yi taşıyamaz.
- **Plan:** Mozilla'nın resmi, değiştirilmemiş Firefox'u uygulama klasörüne (`motor\firefox\`) konur; Rust exe onu
  `--profile data\profil --no-remote` ile **yalnızca WhatsApp penceresi** olarak açar (`userChrome.css` ile sekme/adres
  çubuğu gizli, `toolkit.legacyUserProfileCustomizations.stylesheets=true`, koyu tema).
  - Okunmamış: Firefox penceresinin başlığı ("(3) WhatsApp …") Win32 `GetWindowText` ile okunur (eklenti gerekmez).
  - Bildirim: Firefox'un kendi Windows bildirimleri (`alerts.useSystemBackend`).
  - Kapat → tepsi: Firefox X ile kapanırsa Rust onu **gizli olarak yeniden başlatır** (bildirimler sürer); küçültünce
    `SetWinEventHook(EVENT_SYSTEM_MINIMIZESTART)` + `ShowWindow(SW_HIDE)`.
  - Firefox güvenlik güncellemelerini kendi güncelleyicisi yapar (klasör yazılabilir); WhatsappRS güncellemesi bizde kalır.
  - Bedel: kurulum ~100 MB, pencere/WebView katmanının yeniden yazılması.
- **Önce deneme (spike):** ayrı klasörde, otomatik güncelleme kanalına girmeyen bir prototip. Doğrulanacaklar:
  1. saatler sonra kasma yok mu, 2. **aramalar Firefox'ta çalışıyor mu** (haberler "çalışır" diyor, resmi kaynak görülmedi),
  3. sekme/adres çubuğu gizli mi, 4. tepsi/rozet/bildirim akışı.
- Başarılıysa: brainstorming → spec → plan → 1.0 (Firefox motoru).

## 5. Diğer doğrulanmamış / açık konular

- **Bağlantılar açılmıyor** sorunu (0.2.0'da bildirildi): 0.2.1'de `about:blank` için yardımcı pencere eklendi;
  kullanıcı **düzeldiğini teyit etmedi**. Log'da `yeni pencere isteği:` satırlarına bak.
- WebView2 izin türü **13** (bilinmiyor, reddediliyor) — WhatsApp açılışta istiyor; etkisi bilinmiyor.
- Bellek: resmi uygulama arka planda sayfayı "verimlilik modu"na alıyor; biz 0.2.3'ten beri gizliyken
  `MemoryUsageTargetLevel=Low` istiyoruz. Etkisi ölçülmedi.
- Teşhis/deney tepsi seçenekleri (uyumluluk modu, WebView2 varsayılan ayarları, yüksek performans, önbelleği temizle)
  sorun çözülünce sadeleştirilmeli/kaldırılmalı.
- Ertelenen küçükler: tepsi menüsü öğeleri çoğaldı; `docs/TEST.md` bazı yeni öğeleri içermiyor.

## 6. Mimari ve dosya haritası

```
Cargo.toml                 workspace: core, app
core/  (Tauri'siz, Linux'ta test edilir — 43 birim test)
  src/unread.rs            "(3) WhatsApp" → 3
  src/policy.rs            izin kararları, iç/dış URL, yeni pencere kararı (OpenInApp/OpenExternal/OpenViaHelper/Deny),
                           bildirim odak kuralı, ikinci kopya, otomatik başlatma komutu (tırnaklı)
  src/paths.rs             exe yanındaki data\ (settings.json, logs\, webview\), yazılabilirlik, .whatsapprs işareti,
                           kurulum algılama (uninstall.exe), WebView2 önbellek temizliği
  src/settings.rs          Settings {shortcut, window, compat_mode, webview_defaults, clear_cache_on_start,
                           high_performance}; load (bozuksa .bak, .bak1…), save (atomik), update (diğer alanları korur),
                           geom_is_visible
  src/selfupdate.rs        taşınabilir sürüm exe takası (.new/.old), cleanup_old
app/   (Tauri v2 uygulaması; yalnızca Windows'ta çalıştırılır)
  tauri.conf.json          sürüm, NSIS, updater (pubkey + latest.json endpoint), createUpdaterArtifacts
  capabilities/remote.json web.whatsapp.com'a YALNIZCA allow-set-unread, allow-notify, allow-perf-report
  build.rs                 AppManifest commands: set_unread, notify, perf_report (yeni komut eklerken BURAYA da ekle)
  inject/bridge.js         sayfaya enjekte: başlık izleme (<title>), Notification taklidi, indirme çubuğu gizleme,
                           long-animation-frame ölçümü; test: inject/test/bridge.test.js (Playwright, 6 senaryo)
  src/lib.rs               başlangıç: yollar, log, ayarlar, önbellek temizliği, eklentiler, setup
  src/window.rs            ana pencere, koyu tema, gezinme/yeni pencere/indirme işleyicileri, bağlantı yardımcı penceresi,
                           kapat→gizle, konum kaydı
  src/tray.rs              tepsi menüsü ve okunmamış noktası, görev çubuğu kaplaması
  src/notify.rs            WinRT toast (tauri-winrt-notification); kurulu sürümde AUMID com.efeozbilgin.whatsapprs
  src/permissions.rs       WebView2 PermissionRequested (origin kontrolü) + ProcessFailed (çökmede yeniden yükle)
  src/update.rs            tauri-plugin-updater (native-tls); taşınabilir için hedef "windows-x86_64-portable"
  src/autostart.rs         HKCU\…\Run\WhatsappRS = "<exe>" --minimized (eklenti DEĞİL, kendi kodumuz)
  src/titlebar.rs          DWM başlık rengi #161717 (Windows 11)
  src/memory.rs            gizliyken MemoryUsageTargetLevel Low
  src/watchdog.rs, diag.rs, perf.rs   teşhis (ana iş parçacığı, grafik/JIT, EcoQoS)
  src/shortcuts.rs, commands.rs, logging.rs (flexi_logger, data\logs, ≤5 MB), fatal.rs (MessageBox)
  windows/installer.nsi    tauri-bundler 2.10.4 şablonunun uyarlaması (WHATSAPPRS etiketli satırlar)
assets/icon-source.png     1024 px logo (Gemini ile üretildi, eflatun zemin silindi)
.github/workflows/build.yml  test (Linux) + windows (derleme, imza, latest.json) + release
```

### Önemli tasarım kararları
- **Tek klasör:** her şey exe'nin yanında (`data\`). Tek dış iz: `HKCU\…\Run\WhatsappRS` (kaldırıcı siler; ayrıca
  `StartupApproved\Run` ve HKLM değeri de silinir). `tauri-plugin-window-state` bu yüzden kullanılmadı.
- **Kurulum:** NSIS, `currentUser`, varsayılan `%LOCALAPPDATA%\Programs\WhatsappRS`. Kaldırıcıda özel sayfa:
  "Verileriniz saklansın mı?" Evet/Hayır, **ikisi de seçili değil, seçmeden İleri yok**. "Hayır" yalnızca
  `.whatsapprs` işaretli `data\` klasörünü siler, `$INSTDIR`'i özyinelemeli silmez (kullanıcı kendi klasörüne kurmuş olabilir).
  Sessiz/pasif/güncelleme kaldırmalarında veri korunur.
- **Güvenlik:** uzak sayfaya yalnızca 3 komut açık; izinler origin'e göre; dış bağlantılar varsayılan tarayıcıda.

## 7. Geliştirme ortamı ve doğrulama

- Bu bulut oturumu **Linux**; Windows uygulaması burada çalıştırılamaz. Doğrulama:
  - `cargo test -p whatsapprs-core` (43 test)
  - `NODE_PATH=$(npm root -g) node app/inject/test/bridge.test.js` (Playwright; 6 senaryo, ~25 sn)
  - `rustup target add x86_64-pc-windows-msvc && cargo check -p whatsapprs --target x86_64-pc-windows-msvc`
    (çapraz **check** çalışır; `ring` C derleyici istediği için updater **native-tls** ile kullanılıyor)
  - Asıl Windows derlemesi/testi: GitHub Actions (`windows-latest`).
- Tuzaklar:
  - Tauri komut parametreleri JS'te varsayılan **camelCase**; snake_case gönderiliyorsa `#[tauri::command(rename_all = "snake_case")]`.
  - Yeni uzak komut: `build.rs` listesi + `capabilities/remote.json` izni + `generate_handler!`.
  - WebView2 başlatma bayrakları değişince `app.restart()` yarışa girer (eski süreç kapanmadan) → bu tür ayarlarda çık ve kullanıcı açsın.
  - PowerShell'de satır içi JSON tırnakları güvenilmez → dosyaya yazıp `-c dosya.json`.
  - Bu oturumdan **git etiket (tag) gönderimi 403** ile engelli; GitHub gizli anahtar yazma da engelli.

## 8. Sürüm yayınlama (bu oturumda çalışan yöntem)

1. Sürümü üç yerde artır: `app/tauri.conf.json` (`"version"`), `app/Cargo.toml`, `core/Cargo.toml`.
2. `ilk-surum`'a commit + push, sonra `git checkout main && git merge --no-ff ilk-surum && git push origin main`.
3. `gh workflow run build.yml --repo efeozbilgin123/WhatsappRS --ref main -f release=true`
   → iş akışı derler, imzalar, `v<sürüm>` etiketini ve GitHub Release'i kendisi oluşturur, `latest.json` yükler.
4. Doğrula: `curl -sL https://github.com/efeozbilgin123/WhatsappRS/releases/latest/download/latest.json`.
- İmzalama: depo gizli anahtarları `TAURI_SIGNING_PRIVATE_KEY` ve `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  **kullanıcı tarafından eklendi**. Anahtarın kopyası kullanıcıda. Açık anahtar `tauri.conf.json`'da.
  **Anahtar kaybolursa mevcut kurulumlara güncelleme gönderilemez.**
- `latest.json` platformları: `windows-x86_64` (NSIS setup.exe) ve `windows-x86_64-portable` (çıplak exe).

## 9. Tepsi menüsü (0.2.8)

Göster/Gizle · Yeniden yükle · ─ · Açılışta başlat ✓ · Uyumluluk modu (sorun giderme) ✓ ·
Deney: WebView2 varsayılan ayarları ✓ · Deney: yüksek performans ✓ · Önbelleği temizle (oturum korunur) ·
Güncellemeleri denetle / Güncellemeyi yükle (X) · ─ · Çıkış

## 10. Logo

Kullanıcı düz (flat) tasarımı istemedi; Discord/Steam/macOS tarzı derinlikli, sade bir ikon seçildi: lacivert→turkuaz
kare zemin, beyaz dişli kenarlı konuşma balonu, üç turkuaz nokta, turuncu yengeç kıskacı (Rust göndermesi).
Kaynak: `assets/icon-source.png`. Tüm ikonlar (ico, png, tepsi, okunmamış noktası) Python/Pillow ile bundan üretildi;
okunmamış noktası turuncu `#F97316`, görev çubuğu kaplaması yalnızca turuncu daire.

## 11. Bir sonraki ajan için yapılacaklar sırası

1. Kullanıcıdan **0.2.8 log'unu** iste (`js_bench_ms`, `wasm`). JIT kapalıysa nedenini bul.
2. Kullanıcıya **Firefox motoru deneme sürümünü** tekrar sor; onay gelmeden başlama. Onaylanırsa brainstorming → spike.
3. Bağlantıların açıldığını teyit et.
4. Kasma çözülünce deney/teşhis seçeneklerini sadeleştir, `docs/TEST.md`'yi güncelle.
