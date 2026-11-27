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
  if (want("hud")) for (const [t, s] of [["day", "full"], ["night", "full"], ["day", "fine"], ["night", "fine"], ["day", "hungry"], ["night", "hungry"], ["day", "pack"], ["night", "pack"],
    ["day", "hot"], ["day", "exhaust"], ["night", "exhaust"], ["day", "stroke"], ["night", "cold"], ["night", "under"], ["night", "winded"], ["day", "worn"], ["night", "dead"]]) {
    await p.evaluate(([t, s]) => window.hudTab.set(t, s), [t, s]); await shot(`hud-${t}-${s}`);
  }
  // Stamina
  await p.evaluate(() => window.survivalPage.show("stamina")); await p.waitForTimeout(500);
  if (want("stamina")) {
    for (const k of ["sprint", "swim", "day"]) { await p.evaluate((k) => window.staminaTab.setCase(k), k); await shot(`stamina-${k}`); }
    // the model in action, behind Expand (survey G6)
    await p.evaluate(() => window.staminaTab.setCase("sprint"));
    for (const [n, o] of [["stamina-anim-sprint-day", { t: 15, night: false }], ["stamina-anim-swim-day", { t: 38, night: false }], ["stamina-anim-sprint-night", { t: 15, night: true }],
      ["stamina-anim-swim-night", { t: 38, night: true }], ["stamina-anim-climb-night", { t: 52.6, night: true }], ["stamina-anim-rest-night", { t: 62, night: true }]]) {
      console.log(JSON.stringify(await p.evaluate((o) => window.staminaAnim.at(o), o))); await shot(n);
    }
  }
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
  // M33's Survival tab, version 11: the starter trunk to the fire, then the path picker
  // (survival-mode design 18), at deviceScaleFactor 2
  if (want("m33")) {
  const q = await b.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 2 });
  q.setDefaultTimeout(300000);
  q.on("pageerror", (e) => errs.push(e.message));
  const qs = async (n) => { await q.waitForTimeout(700); await q.screenshot({ path: path.join(OUT, n + ".png") }); console.log(n, JSON.stringify(await q.evaluate(() => window.missionTree.state()))); };
  await q.goto("file://" + path.join(ROOT, "docs/mockups/mission-tree.html#survival")); await q.waitForTimeout(1500);
  const T = (f, a) => q.evaluate(f, a);
  // the tree up to the fire: the trunk done, the paths open
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(0) - 1); m.focus("fire", 4); });
  await qs("m33-survival-v11-fire");
  // the whole tree at the fire
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(0) - 1); m.fit(); });
  await qs("m33-survival-v11-fit");
  // the salt water note on the way to the fire (a coastal start)
  await T(() => { const m = window.missionTree; m.setStep(m.stepList().findIndex((x) => x.startsWith("note:Tried the sea")) + 1); m.focus("planks-rope", 6); });
  await qs("m33-survival-v11-salt");
  // the picker as the fire is lit, the bot's choice marked
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(0)); m.fit(); });
  await qs("m33-survival-v11-picker");
  // a click on Eat and drink: the tree pans to that path, its first mission Now
  await T(() => window.missionTree.pick("eat-drink"));
  await qs("m33-survival-v11-picked");
  // the bot partway along the path: cactus juice, two sips of three... the straw made
  await T(() => { const m = window.missionTree; const i = m.stepList().findIndex((x) => x === "obj:cactus"); m.setStep(i + 1); m.focusPath("eat-drink"); });
  await qs("m33-survival-v11-along");
  // the path done: the picker again with what is left
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(1)); });
  await qs("m33-survival-v11-reoffer");
  // later: Hunting waits on Smelt bronze, shown on its card
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(2)); });
  await qs("m33-survival-v11-waits");
  // the viewer picks another path than the bot's: Hunting first
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(0)); m.pick("hunting"); });
  await qs("m33-survival-v11-hunting");
  // the picker at phone width
  await q.setViewportSize({ width: 390, height: 844 });
  await q.goto("file://" + path.join(ROOT, "docs/mockups/mission-tree.html?phone#survival")); await q.waitForTimeout(1500);
  await T(() => { const m = window.missionTree; m.setStep(m.pickAt(0)); m.fit(); });
  await qs("m33-survival-v11-picker-phone");
  await q.close();
  }
  console.log(errs.length ? "ERRORS " + errs.join(" | ") : "no page errors");
  await b.close();
})();
