// Stills of docs/mockups/survival.html and M33's Survival tab, headless Chromium.
// In a cloud session with no GPU this renders on SwiftShader (software); on the desktop use the GPU
// flags of CLAUDE.md ("Render on the GPU when there is one").
//   node tools/mockup_survival_shots.js [--gpu]
const { chromium } = require("playwright");
const path = require("path");
const ROOT = path.resolve(__dirname, "..");
const OUT = path.join(ROOT, "docs/screenshots/survival-mode/mockup");
const gpu = process.argv.includes("--gpu");
const onlyArg = process.argv.find((a) => a.startsWith("--only="));
const only = onlyArg ? onlyArg.slice(7).split(",") : null;
const want = (k) => !only || only.includes(k);
(async () => {
  const b = await chromium.launch({ args: gpu ? ["--use-angle=d3d11", "--enable-gpu", "--ignore-gpu-blocklist"] : ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"] });
  const p = await b.newPage({ viewport: { width: 1600, height: 900 } });
  p.setDefaultTimeout(300000);
  const errs = [];
  p.on("pageerror", (e) => errs.push(e.message));
  p.on("console", (m) => { if (m.type() === "error") errs.push(m.text()); });
  const url = "file://" + path.join(ROOT, "docs/mockups/survival.html");
  const shot = async (n) => { await p.waitForTimeout(400); await p.screenshot({ path: path.join(OUT, n + ".png") }); console.log(n); };
  // HUD
  await p.goto(url + "#hud"); await p.waitForTimeout(1500);
  if (want("hud")) for (const [t, s] of [["day", "fine"], ["night", "fine"], ["day", "hot"], ["day", "stroke"], ["night", "cold"], ["night", "under"], ["night", "winded"], ["day", "worn"], ["night", "dead"]]) {
    await p.evaluate(([t, s]) => window.hudTab.set(t, s), [t, s]); await shot(`hud-${t}-${s}`);
  }
  // Stamina
  await p.evaluate(() => window.survivalPage.show("stamina")); await p.waitForTimeout(500);
  if (want("stamina")) for (const k of ["sprint", "swim", "day"]) { await p.evaluate((k) => window.staminaTab.setCase(k), k); await shot(`stamina-${k}`); }
  // Fire
  if (want("fire")) {
  await p.evaluate(() => window.survivalPage.show("fire")); await p.waitForTimeout(3000);
  console.log(await p.evaluate(() => JSON.stringify(window.fireTab.still({ reset: true, ticks: 12, night: true, ft: 3.2 }))));
  await shot("fire-night-front");
  console.log(await p.evaluate(() => JSON.stringify(window.fireTab.still({ ticks: 30, view: { t: [-6, 2, 2], r: 34, pol: 1.05, az: 0.9 }, ft: 4.1 }))));
  await shot("fire-night-close");
  console.log(await p.evaluate(() => JSON.stringify(window.fireTab.still({ rain: 4, ticks: 6, view: { t: [-4, 0, 0], r: 92, pol: 0.95, az: 0.55 }, ft: 5, rainY: true }))));
  await shot("fire-night-rain");
  console.log(await p.evaluate(() => JSON.stringify(window.fireTab.still({ rain: 0, reset: true, ticks: 160, ft: 6 }))));
  await shot("fire-night-late");
  console.log(await p.evaluate(() => { let k = 0; while (k++ < 600 && window.fireTab.still({ ticks: 10 }).burning) {} return JSON.stringify(window.fireTab.still({ night: false })); }));
  await shot("fire-day-after");
  }
  // Weapons
  if (want("weapons")) {
  await p.evaluate(() => window.survivalPage.show("weapons")); await p.waitForTimeout(2500);
  for (const [n, o] of [["weapons-day-all", { night: false, look: "all" }], ["weapons-night-all", { night: true, look: "all", ft: 1.3 }],
    ["weapons-day-spear", { night: false, look: "spear" }], ["weapons-day-bow", { night: false, look: "bow" }],
    ["weapons-day-axe", { night: false, look: "axe" }], ["weapons-day-sword", { night: false, look: "sword" }], ["weapons-night-sword", { night: true, look: "sword", ft: 2.1 }]]) {
    await p.evaluate((o) => window.weaponsTab.still(o), o); await shot(n);
  }
  }
  // M33's Survival tab
  if (want("m33")) {
  await p.goto("file://" + path.join(ROOT, "docs/mockups/mission-tree.html#survival")); await p.waitForTimeout(1500);
  await shot("m33-survival-hunt");
  await p.evaluate(() => window.missionTree.fit()); await shot("m33-survival-fit");
  await p.evaluate(() => window.missionTree.focus("long-shot", 5)); await shot("m33-survival-hunting");
  await p.evaluate(() => window.missionTree.focus("cool-down", 5)); await shot("m33-survival-heat");
  }
  console.log(errs.length ? "ERRORS " + errs.join(" | ") : "no page errors");
  await b.close();
})();
