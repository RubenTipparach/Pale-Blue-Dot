// The towns mockup's Wide roads scene (`makeWideRoads`, townsfolk design
// 10b): each of its places by day (11:00) and at 22:30, the whole scene from
// above, and walks across every lane up and down the wide climbs and round
// the bends that count the eye's jumps (there must be none).
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g)
// node tools/mockup_wide_roads_shots.js <towns.html> <out dir> [--no-shots]
// SwiftShader (software GL): for a container with no GPU only (CLAUDE.md).
const fs = require("fs"); const { chromium } = require("playwright");
const CLOSE = [["bends", { tx: 2.833 * 42, ty: 11, tz: 2.4535 * 9, dist: 52, yaw: -0.4, pitch: 0.95 }], ["climb", { tx: 2.833 * 32, ty: 6, tz: 2.4535 * 8, dist: 50, yaw: -1.25, pitch: 0.42 }]];
const SPOTS = ["parallel", "lanes", "climbfoot", "climbing", "cutfill", "bend3", "bend2", "junction", "bridge2", "bridge3"];
(async () => {
  const [html, out, flag] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file://${html}#wideroads`);
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "wideroads", null, { timeout: 120000 });
  await page.waitForTimeout(1500);
  // The walks: the walker pressing forward, turned toward each waypoint in turn.
  const walks = await page.evaluate(() => {
    const T = window.__town, res = {};
    for (const [name, pts] of Object.entries(WIDE_WALKS)) {
      T.teleport({ x: pts[0][0], z: pts[0][1], y: 12 });
      T.P.jolts.length = 0;
      let maxRise = 0, maxDrop = 0, prev = T.P.feet, ticks = 0, lo = T.P.feet, hi = T.P.feet, missed = 0;
      for (let k = 1; k < pts.length; k++) {
        const [tx, tz] = pts[k]; let n = 0;
        for (; n < 1200; n++) {
          T.P.yaw = Math.atan2(tz - T.P.z, tx - T.P.x);
          T.tick(1 / 120, { fwd: 1, side: 0, jump: false, sprint: false, down: false });
          const d = T.P.feet - prev; maxRise = Math.max(maxRise, d); maxDrop = Math.min(maxDrop, d); prev = T.P.feet; ticks++;
          lo = Math.min(lo, T.P.feet); hi = Math.max(hi, T.P.feet);
          if (Math.hypot(tx - T.P.x, tz - T.P.z) < 0.3) break;
        }
        if (n === 1200) missed++;
      }
      res[name] = { jolts: T.P.jolts.length, maxRisePerTick: +maxRise.toFixed(3), maxDropPerTick: +maxDrop.toFixed(3), feet: [+lo.toFixed(2), +hi.toFixed(2)], ticks, waypointsMissed: missed };
    }
    // the beds the shared plane needs, by lane: in step, or half a cell along
    const beds = {};
    for (const w of WIDE_CELLS) { const k = `bed ${w.bed0.toFixed(1)}-${w.bed1.toFixed(1)} m, cut ${w.ground - w.lo}`; beds[k] = (beds[k] || 0) + 1; }
    // the largest step between two neighbouring road surfaces along any shared edge
    let worst = 0;
    const near = [], iv = [];
    for (const w of WIDE_CELLS) for (let d = 0; d < 6; d++) {
      const [A, B] = edgeEnds(w.c, w.r, d);
      for (const t of [0.1, 0.5, 0.9]) {
        const x = A[0] + (B[0] - A[0]) * t, z = A[1] + (B[1] - A[1]) * t, [nx, nz] = dvec(d);
        const top = (px, pz) => { nearSolids(px, pz, 0.5, near); intervalsAt(px, pz, iv, near); return Math.max(...iv.filter(v => v[2] !== "edge").map(v => v[1])); };
        const a = top(x - nx * 0.001, z - nz * 0.001), b = top(x + nx * 0.001, z + nz * 0.001);
        const [c2, r2] = cellAt(x + nx * 0.001, z + nz * 0.001);
        if (inMap(c2, r2) && WIDE.has(idx(c2, r2))) worst = Math.max(worst, Math.abs(a - b));
      }
    }
    return { walks: res, beds, worstStepBetweenLanes_m: +worst.toFixed(4), slopedCells: WIDE_CELLS.length };
  });
  console.log(JSON.stringify(walks, null, 1));
  if (flag !== "--no-shots") {
    const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; }, h);
    const shot = async (name, fn, arg) => { await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 85 }); };
    for (const [h, tag] of [[11, "11h00"], [22.5, "22h30"]]) {
      await at(h);
      for (const spot of SPOTS) await shot(`mockup-wide-${spot}-${tag}`, (s) => { teleport(GOTO[s]); setView("walk"); }, spot);
      await shot(`mockup-wide-overview-${tag}`, () => { Object.assign(ORB, { ty: 2 }, SCENES.wideroads.orbit); setView("overview"); });
      // close views from above: the two bends on the plateau, and the climbs from the side
      for (const [name, o] of CLOSE) await shot(`mockup-wide-${name}-above-${tag}`, (o) => { Object.assign(ORB, o); setView("overview"); }, o);
    }
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
