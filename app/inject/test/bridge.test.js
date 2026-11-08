// bridge.js davranış testi: web.whatsapp.com'u taklit eden sahte sayfa üzerinde Chromium ile.
// Çalıştırma: node app/inject/test/bridge.test.js
const { chromium } = require("playwright");
const fs = require("fs");
const path = require("path");
const assert = require("assert");

const bridge = fs.readFileSync(path.join(__dirname, "..", "bridge.js"), "utf8");
const page_html = `<!doctype html><html><head><title>WhatsApp</title></head><body>
<div id="app" style="height:700px">
  <div id="side" style="height:700px">
    <div id="banner" style="height:56px"><div><div><span>Windows için WhatsApp'ı indirin</span></div><button aria-label="Kapat" onclick="window.__closed = true">x</button></div></div>
    <div id="chats" style="height:600px"><span>Sohbet 1</span></div>
  </div>
</div></body></html>`;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.route("https://web.whatsapp.com/**", (r) => r.fulfill({ contentType: "text/html; charset=utf-8", body: page_html }));
  await page.addInitScript(() => {
    window.__calls = [];
    window.__TAURI_INTERNALS__ = { invoke: (cmd, args) => { window.__calls.push([cmd, args]); return Promise.resolve(); } };
  });
  await page.addInitScript(bridge);
  await page.goto("https://web.whatsapp.com/");

  // 1) İndirme çubuğu gizlenir, sohbet listesi görünür kalır
  await page.waitForFunction(() => getComputedStyle(document.getElementById("banner")).display === "none", null, { timeout: 5000 });
  assert.notStrictEqual(await page.evaluate(() => getComputedStyle(document.getElementById("chats")).display), "none");
  assert.notStrictEqual(await page.evaluate(() => getComputedStyle(document.getElementById("side")).display), "none");
  assert.strictEqual(await page.evaluate(() => window.__closed === true), true, "WhatsApp'ın kapatma düğmesine basılmalı (kalıcı kapatma)");

  // 2) Başlık değişince set_unread hemen (gözlemciyle) gönderilir
  await page.evaluate(() => { window.__calls = []; document.title = "(3) WhatsApp"; });
  await page.waitForFunction(() => window.__calls.some(([c, a]) => c === "set_unread" && a.title === "(3) WhatsApp"), null, { timeout: 1000 });

  // 3) Notification taklidi: izin "granted", notify çağrısı, tıklama onclick'i tetikler
  const r = await page.evaluate(async () => {
    window.__calls = [];
    const perm = Notification.permission;
    const n = new Notification("Ali", { body: "selam" });
    let clicked = false;
    n.onclick = () => { clicked = true; };
    const call = window.__calls.find(([c]) => c === "notify");
    window.__wrs.click(call[1].id);
    return { perm, call: call[1], clicked };
  });
  assert.strictEqual(r.perm, "granted");
  assert.strictEqual(r.call.title, "Ali");
  assert.strictEqual(r.call.body, "selam");
  assert.ok(r.clicked, "bildirime tıklama onclick'i tetiklemeli");

  // 4) Başlık izleyicisi <head> içindeki başka değişikliklerde tetiklenmez (yalnızca <title>)
  const fired = await page.evaluate(async () => {
    const desc = Object.getOwnPropertyDescriptor(Document.prototype, "title");
    let reads = 0;
    Object.defineProperty(document, "title", { configurable: true, get() { reads++; return desc.get.call(this); }, set(v) { desc.set.call(this, v); } });
    for (let i = 0; i < 200; i++) {
      const st = document.createElement("style");
      st.textContent = ".x" + i + "{color:red}";
      document.head.appendChild(st);
    }
    await new Promise((r) => setTimeout(r, 50));
    delete document.title;
    return reads;
  });
  assert.strictEqual(fired, 0, "<head> içindeki stil eklemeleri başlık izleyicisini çalıştırmamalı");

  await browser.close();
  console.log("bridge.js: 4/4 senaryo geçti");
})().catch((e) => { console.error(e); process.exit(1); });
