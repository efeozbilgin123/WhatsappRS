// WhatsappRS köprüsü: WhatsApp Web'den yalnızca okunmamış sayısını ve bildirimleri Rust'a iletir.
(() => {
  if (window.top !== window || location.host !== "web.whatsapp.com") return;

  const invoke = (cmd, args) => {
    const t = window.__TAURI_INTERNALS__;
    if (!t) return;
    Promise.resolve(t.invoke(cmd, args)).catch(() => {});
  };

  // Okunmamış sayısı: başlık "(3) WhatsApp" biçimindedir; ayrıştırmayı Rust yapar.
  // Yalnızca <title> öğesi izlenir (WhatsApp <head>'e sürekli stil ekler; onları izlemek gereksiz yük).
  // Pencere gizliyken zamanlayıcılar yavaşlayabildiği için asıl tetikleyici gözlemci, setInterval yedek.
  let lastTitle = null;
  let watchedTitle = null;
  const titleObserver = new MutationObserver(() => sendTitle());
  const sendTitle = () => {
    const el = document.querySelector("head > title");
    if (el && el !== watchedTitle) {
      titleObserver.disconnect();
      titleObserver.observe(el, { subtree: true, childList: true, characterData: true });
      watchedTitle = el;
    }
    const title = document.title;
    if (title !== lastTitle) {
      lastTitle = title;
      invoke("set_unread", { title });
    }
  };
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", sendTitle, { once: true });
  else sendTitle();
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

  // "Windows için WhatsApp'ı indirin" çubuğu: metinden bulunur (CSS sınıfları rastgele), WhatsApp'ın kendi
  // kapatma düğmesine basılır (kalıcı olarak hatırlar) ve gizlenir. Uzun sohbetlerde yük olmasın diye
  // arama yalnızca açılıştan sonraki 2 dakika sürer ve çubuk bulununca durur.
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
      const close = el.querySelector("button, [role='button']");
      if (close) close.click();
      el.style.setProperty("display", "none", "important");
    }
    return found.snapshotLength > 0;
  };
  let bannerChecks = 0;
  const bannerTimer = setInterval(() => {
    if (hideBanner() || ++bannerChecks >= 40) clearInterval(bannerTimer);
  }, 3000);

  // Teşhis: 50 ms'den uzun süren işleri (takılan kareler) 10 saniyelik özetler hâlinde log'a gönderir.
  // "source", takılmada en çok süre harcayan betiğin adı (WhatsApp'ın kendi kodu mu, başka bir şey mi).
  const perf = { count: 0, total: 0, max: 0, sources: new Map(), scrolledAt: 0 };
  document.addEventListener("scroll", () => { perf.scrolledAt = Date.now(); }, { capture: true, passive: true });
  document.addEventListener("wheel", () => { perf.scrolledAt = Date.now(); }, { capture: true, passive: true });
  const record = (duration, source) => {
    perf.count++;
    perf.total += duration;
    perf.max = Math.max(perf.max, duration);
    perf.sources.set(source, (perf.sources.get(source) || 0) + duration);
  };
  const shortSource = (url) => {
    if (!url) return "bilinmiyor";
    try { const u = new URL(url); return u.host + u.pathname.split("/").slice(-1)[0].slice(0, 40); } catch { return String(url).slice(0, 40); }
  };
  try {
    const types = PerformanceObserver.supportedEntryTypes || [];
    if (types.includes("long-animation-frame")) {
      new PerformanceObserver((list) => {
        for (const e of list.getEntries()) {
          const top = (e.scripts || []).slice().sort((a, b) => b.duration - a.duration)[0];
          record(e.duration, top ? shortSource(top.sourceURL) + " " + (top.invoker || "").slice(0, 40) : "çizim/stil");
        }
      }).observe({ type: "long-animation-frame", buffered: false });
    } else if (types.includes("longtask")) {
      new PerformanceObserver((list) => { for (const e of list.getEntries()) record(e.duration, "longtask"); })
        .observe({ type: "longtask", buffered: false });
    }
  } catch (_) {}
  setInterval(() => {
    if (perf.count === 0) return;
    const top = [...perf.sources.entries()].sort((a, b) => b[1] - a[1])[0];
    invoke("perf_report", {
      count: perf.count,
      total_ms: Math.round(perf.total),
      max_ms: Math.round(perf.max),
      scrolling: Date.now() - perf.scrolledAt < 10000,
      source: top ? top[0] : "",
    });
    perf.count = 0; perf.total = 0; perf.max = 0; perf.sources.clear();
  }, 10000);

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
