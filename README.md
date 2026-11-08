# WhatsappRS

WhatsApp Web'i hafif bir Rust ([Tauri v2](https://tauri.app)) penceresinde çalıştıran Windows masaüstü uygulaması.
Mesajlar, medya ve sesli/görüntülü/grup aramaları dahil WhatsApp Web'in tüm özellikleri aynen çalışır;
üstüne şunları ekler:

- **Sistem tepsisi:** pencereyi kapatınca arka planda çalışır, okunmamış mesajda kırmızı nokta.
- **Windows bildirimleri:** tıklayınca ilgili sohbet açılır.
- **Global kısayol:** `Ctrl+Alt+W` ile göster/gizle.
- **Açılışta başlatma:** tepsi menüsünden açılıp kapatılır, gizli başlar.
- **Tek klasör:** uygulama, oturumun ve ayarların hep aynı klasörde durur.

Resmi olmayan WhatsApp protokolü kullanılmaz; yalnızca resmi `web.whatsapp.com` yüklenir, hesap ban riski yoktur.

## İndirme

- Son derleme: GitHub → **Actions** → en son başarılı `build` → **Artifacts** → `WhatsappRS-windows`.
- Sürümler: GitHub → **Releases**.

İki seçenek var:

| Dosya | Ne |
|---|---|
| `WhatsappRS_<sürüm>_x64-setup.exe` | Kurulum programı. Yönetici izni istemez, `%LOCALAPPDATA%\Programs\WhatsappRS` klasörüne kurar. |
| `WhatsappRS-portable.zip` | Taşınabilir sürüm. İstediğin yere aç ve çalıştır. |

Uygulama imzalı olmadığı için Windows SmartScreen ilk açılışta uyarabilir: **Ek bilgi → Yine de çalıştır**.

## Klasör yapısı

```
WhatsappRS\
├─ WhatsappRS.exe
└─ data\
   ├─ settings.json   ayarlar
   ├─ logs\           hata kayıtları (en fazla ~5 MB)
   └─ webview\        WhatsApp oturumu ve önbellek
```

`data\webview` WhatsApp oturumunu içerir: bu klasörü kopyalayan oturumunu da kopyalamış olur.
Oturumu her zaman telefondan **Bağlı cihazlar** menüsünden kapatabilirsin.

## Kaldırma

- **Kurulum sürümü:** Ayarlar → Uygulamalar → WhatsappRS → Kaldır. Verilerini saklayıp saklamamayı sorar.
  "Hayır" dersen hiçbir iz kalmaz (klasör, kısayollar, açılışta başlatma kaydı silinir).
- **Taşınabilir sürüm:** tepsi menüsü → **Kaldırmaya hazırla** (açılışta başlatma kaydını siler), sonra klasörü sil.

## Ayarlar

`data\settings.json` dosyasını düzenleyebilirsin (değişiklik uygulama yeniden açılınca geçerli olur):

```json
{ "shortcut": "Ctrl+Alt+W" }
```

Türkçe klavyede `Ctrl+Alt` tuşları AltGr gibi davranabilir. Kısayol bir karakterle çakışırsa
örneğin `"Ctrl+Shift+Alt+W"` yap. Dosya bozulursa uygulama onu `settings.json.bak` olarak saklar
ve varsayılanlarla açılır.

## Geliştirme

```bash
cargo test --workspace          # testler
cargo install tauri-cli --version "^2.12" --locked
cd app && cargo tauri build     # Windows'ta: kurulum programı
```

- `core/`: Tauri'siz saf mantık (başlık ayrıştırma, izin kararları, yollar, ayarlar), Linux'ta da test edilir.
- `app/`: Tauri uygulaması. `app/inject/bridge.js` WhatsApp Web sayfasına enjekte edilir.
- `app/windows/installer.nsi`: özelleştirilmiş NSIS şablonu.
- Tasarım: `docs/superpowers/specs/`, test listesi: `docs/TEST.md`.

## Not

"WhatsApp", Meta Platforms, Inc.'in tescilli markasıdır. Bu proje Meta ile bağlantılı değildir ve kişisel
kullanım içindir.
