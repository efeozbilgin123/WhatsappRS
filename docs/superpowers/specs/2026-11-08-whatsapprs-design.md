# WhatsappRS — Tasarım Belgesi

Tarih: 2026-11-08
Durum: İncelemede

## 1. Amaç

Windows'taki resmi WhatsApp masaüstü uygulaması yavaş, çok RAM/CPU kullanıyor,
hatalı (donma, bildirim kaçırma) ve bazı kullanım kolaylıkları eksik.
WhatsappRS, **WhatsApp Web'i hafif bir Rust (Tauri v2) kabuğu içinde** çalıştırarak
WhatsApp'ın tüm özelliklerini (mesajlaşma, medya, sesli/görüntülü ve grup aramaları)
korur ve etrafına yerel masaüstü özellikleri ekler.

### Kullanıcının söyledikleri
- Windows kullanıyor; Linux ileride olabilir.
- Sorunlar: yavaşlık, hatalar, eksik özellikler, arayüz.
- Arama özelliği mutlaka korunmalı.
- İlk sürümdeki ek özellikler: sistem tepsisi, yerel bildirimler, klavye kısayolu, otomatik başlatma.
- Uygulama ve tüm verisi tek klasörde olmalı; silindiğinde iz kalmamalı.
- Kurulum programı + taşınabilir (portable) sürüm.
- Derleme GitHub Actions ile.

### Varsayımlar
- Kişisel kullanım, kendi hesabı. Hesap banı kabul edilemez → resmi olmayan
  WhatsApp protokol kütüphaneleri **kullanılmaz**; yalnızca resmi WhatsApp Web yüklenir.
- WhatsApp Web aramaları (Temmuz 2026'da genel kullanıma açıldı) kullanıcının hesabında
  etkin. Değilse uygulama yine çalışır; arama, hesaba geldiğinde kendiliğinden çalışır.

### Başarı ölçütleri
- QR bir kez okutulur; yeniden açılışta oturum kalır.
- Sesli/görüntülü (bire bir ve grup) arama yapılabilir ve gelen arama alınabilir.
- Pencere kapatılınca uygulama tepside yaşar; okunmamış sayısı tepside ve görev çubuğunda görünür.
- Yeni mesajda Windows bildirimi gelir; bildirime tıklayınca ilgili sohbet açılır.
- `Ctrl+Alt+W` pencereyi gösterir/gizler.
- Boşta RAM kullanımı resmi masaüstü uygulamasından belirgin şekilde düşük.
- Kaldırma sonrası (veri silme seçildiyse) diskte ve kayıt defterinde iz kalmaz.

### Kapsam dışı (ilk sürüm)
Çoklu hesap, kendi tema/arayüz özelleştirmesi, uygulama kilidi, otomatik güncelleme,
Linux paketleri (kod Linux'a derlenebilir tutulur ama test edilmez), ayar penceresi.

## 2. Mimari

Tauri v2 uygulaması tek pencere açar; webview doğrudan `https://web.whatsapp.com`
yükler. Windows'ta motor WebView2'dir (Chromium/Edge tabanlı, Windows 10/11'de hazır).

```
┌──────────── WhatsappRS.exe (Rust) ────────────┐
│ main · window · tray · notify · shortcuts     │
│ autostart · settings · paths · permissions    │
│            ▲ set_unread(title)                │
│            │ notify(id, title, body)          │
│            ▼ __wrs.click(id)                  │
│ ┌───────── WebView2 ─────────────────────┐   │
│ │ web.whatsapp.com + bridge.js (enjekte) │   │
│ └────────────────────────────────────────┘   │
└───────────────────────────────────────────────┘
```

### Modüller

| Modül | Sorumluluk | Saf/test edilebilir |
|---|---|---|
| `main.rs` | Eklentileri ve modülleri bağlar, `--minimized` bayrağını okur | — |
| `paths.rs` | Exe'nin yanındaki `data\` klasörünü bulur, yazılabilirliğini doğrular | Evet |
| `settings.rs` | `data\settings.json` okuma/yazma; bozuksa `.bak` yedekleyip varsayılana döner | Evet |
| `window.rs` | Pencere oluşturma, kapatınca tepsiye gizleme, dış bağlantıları varsayılan tarayıcıda açma | — |
| `unread.rs` | Sayfa başlığından okunmamış sayısını çıkarır | Evet |
| `tray.rs` | Tepsi simgesi, ipucu, rozet; görev çubuğu kaplama simgesi; menü | — |
| `notify.rs` | `notify` komutu → Windows toast; tıklamada pencereyi öne alır ve `__wrs.click(id)` çağırır | — |
| `permissions.rs` | Kamera/mikrofon/ekran paylaşımı izin kararları (yalnız `https://web.whatsapp.com`) | Evet (karar fonksiyonu) |
| `shortcuts.rs` | Global kısayolu kaydeder (varsayılan `Ctrl+Alt+W`, ayardan değiştirilebilir) | — |
| `autostart.rs` | `HKCU\...\Run` girdisi ekler/kaldırır (`--minimized` ile) | — |
| `inject/bridge.js` | Başlık değişimini ve `Notification` çağrılarını Rust'a iletir | — |

Kullanılan eklentiler: `tauri-plugin-single-instance`, `tauri-plugin-global-shortcut`,
`tauri-plugin-autostart`, `tauri-plugin-window-state`, `tauri-plugin-opener`.
Bildirimler, tıklama olayını desteklemesi için Windows'ta doğrudan WinRT toast
(`tauri-winrt-notification`) ile gösterilir.

### Güvenlik
- Uzak siteye Tauri'nin genel API'leri açılmaz. Capability dosyası yalnızca
  `https://web.whatsapp.com` kaynağına yalnızca iki komut izni verir: `set_unread`, `notify`.
- İzin işleyicisi kaynak ≠ `https://web.whatsapp.com` ise her isteği reddeder.
- `web.whatsapp.com` dışındaki gezinmeler ve yeni pencere istekleri webview'da açılmaz,
  sistemin varsayılan tarayıcısına gönderilir.

## 3. Veri akışı

**Açılış:** single-instance kontrolü (ikinci kopya mevcut pencereyi öne getirip çıkar) →
`paths` `data\` klasörünü hazırlar → ayarlar yüklenir → pencere (WebView2 veri klasörü
`data\webview`) oluşturulur; `--minimized` ise gizli → `bridge.js` sayfa betiklerinden
önce enjekte edilir → tepsi ve kısayol kaydedilir.

**Okunmamış:** `bridge.js` `<title>` değişimini izler → ham başlığı `set_unread`'e yollar →
`unread::parse` sayıyı çıkarır → değer değiştiyse tepsi ipucu + rozet ve görev çubuğu
kaplaması güncellenir.

**Bildirim:** WhatsApp `new Notification(title, {body, ...})` çağırır → `bridge.js`
nesneyi bir kimlikle saklar ve `notify(id, title, body)` yollar → pencere görünür ve
odaktaysa gösterilmez, değilse toast gösterilir → tıklanınca pencere öne gelir ve
`__wrs.click(id)` saklanan bildirimin `onclick`'ini tetikler (WhatsApp ilgili sohbeti açar).
`Notification.permission` sayfaya `"granted"` olarak bildirilir.

**Arama:** WebView2 `PermissionRequested` olayı → `permissions::decide(origin, kind)` →
izin ver / reddet. Gelen arama bildirimleri bildirim yolundan geçer.

**Tepsi menüsü:** Göster/Gizle · Yeniden yükle · Açılışta başlat ✓ ·
Kaldırmaya hazırla (yalnızca taşınabilir sürümde; Run girdisini siler) · Çıkış.
Tepsi simgesine sol tık pencereyi gösterir/gizler.

## 4. Klasör yapısı ve dağıtım

Her iki sürüm aynı exe'dir; veriler daima exe'nin yanındadır:

```
WhatsappRS\
├─ WhatsappRS.exe
└─ data\
   ├─ settings.json
   ├─ logs\            (dönüşümlü, toplam ≤ 5 MB)
   └─ webview\         (WhatsApp oturumu, önbellek)
```

**Kurulum programı (NSIS, `currentUser`, yönetici izni istemez):**
- Varsayılan konum `%LOCALAPPDATA%\Programs\WhatsappRS\`; konum değiştirilebilir,
  seçilen yer yazılabilir değilse uyarı verilir.
- Başlat menüsü kısayolu; isteğe bağlı masaüstü kısayolu; "Uygulamalar" listesinde girdi.
- Güncelleme/yeniden kurulum `data\`'ya dokunmaz.
- WebView2 yoksa bootstrapper ile kurulur (WebView2'nin kendisi kaldırılmaz; Windows bileşenidir).
- **Kaldırıcı:** özel bir sayfada "WhatsApp oturumumu ve verilerimi sakla" sorusu,
  **Evet / Hayır** seçenekleri; başta hiçbiri seçili değildir ve seçim yapılmadan
  "İleri" etkin olmaz. Her durumda Run girdisi, kısayollar ve program dosyaları silinir;
  "Hayır" seçilirse `data\` ve klasörün tamamı da silinir.
- Kaldırmadan önce çalışan uygulama kapatılır.

**Taşınabilir sürüm:** `WhatsappRS-portable.zip`; nereye açılırsa orada çalışır.
Kaldırma: tepsi → "Kaldırmaya hazırla", sonra klasör silinir.

Not: `data\webview` WhatsApp oturumunu içerir; klasörü kopyalamak oturumu kopyalamaktır.
Oturum her zaman telefondan "Bağlı cihazlar" ile kapatılabilir.

## 5. Hata durumları

| Durum | Davranış |
|---|---|
| İnternet yok / sayfa yüklenemedi | WhatsApp'ın kendi ekranı; tepsiden "Yeniden yükle" |
| `data\` yazılamıyor | Açılışta anlaşılır hata penceresi, uygulama çıkar |
| WebView2 yok | Kurulum programı kurar; taşınabilir sürüm açılışta indirme bağlantılı hata gösterir |
| Başlık ayrıştırılamıyor | Sayı 0 sayılır, log'a uyarı yazılır |
| `settings.json` bozuk | `.bak` olarak saklanır, varsayılanlarla devam edilir |
| Global kısayol alınamıyor | Uygulama çalışmaya devam eder; bir kez bildirim + log |
| İzinsiz kaynaktan komut/izin isteği | Reddedilir |
| Otomatik başlatma kaydı başarısız | Menüdeki işaret değişmez, bildirim + log |

## 6. Test

**Otomatik (cargo test, Actions'ta Windows'ta çalışır):**
- `unread::parse`: `"(3) WhatsApp"`→3, `"WhatsApp"`→0, `"(99+) WhatsApp"`→99, boş/bozuk→0.
- `permissions::decide`: doğru kaynak + kamera/mikrofon/ekran → izin; başka kaynak veya
  başka izin türü → ret; `http://` ve alt alan adı taklitleri → ret.
- `settings`: yaz-oku döngüsü, eksik alanlarda varsayılan, bozuk dosyada `.bak` + varsayılan.
- `paths`: exe yanındaki `data\` çözümü, yazılamayan klasörde hata.

**Elle (kullanıcı, Windows):** QR ile giriş; yeniden açılışta oturumun kalması; bildirim
ve tıklayınca sohbetin açılması; rozet; tepsiye gizleme; `Ctrl+Alt+W`; otomatik başlatma;
sesli + görüntülü + ekran paylaşımlı test araması; dış bağlantının tarayıcıda açılması;
kurulum, güncelleme üzerine kurulum, iki seçenekle kaldırma; taşınabilir sürümde
"Kaldırmaya hazırla".

## 7. Derleme (GitHub Actions)

`windows-latest` üzerinde: `cargo test` → `tauri build` (NSIS) → taşınabilir zip.
Çıktılar workflow artifact'ı olarak yüklenir; `v*` etiketinde GitHub Release'e eklenir.
Kod imzalama yok (SmartScreen ilk açılışta uyarı gösterebilir: "Ek bilgi → Yine de çalıştır").

## 8. Riskler

- **Toast tıklaması:** Kurulmamış (imzasız, AUMID kayıtsız) uygulamalarda Windows toast
  tıklama olayları güvenilir olmayabilir. Kurulumda Başlat menüsü kısayoluna AUMID
  atanır; taşınabilir sürümde tıklama çalışmazsa yalnızca pencere öne getirme hedeflenir.
- **WhatsApp değişiklikleri:** Başlık biçimi veya bildirim kullanımı değişirse rozet/bildirim
  bozulabilir; mesajlaşma ve arama etkilenmez.
- **Kısayol çakışması:** Türkçe klavyede `Ctrl+Alt` AltGr gibi davranır; `AltGr+W` bir
  karaktere bağlıysa çakışabilir. Kısayol `settings.json`'dan değiştirilebilir.
- **Marka:** "WhatsApp" adı Meta'nın tescilli markasıdır; depo herkese açık. Kişisel kullanım
  için sorun değil, dağıtılacaksa isim değiştirilmeli.
