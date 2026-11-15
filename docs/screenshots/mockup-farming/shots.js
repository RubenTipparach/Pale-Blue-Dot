// Before/after views of the farming mockup (M8), for the 2026-11-15 art-style pass.
//   THREE_JS=<three.min.js r128> [PAGE_HTML=<a farming.html to serve instead>] node docs/screenshots/mockup-farming/shots.js <out dir> <prefix>
// Serves docs/mockups/ from a fake origin with page.route; headless Chromium on SwiftShader (no GPU).
// Views: the fields (default orbit), the greenhouse close up, both at 10:00, and the fields at 22:00.
// Checks: no console errors, one WebGL context, the camera holds still while idle (the penned
// animals walk, as the world moves them), the night is lit, no horizontal scroll at 390 px.
const fs = require("fs"), path = require("path"); const { chromium } = require("playwright");
const ROOT = path.resolve(__dirname, "../../mockups");
const TYPES = { ".html": "text/html", ".png": "image/png", ".js": "text/javascript" };
(async () => {
  const [out, prefix = ""] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"] });
  const run = async (w, h) => {
    const page = await browser.newPage({ viewport: { width: w, height: h } });
    const errors = []; page.on("pageerror", e => errors.push(String(e))); page.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
    await page.addInitScript(() => { window.__gl = 0; const g = HTMLCanvasElement.prototype.getContext; HTMLCanvasElement.prototype.getContext = function (t, ...a) { const c = g.call(this, t, ...a); if (c && /webgl/.test(t) && !this.__counted) { this.__counted = 1; window.__gl++; } return c; }; });
    await page.route("**/three.min.js", r => r.fulfill({ body: fs.readFileSync(process.env.THREE_JS), contentType: "text/javascript" }));
    await page.route("http://mock.local/**", r => {
      const rel = decodeURIComponent(new URL(r.request().url()).pathname);
      const p = rel === "/farming.html" && process.env.PAGE_HTML ? process.env.PAGE_HTML : path.join(ROOT, rel);
      if (!fs.existsSync(p)) return r.fulfill({ status: 404, body: "" });
      r.fulfill({ body: fs.readFileSync(p), contentType: TYPES[path.extname(p)] || "application/octet-stream" });
    });
    await page.goto("http://mock.local/farming.html");
    await page.waitForTimeout(4000);
    return { page, errors };
  };
  const { page, errors } = await run(1280, 900);
  const stage = await page.$("#stage");
  const shot = async (name, js) => { await page.evaluate(js); await page.waitForTimeout(1500); await stage.screenshot({ path: `${out}/${prefix}${name}.png` }); };
  await shot("fields", () => { setHour(10); orbit.yaw = 0.5; orbit.pitch = 0.72; orbit.dist = 30; orbit.target.set(0, 0, 0); });
  await shot("fields-close", () => { orbit.yaw = 0.9; orbit.pitch = 0.42; orbit.dist = 11; orbit.target.set(-2, 0, 3); });
  await shot("greenhouse", () => { const [x, z] = hexXZ(GH.q, GH.r); orbit.yaw = 0.4; orbit.pitch = 0.5; orbit.dist = 10; orbit.target.set(x, 0.6, z); });
  await shot("fields-night", () => { setHour(22); orbit.yaw = 0.5; orbit.pitch = 0.72; orbit.dist = 30; orbit.target.set(0, 0, 0); });
  const idle = await page.evaluate(async () => { const a = camera.position.toArray(); await new Promise(r => setTimeout(r, 1500)); const b = camera.position.toArray(); return { camMoved: a.some((v, i) => Math.abs(v - b[i]) > 1e-6) }; });
  const gl = await page.evaluate(() => window.__gl);
  await page.close();
  const ph = await run(390, 844);
  const scroll = await ph.page.evaluate(() => ({ sw: document.documentElement.scrollWidth, cw: document.documentElement.clientWidth }));
  await ph.page.screenshot({ path: `${out}/${prefix}phone-390.png` });
  let modal = null;
  if (await ph.page.$("#openNotes")) { await ph.page.click("#openNotes"); await ph.page.waitForTimeout(300); modal = await ph.page.evaluate(() => ({ open: !document.getElementById("notes").hidden, sw: document.documentElement.scrollWidth })); await ph.page.screenshot({ path: `${out}/${prefix}phone-390-notes.png` }); await ph.page.keyboard.press("Escape"); modal.closedByEsc = await ph.page.evaluate(() => document.getElementById("notes").hidden); }
  console.log(JSON.stringify({ webglContexts: gl, idle, phone: scroll, modal, errors: errors.concat(ph.errors) }, null, 1));
  await browser.close();
})();
