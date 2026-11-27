// Stills of docs/mockups/survival.html's combat drafts (survival-mode combat C1, C2; tasks 1.2, 1.6):
// the Weapons tab's In hand swings beside today's tool stroke, and the Hunt tab, by day and at night,
// headless Chromium at deviceScaleFactor 1. In a cloud session with no GPU this renders on
// SwiftShader (software); on the desktop pass --gpu (CLAUDE.md, "Render on the GPU when there is one").
//   node tools/mockup_survival_combat_shots.js [--gpu] [--only=swing,hunt]
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
  const p = await b.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
  p.setDefaultTimeout(300000);
  const errs = [];
  p.on("pageerror", (e) => errs.push(e.message));
  p.on("console", (m) => { if (m.type() === "error") errs.push(m.text()); });
  const url = "file://" + path.join(ROOT, "docs/mockups/survival.html");
  const shot = async (n) => { await p.waitForTimeout(300); await p.screenshot({ path: path.join(OUT, n + ".png") }); console.log(n); };
  if (want("swing")) {
    await p.goto(url + "#weapons"); await p.waitForTimeout(2500);
    const W = (o) => p.evaluate((o) => JSON.stringify(window.weaponsTab.still(o) && window.SurvivalSwing.set({})), o);
    // each proposed swing at its hit (the rule's moment), and at the top of its windup
    const at = (w) => p.evaluate((w) => { const S = window.SurvivalSwing, rel = S.P[w].kind === "draw" ? 0.9 : null, T = S.times(w, S.P[w], rel); return [T.w + 0.6 * T.a, rel == null ? T.hit + 0.01 : rel + 0.035]; }, w);
    for (const w of ["spear", "axe", "sword", "throw", "bow"]) {
      const [tTop, tHit] = await at(w);
      for (const [n, t] of [["top", w === "bow" ? 0.85 : tTop], ["hit", tHit]]) {
        console.log(await W({ view: "hand", night: false, swing: { weapon: w, motion: "proposed", playing: false, poses: false, clock: t } }));
        await shot(`swing-day-${w}-${n}`);
      }
    }
    // today's tool stroke, beside, at its top and its hit
    const [tt, th] = await at("today");
    for (const [n, t] of [["top", tt], ["hit", th]]) { await W({ view: "hand", night: false, swing: { motion: "today", clock: t } }); await shot(`swing-day-today-${n}`); }
    // by night, lit by the campfire and a lantern
    await W({ view: "hand", night: true, swing: { weapon: "sword", motion: "proposed", clock: (await at("sword"))[1] }, ft: 1.3 }); await shot("swing-night-sword-hit");
    await W({ view: "hand", night: true, swing: { weapon: "spear", motion: "proposed", clock: (await at("spear"))[1] }, ft: 2.1 }); await shot("swing-night-spear-hit");
    await W({ view: "hand", night: true, swing: { weapon: "bow", motion: "proposed", clock: 0.85 }, ft: 0.7 }); await shot("swing-night-bow-drawn");
  }
  if (want("hunt")) {
    await p.goto(url + "#hunt"); await p.waitForTimeout(2500);
    const Hn = async (n, o) => { console.log(n, JSON.stringify(await p.evaluate((o) => window.huntTab.still(o), o))); await shot(n); };
    const near = { t: [3.4, 0.45, 0.2], r: 6.5, pol: 1.2, az: -0.8 };
    const far = { t: [6, 0.4, 0.5], r: 10, pol: 1.15, az: -0.6 };
    for (const night of [false, true]) {
      const tod = night ? "night" : "day";
      for (const state of ["stalk", "warn", "attack", "hit", "flee"]) await Hn(`hunt-${tod}-fox-${state}`, { species: "red-fox", night, state, view: state === "stalk" || state === "flee" ? far : near, ft: 1.7 });
      await Hn(`hunt-${tod}-hound-warn`, { species: "dust-hound", night, state: "warn", view: near, ft: 0.9 });
    }
    await Hn("hunt-night-fox-close", { species: "red-fox", night: true, state: "warn", view: { t: [4.5, 0.3, 0.3], r: 4.2, pol: 1.2, az: -1.0 }, ft: 2.4 });
  }
  // a phone's width: the tabs and the bars wrap
  if (want("phone")) {
    const q = await b.newPage({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 1 });
    q.on("pageerror", (e) => errs.push(e.message));
    await q.goto(url + "#hunt"); await q.waitForTimeout(2500);
    await q.evaluate(() => window.huntTab.still({ species: "red-fox", night: true, state: "warn", ft: 1 }));
    await q.waitForTimeout(300); await q.screenshot({ path: path.join(OUT, "hunt-phone.png") }); console.log("hunt-phone");
  }
  if (errs.length) { console.log("ERRORS:\n" + errs.join("\n")); process.exitCode = 1; }
  await b.close();
})();
