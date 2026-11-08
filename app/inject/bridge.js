// WhatsappRS köprüsü: WhatsApp Web'den yalnızca okunmamış sayısını ve bildirimleri Rust'a iletir.
(() => {
  if (window.top !== window || location.host !== "web.whatsapp.com") return;

  const invoke = (cmd, args) => {
    const t = window.__TAURI_INTERNALS__;
    if (!t) return;
    Promise.resolve(t.invoke(cmd, args)).catch(() => {});
  };

  // Okunmamış sayısı: başlık "(3) WhatsApp" biçimindedir; ayrıştırmayı Rust yapar.
  // Pencere gizliyken tarayıcı zamanlayıcıları dakikada bire kadar yavaşlatabilir; bu yüzden asıl
  // tetikleyici başlık değişimini izleyen MutationObserver, setInterval yalnızca yedek.
  let lastTitle = null;
  const sendTitle = () => {
    const title = document.title;
    if (title !== lastTitle) {
      lastTitle = title;
      invoke("set_unread", { title });
    }
  };
  const watchHead = () =>
    new MutationObserver(sendTitle).observe(document.head, { subtree: true, childList: true, characterData: true });
  if (document.head) watchHead();
  else document.addEventListener("DOMContentLoaded", watchHead, { once: true });
  setInterval(sendTitle, 5000);

  // Bildirimler: WebView2 web bildirimlerini göstermez; WhatsApp'ın çağrılarını yakalayıp Rust'a iletiyoruz.
  const store = new Map();
  // Sayfa yenilenince eski bildirime tıklanırsa yanlış sohbet açılmasın: kimlikler her yüklemede farklı başlar.
  let seq = Math.floor(Math.random() * 1e6) * 1000;
  class WrsNotification extends EventTarget {
    constructor(title, opts = {}) {
      super();
      this.id = ++seq;
      this.title = String(title);
      this.body = opts.body ? String(opts.body) : "";
      this.tag = opts.tag || "";
      this.data = opts.data;
      this.onclick = null;
      this.onclose = null;
      this.onerror = null;
      this.onshow = null;
      store.set(this.id, this);
      if (store.size > 100) store.delete(store.keys().next().value);
      invoke("notify", { id: this.id, title: this.title, body: this.body });
    }
    close() { store.delete(this.id); }
    static get permission() { return "granted"; }
    static requestPermission(cb) {
      if (typeof cb === "function") cb("granted");
      return Promise.resolve("granted");
    }
  }
  window.Notification = WrsNotification;
  if (window.ServiceWorkerRegistration) {
    ServiceWorkerRegistration.prototype.showNotification = function (title, opts) {
      new WrsNotification(title, opts);
      return Promise.resolve();
    };
  }

  // "Windows için WhatsApp'ı indirin" çubuğunu gizle. WhatsApp'ın CSS sınıfları rastgele olduğundan
  // metinden bulunur; çubuğun kendisi, metni içeren ve 100 px'den alçak en dış kapsayıcıdır.
  const BANNER_TEXT = ["WhatsApp'ı indirin", "Download WhatsApp for Windows", "WhatsApp uygulamasını indirin"];
  const xpath = "//span[" + BANNER_TEXT.map((t) => `contains(text(), "${t}")`).join(" or ") + "]";
  const hideBanner = () => {
    if (!document.body) return false;
    const found = document.evaluate(xpath, document.body, null, XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null);
    for (let i = 0; i < found.snapshotLength; i++) {
      let el = found.snapshotItem(i);
      while (el.parentElement && el.parentElement !== document.body && el.parentElement.offsetHeight > 0 && el.parentElement.offsetHeight < 100) {
        el = el.parentElement;
      }
      if (el.style.display !== "none") el.style.setProperty("display", "none", "important");
    }
    return found.snapshotLength > 0;
  };
  // Sayfa yüklenirken sık, sonra seyrek kontrol (uzun sohbetlerde performansı etkilemesin).
  let checks = 0;
  const bannerTimer = setInterval(() => {
    hideBanner();
    if (++checks === 60) {
      clearInterval(bannerTimer);
      setInterval(hideBanner, 30000);
    }
  }, 2000);

  window.__wrs = {
    click(id) {
      const n = store.get(id);
      if (!n) return;
      store.delete(id);
      const e = new Event("click");
      if (typeof n.onclick === "function") n.onclick(e);
      n.dispatchEvent(e);
    },
  };
})();
