// Before/after views of the foliage mockup (M13), for the 2026-11-15 art-style pass.
//   THREE_JS=<three.min.js r128> node docs/screenshots/mockup-foliage/shots.js <out dir> <prefix>
// Serves docs/mockups/ from a fake origin with page.route, renders in headless Chromium on
// SwiftShader (no GPU), and checks: no console errors, one WebGL context, nothing moves while idle
// (two orbit frames 1.5 s apart, wind off, must match), and the night is lit (lamp pools brighter
// than the dark ground). Views: temperate, savanna, fungal at the default orbit and at Walk height.
const fs = require("fs"), path = require("path"); const { chromium } = require("playwright");
const ROOT = path.resolve(__dirname, "../../mockups");
const TYPES = { ".html": "text/html", ".png": "image/png", ".js": "text/javascript", ".json": "application/json" };
(async () => {
  const [out, prefix = "", only] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"] });
  const run = async (w, h) => {
    const page = await browser.newPage({ viewport: { width: w, height: h } });
    const errors = []; page.on("pageerror", e => errors.push(String(e))); page.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
    await page.addInitScript(() => { window.__gl = 0; const g = HTMLCanvasElement.prototype.getContext; HTMLCanvasElement.prototype.getContext = function (t, ...a) { const c = g.call(this, t, ...a); if (c && /webgl/.test(t) && !this.__counted) { this.__counted = 1; window.__gl++; } return c; }; });
    await page.route("**/three.min.js", r => r.fulfill({ body: fs.readFileSync(process.env.THREE_JS), contentType: "text/javascript" }));
    await page.route("http://mock.local/**", r => { const p = path.join(ROOT, decodeURIComponent(new URL(r.request().url()).pathname)); if (!fs.existsSync(p)) return r.fulfill({ status: 404, body: "" }); r.fulfill({ body: fs.readFileSync(p), contentType: TYPES[path.extname(p)] || "application/octet-stream" }); });
    await page.goto("http://mock.local/foliage.html");
    await page.waitForFunction(() => document.getElementById("loading").hidden, null, { timeout: 120000 });
    await page.waitForTimeout(1500);
    return { page, errors };
  };
  const { page, errors } = await run(1280, 800);
  await page.evaluate(() => { for (const s of document.querySelectorAll(".panel")) s.style.visibility = "hidden"; });
  const settle = async () => { await page.waitForTimeout(2500); };
  const views = [];
  for (const biome of ["temperate", "savanna", "fungal"]) {
    if (only && biome !== only) continue;
    await page.evaluate(b => { setCamMode("orbit"); Object.assign(CAM, { tx: 0, tz: 0, yaw: 0.8, pitch: 0.42, dist: 32 }); S.biome = b; syncUI(); rebuild(); }, biome);
    await settle(); await page.screenshot({ path: `${out}/${prefix}${biome}-orbit.png` }); views.push(`${prefix}${biome}-orbit.png`);
    await page.evaluate(() => { setCamMode("walk"); }); await settle();
    await page.screenshot({ path: `${out}/${prefix}${biome}-walk.png` }); views.push(`${prefix}${biome}-walk.png`);
  }
  // night: temperate at 22:00, default orbit
  await page.evaluate(() => { setCamMode("orbit"); Object.assign(CAM, { tx: 0, tz: 0, yaw: 0.8, pitch: 0.42, dist: 32 }); S.biome = "temperate"; S.time = 22; syncUI(); rebuild(); });
  await settle(); await page.screenshot({ path: `${out}/${prefix}temperate-night.png` });
  // idle: wind off, two frames 1.5 s apart must match (fireflies off for the check)
  const idle = await page.evaluate(async () => {
    S.time = 12; S.wind = 0; S.gust = 0; S.layers.fireflies.on = false; syncUI(); rebuild();
    await new Promise(r => setTimeout(r, 3000));
    const grab = () => { renderer.render(scene, camera); return renderer.domElement.toDataURL(); };
    const a = grab(), p0 = camera.position.toArray(); await new Promise(r => setTimeout(r, 1500)); const b = grab(), p1 = camera.position.toArray();
    return { same: a === b, camMoved: p0.some((v, i) => Math.abs(v - p1[i]) > 1e-6) };
  });
  const gl = await page.evaluate(() => window.__gl);
  const grass = await page.evaluate(() => S.grassStyle);
  await page.close();
  // phone width
  const ph = await run(390, 844);
  const scroll = await ph.page.evaluate(() => ({ sw: document.documentElement.scrollWidth, cw: document.documentElement.clientWidth }));
  await ph.page.screenshot({ path: `${out}/${prefix}phone-390.png` });
  console.log(JSON.stringify({ views, grassDefault: grass, webglContexts: gl, idle, phone: scroll, errors: errors.concat(ph.errors) }, null, 1));
  await browser.close();
})();
