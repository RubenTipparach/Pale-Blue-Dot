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
  const F = async (o) => console.log(await p.evaluate((o) => JSON.stringify(window.fireTab.still(o)), o));
  // the dry season (wildfire decision 12): a small grass fire at its front, lit by its own flames
  await F({ season: "dry", wind: 3, rain: 0, overlay: false, reset: true, ticks: 40, night: true, view: { t: [-22, 0, 0], r: 50, pol: 0.95, az: 3.7 }, ft: 3.2 });
  await shot("fire-night-front");
  await F({ view: { t: [-20, 1, -6], r: 26, pol: 1.05, az: 3.4 }, ft: 4.1 });
  await shot("fire-night-close");
  await F({ rain: 4, ticks: 4, view: { t: [-22, 0, 0], r: 50, pol: 0.95, az: 3.7 }, ft: 5, rainY: true });
  await shot("fire-night-rain");
  // the plants' dryness as an overlay, the same fire: green is wetter (banks, wood floor, pasture)
  await F({ rain: 0, reset: true, ticks: 40, overlay: true, night: false, view: { t: [-6, 0, 0], r: 88, pol: 0.9, az: 3.9 }, ft: 3 });
  await shot("fire-dryness");
  // spring: the same torch, nothing catches
  await F({ season: "spring", overlay: false, reset: true, ticks: 0, night: true, view: { t: [-24, 0, -2], r: 40, pol: 1.0, az: 3.7 }, ft: 0.6 });
  console.log(await p.evaluate(() => { for (let k = 0; k < 30; k++) window.fireTab.still({ ticks: 1 }); return JSON.stringify(window.fireTab.still({ torch: [-30, -2], ft: 0.6 })); }));
  await shot("fire-spring-green");
  // the dry season run to its end, then by day
  await F({ season: "dry", reset: true, ticks: 160, night: true, view: { t: [-10, 0, 0], r: 70, pol: 0.95, az: 3.8 }, ft: 6 });
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
