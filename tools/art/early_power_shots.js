// Stills of M33's POWER and MILLS tabs (docs/mockups/mission-tree.html#power, #mills; early-power) and
// the Survival tab after fishing and farming moved under it, headless Chromium at device scale 2.
// The page is served over a local http server, so the MILLS tab's textures load as they do when
// published. In a cloud session with no GPU this renders on SwiftShader (software); on the desktop use
// --gpu (CLAUDE.md, "Render on the GPU when there is one").
//   node tools/art/early_power_shots.js [--gpu] [--only=power,mills,survival,tabs]
const { chromium } = require("playwright");
const http = require("http");
const fs = require("fs");
const path = require("path");
const ROOT = path.resolve(__dirname, "../..");
const OUT = path.join(ROOT, "docs/screenshots/early-power");
const SV_OUT = path.join(ROOT, "docs/screenshots/survival-mode/mockup");
const gpu = process.argv.includes("--gpu");
const onlyArg = process.argv.find((a) => a.startsWith("--only="));
const only = onlyArg ? onlyArg.slice(7).split(",") : null;
const want = (k) => !only || only.includes(k);
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".png": "image/png", ".json": "application/json", ".css": "text/css" };

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const srv = http.createServer((q, r) => {
    const f = path.join(ROOT, "docs/mockups", decodeURIComponent(q.url.split("?")[0].split("#")[0]));
    if (!f.startsWith(path.join(ROOT, "docs/mockups")) || !fs.existsSync(f)) { r.writeHead(404); r.end(); return; }
    r.writeHead(200, { "content-type": TYPES[path.extname(f)] || "application/octet-stream" });
    fs.createReadStream(f).pipe(r);
  }).listen(0);
  const base = `http://127.0.0.1:${srv.address().port}/mission-tree.html`;
  const b = await chromium.launch({ args: gpu ? ["--use-angle=d3d11", "--enable-gpu", "--ignore-gpu-blocklist"] : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
  const ctx = await b.newContext({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 2 });
  const p = await ctx.newPage();
  p.setDefaultTimeout(300000);
  const errs = [];
  p.on("pageerror", (e) => errs.push(e.message));
  p.on("console", (m) => { if (m.type() === "error") errs.push(m.text()); });
  const shot = async (dir, n) => { await p.waitForTimeout(500); await p.screenshot({ path: path.join(dir, n + ".png") }); console.log(n); };
  const go = async (h) => { await p.goto("about:blank"); await p.goto(base + "#" + h); await p.waitForTimeout(1500); };

  if (want("power")) {
    await go("power");
    console.log(JSON.stringify(await p.evaluate(() => ({ tree: window.missionTree.tree, missions: window.missionTree.missions, steps: window.missionTree.steps }))));
    await shot(OUT, "power-line");
    await p.evaluate(() => window.missionTree.fit()); await shot(OUT, "power-fit");
    await p.evaluate(() => { window.missionTree.select("windmill"); window.missionTree.focus("windmill", 7); }); await shot(OUT, "power-windmill-selected");
    await p.evaluate(() => { window.missionTree.select("line"); window.missionTree.focus("line", 7); }); await shot(OUT, "power-line-selected");
    await p.click("#recipesBtn"); await shot(OUT, "power-recipes");
    await p.evaluate(() => { window.missionTree.setStep(window.missionTree.steps); window.missionTree.select(null); window.missionTree.fit(); }); await shot(OUT, "power-done-fit");
  }
  if (want("mills")) {
    await go("mills");
    await p.waitForFunction(() => window.millsTab && window.millsTab.ready());
    await p.waitForTimeout(2500);
    await shot(OUT, "mills-night");
    await p.evaluate(() => window.millsTab.still({ st: { night: false } })); await p.waitForTimeout(800); await shot(OUT, "mills-day");
    await p.evaluate(() => window.millsTab.still({ st: { night: true }, az: 0.35, pol: 1.2, r: 24, t: [3, 2.5, 1] })); await shot(OUT, "mills-night-watermill");
    await p.evaluate(() => window.millsTab.still({ st: { night: true, sluice: false } })); await shot(OUT, "mills-night-sluice-shut");
    await p.evaluate(() => window.millsTab.still({ st: { night: false, sluice: true }, az: -0.9, pol: 1.15, r: 22, t: [-14, 7, -4] })); await shot(OUT, "mills-day-windmill");
    await p.evaluate(() => window.millsTab.still({ st: { night: true }, az: -0.9, pol: 1.15, r: 22, t: [-14, 7, -4] })); await shot(OUT, "mills-night-windmill");
    await p.evaluate(() => window.millsTab.still({ st: { night: false }, az: 0.6, pol: 1.25, r: 9, t: [6.2, 2, 0] })); await shot(OUT, "mills-day-wheel-close");
  }
  if (want("survival")) {
    await go("survival");
    await shot(SV_OUT, "m33-survival-hunt");
    await p.evaluate(() => window.missionTree.fit()); await shot(SV_OUT, "m33-survival-fit");
    await p.evaluate(() => window.missionTree.focus("long-shot", 5)); await shot(SV_OUT, "m33-survival-hunting");
    await p.evaluate(() => window.missionTree.focus("cool-down", 5)); await shot(SV_OUT, "m33-survival-heat");
    await p.evaluate(() => { window.missionTree.select("rod"); window.missionTree.focus("rod", 5); }); await shot(SV_OUT, "m33-survival-fishing-farming");
  }
  if (want("tabs")) {
    for (const h of ["starter", "town", "crew"]) { await go(h); await shot(OUT, "tab-" + h); }
  }
  console.log(errs.length ? "ERRORS " + errs.join(" | ") : "no page errors");
  await b.close();
  srv.close();
})();
