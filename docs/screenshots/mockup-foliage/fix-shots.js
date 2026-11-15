// The 2026-11-15 fix pass on M13 (fallen leaves, no painted flowers, seasonless impostor bake, savanna and fungal
// palettes, fruit on the wood): THREE_JS=<three.min.js r128> node docs/screenshots/mockup-foliage/fix-shots.js <out dir> <prefix> [only]
// Headless Chromium on SwiftShader (no GPU): what things look like, never how fast.
const fs = require("fs"), path = require("path"); const { chromium } = require("playwright");
const ROOT = path.resolve(__dirname, "../../mockups"), THREE_JS = process.env.THREE_JS;
const TYPES = { ".html": "text/html", ".png": "image/png", ".js": "text/javascript" };
const [out, prefix = "", only] = process.argv.slice(2);
(async () => {
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist"] });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const errors = []; page.on("pageerror", e => { errors.push(String(e)); console.log("PAGEERR", String(e).slice(0,400)); }); page.on("console", m => { if (m.type() === "error") { errors.push(m.text()); console.log("CONSOLE", m.text().slice(0,1500)); } });
  await page.route("**/three.min.js", r => r.fulfill({ body: fs.readFileSync(THREE_JS), contentType: "text/javascript" }));
  await page.route("http://mock.local/**", r => { const p = path.join(ROOT, decodeURIComponent(new URL(r.request().url()).pathname)); if (!fs.existsSync(p)) return r.fulfill({ status: 404, body: "" }); r.fulfill({ body: fs.readFileSync(p), contentType: TYPES[path.extname(p)] || "application/octet-stream" }); });
  await page.goto("http://mock.local/foliage.html");
  await page.waitForFunction(() => document.getElementById("loading").hidden, null, { timeout: 120000 });
  await page.waitForTimeout(1500);
  await page.evaluate(() => { for (const s of document.querySelectorAll(".panel")) s.style.visibility = "hidden"; window.__near = (key, f) => { const L = LAYERS.find(l => l.key === key); return L && L.objs.filter(f || (() => true)).sort((a, b) => Math.hypot(a.x, a.z) - Math.hypot(b.x, b.z))[0]; }; window.__cam = (o) => { setCamMode("orbit"); delete CAM.tyOff; Object.assign(CAM, o); }; });
  const shot = async (name, fn, arg, wait = 3000) => {
    if (only && !name.includes(only)) return;
    await page.evaluate(fn, arg); await page.waitForTimeout(wait);
    await page.screenshot({ path: `${out}/${prefix}${name}.png` }); console.log("shot", name);
  };
  const setB = (b, se, t = 12) => `S.biome="${b}"; S.season="${se}"; S.time=${t}; syncUI(); rebuild();`;
  // fallen leaves at 0 / 50 / 100 %
  for (const f of [0, 0.5, 1]) await shot("leaves-" + Math.round(f * 100), new Function("f", setB("temperate", "autumn", 13) + "S.leafFall=f; applyLive(); const t=__near('trees'); __cam({tx:t.x+2,tz:t.z+2,yaw:0.8,pitch:0.62,dist:15});"), f, 4000);
  // ground close-up, plants off, per biome
  for (const b of ["temperate", "jungle", "swamp", "taiga"]) await shot("ground-" + b, new Function(`for (const k in S.layers) S.layers[k].on = false; ${setB(b, "summer", 12)} const c = CELLS.filter(c=>!c.cold&&!c.wet&&!c.path).sort((a,b)=>Math.hypot(a.x,a.z)-Math.hypot(b.x,b.z))[0]; __cam({tx:c.x,tz:c.z,yaw:0.3,pitch:1.25,dist:3.2});`), null, 3000);
  await page.evaluate(() => { for (const k in S.layers) S.layers[k].on = true; });
  // summer orbit after autumn (provoke: autumn, rebuild via biome round trip, back to summer)
  await shot("summer-after-autumn", () => { S.biome = "temperate"; S.season = "autumn"; S.time = 13; syncUI(); rebuild(); return new Promise(r => setTimeout(() => { S.season = "summer"; syncUI(); __cam({ tx: 0, tz: 0, yaw: 0.8, pitch: 0.42, dist: 70 }); r(); }, 2500)); }, null, 4000);
  // savanna
  await shot("savanna", new Function(setB("savanna", "summer", 13) + "const t=__near('trees'); __cam({tx:t.x,tz:t.z,yaw:0.8,pitch:0.3,dist:14});"), null, 4000);
  await shot("savanna-wide", () => { __cam({ tx: 0, tz: 0, yaw: 0.8, pitch: 0.42, dist: 32 }); }, null, 3000);
  // fungal by day and at night
  await shot("fungal-day", new Function(setB("fungal", "summer", 13) + "__cam({tx:0,tz:0,yaw:0.8,pitch:0.42,dist:26});"), null, 4000);
  await shot("fungal-night", () => { S.time = 22; syncUI(); }, null, 3000);
  // taiga pines with cones
  await shot("taiga-cones", new Function(setB("taiga", "autumn", 13) + "const f=__near('fruit'); __cam({tx:f.x,tz:f.z,yaw:0.8,pitch:0.15,dist:6, tyOff:f.y-groundAt(f.x,f.z)-0.25});"), null, 4000);
  await shot("taiga-cones2", new Function("const t=__near('trees', o=>o.sp==='pine' && LAYERS.find(l=>l.key==='fruit').objs.some(f=>Math.hypot(f.x-o.x,f.z-o.z)<3)); __cam({tx:t.x,tz:t.z,yaw:2.2,pitch:0.1,dist:10, tyOff:4.5});"), null, 4000);
  console.log(JSON.stringify({ errors }, null, 1));
  await browser.close();
})();
