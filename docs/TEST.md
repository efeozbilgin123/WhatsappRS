# WhatsappRS — Elle Test Listesi (Windows)

Her maddeyi deneyip kutuyu işaretle. Bir şey beklendiği gibi çalışmazsa
`data\logs\whatsapprs.log` dosyasıyla birlikte bildir.

## Kurulum
- [ ] `WhatsappRS_..._x64-setup.exe` çalıştır → yönetici izni (UAC) **istenmez**.
- [ ] SmartScreen uyarısı çıkarsa: "Ek bilgi" → "Yine de çalıştır".
- [ ] Varsayılan konum `C:\Users\<sen>\AppData\Local\Programs\WhatsappRS` → kurulum biter, Başlat menüsünde "WhatsappRS" var.

## İlk açılış ve oturum
- [ ] Uygulama açılır, WhatsApp Web QR kodu görünür; telefondan okut → sohbetler gelir.
- [ ] Tepsiden "Çıkış" → uygulamayı tekrar aç → **QR istenmez**, sohbetler açılır.
- [ ] Kurulum klasöründe `data\` var (`settings.json`, `logs\`, `webview\`).

## Pencere ve tepsi
- [ ] Pencerenin X'ine bas → pencere kapanır ama tepside simge kalır.
- [ ] Tepsi simgesine sol tık → pencere geri gelir; tekrar sol tık → gizlenir.
- [ ] Pencereyi taşı/boyutlandır, Çıkış yap, tekrar aç → aynı yerde ve boyutta açılır.
- [ ] Tepsi → "Yeniden yükle" → sayfa yenilenir.

## Görünüm
- [ ] Uygulama ilk açılışta koyu temada açılır (beyaz parlama yok).
- [ ] Windows 11: başlık çubuğu WhatsApp'ın koyu rengiyle aynı (turuncu/vurgu rengi değil).
- [ ] "Windows için WhatsApp'ı indirin" çubuğu görünmüyor.
- [ ] `Ctrl` + `+` / `-` / `0` ile yakınlaştırma çalışır.

## Okunmamış rozeti
- [ ] Pencere gizliyken başka bir telefondan mesaj gönder → tepsi simgesinin sağ üstünde turuncu nokta; üzerine gelince "WhatsappRS — 1 okunmamış".
- [ ] Görev çubuğu simgesinin köşesinde de turuncu nokta görünür, logo kapanmaz.
- [ ] Mesajı oku → nokta kaybolur.
- [ ] Pencere 10+ dakika tepsideyken gelen mesajda rozet birkaç saniye içinde güncellenir.

## Bildirimler
- [ ] Pencere gizli/arka plandayken mesaj gelir → Windows bildirimi çıkar (başlık: kişi, metin: mesaj).
- [ ] Bildirime tıkla → pencere öne gelir ve **o sohbet açılır**.
- [ ] Pencere önde ve odaktayken mesaj gelir → bildirim **çıkmaz**.

## Aramalar
- [ ] Sesli arama başlat → ilk seferde mikrofon çalışır, ses iki yönlü gider.
- [ ] Görüntülü arama başlat → kamera görüntüsü gider.
- [ ] Görüntülü aramada ekran paylaş → Windows ekran seçme penceresi çıkar, paylaşım çalışır.
- [ ] Telefondan bilgisayara arama yap → bildirim gelir, aramayı uygulamadan aç.
- [ ] Grup araması (3+ kişi) dene.

## Kısayol ve bağlantılar
- [ ] Başka bir uygulamadayken `Ctrl+Alt+W` → WhatsappRS öne gelir; tekrar → gizlenir.
- [ ] Sohbette bir web bağlantısına tıkla → varsayılan tarayıcında açılır (uygulamanın içinde değil).

## Dosyalar
- [ ] Bir dosyayı Gezgin'den sohbete sürükle-bırak → gönderme ekranı açılır.
- [ ] Bir belge/fotoğraf indir → "İndirildi: dosya adı" bildirimi; tıklayınca İndirilenler klasöründe seçili açılır.
- [ ] Bir sohbette "tümünü indir" gibi birden fazla dosya indir → hepsi iner.
- [ ] Bir PDF'i önizlemede aç/kapat → çalışır.

## Kasma
- [ ] Bir PDF'i önizlemede aç/kapat, sonra uzun bir sohbette yukarı kaydır → kasma yok.
- [ ] Kasma varsa: tepsi → "Uyumluluk modu" işaretle (uygulama yeniden başlar) → aynı adımları tekrarla. Kasma kalktıysa sorun bizim betikte; kalkmadıysa başka yerde. Sonucu bildir, sonra modu kapat.

## Güncelleme
- [ ] Tepsi → "Güncellemeleri denetle" → güncel sürümdeyken "WhatsappRS güncel." bildirimi.
- [ ] Yeni sürüm yayınlandığında: "WhatsappRS X hazır" bildirimi → tıkla → uygulama kapanıp güncellenir, yeniden açılır, QR istenmez.
- [ ] Taşınabilir sürümde aynı akış: klasördeki exe yenisiyle değişir, bir sonraki açılışta `WhatsappRS.exe.old` silinir.

## Açılışta başlatma
- [ ] Tepsi → "Açılışta başlat" işaretle → bilgisayarı yeniden başlat → uygulama **gizli** (sadece tepside) açılır.
- [ ] İşareti kaldır → yeniden başlatınca açılmaz.

## Tek kopya
- [ ] Uygulama açıkken Başlat menüsünden tekrar aç → ikinci pencere açılmaz, mevcut pencere öne gelir.

## Kaldırma (3 senaryo)
- [ ] Ayarlar → Uygulamalar → WhatsappRS → Kaldır → "Verileriniz" sayfasında **hiçbir seçenek seçili değil** ve **İleri basılamıyor**.
- [ ] **Evet, sakla** → kaldır → `...\Programs\WhatsappRS\data` duruyor, diğer her şey silinmiş. Yeniden kur → QR istenmez.
- [ ] **Hayır, hepsini sil** → kaldır → `...\Programs\WhatsappRS` klasörü tamamen yok; `regedit` → `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` altında `WhatsappRS` yok; Başlat menüsünde yok.

## Taşınabilir sürüm
- [ ] `WhatsappRS-portable.zip`'i istediğin yere aç → `WhatsappRS\WhatsappRS.exe` çalışır, `data\` exe'nin yanında oluşur.
- [ ] Bildirim gelir (tıklama sohbeti açmayabilir; bu sürümde bilinen sınır).
- [ ] Tepsi → "Açılışta başlat" işaretle, sonra işareti kaldır → Run kaydı silinmiş; çık ve klasörü sil → iz kalmaz.

## Performans (isteğe bağlı)
- [ ] Görev Yöneticisi'nde boşta RAM kullanımını resmi WhatsApp uygulamasıyla karşılaştır.
