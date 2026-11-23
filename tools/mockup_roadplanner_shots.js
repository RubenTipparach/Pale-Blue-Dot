// The towns mockup's Road planner scene (`makeRoadPlanner`, townsfolk design
// 42, M15 version 5): the open plan's live ghost with a refused tree, a
// steep bank and a town, the bridge leg, the width menu and a three-wide
// ghost, paving with Keep paving, the edit mid-build demo, by day (11:00) and
// at 22:30, at desktop width and at phone width.
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g)
// node tools/mockup_roadplanner_shots.js <towns.html> <out dir>
// SwiftShader (software GL): for a container with no GPU only (CLAUDE.md).
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const body = fs.readFileSync(process.env.THREE_JS);
  const errors = [];
  for (const [tag, vp] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const page = await browser.newPage({ viewport: vp, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => errors.push(String(e)));
    page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
    await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
    await page.goto(`file://${html}#roadplanner`);
    await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "roadplanner", null, { timeout: 120000 });
    await page.waitForTimeout(1500);
    // helpers in the page: stand somewhere and look at a cell; aim the
    // overview's mouse at a cell
    await page.evaluate(() => {
      window.__stand = (c, r, dx, dz, tc, tr, lift = 0) => {
        const T = window.__town; T.teleport({ x: cx(c, r) + dx, z: cz(c, r) + dz, y: T.TOP[idx(c, r)] + 0.3 }); T.setView("walk");
        const P = T.P, ex = cx(tc, tr), ez = cz(tc, tr), ey = T.TOP[idx(tc, tr)] + lift, d = Math.hypot(ex - P.x, ez - P.z);
        P.yaw = Math.atan2(ez - P.z, ex - P.x); P.pitch = Math.atan2(ey - (P.feet + 1.6), d);
      };
      window.__orbit = (o) => { const T = window.__town; Object.assign(T.ORB, { ty: 2 }, o); T.setView("overview"); };
      window.__mouseAt = (c, r) => { const T = window.__town, v = new THREE.Vector3(cx(c, r), T.TOP[idx(c, r)], cz(c, r)); camera.updateMatrixWorld(); v.project(camera); const rc = document.getElementById("view").getBoundingClientRect(); T.rp.RP.mouse = [rc.left + (v.x + 1) / 2 * rc.width, rc.top + (1 - v.y) / 2 * rc.height]; };
      window.__night = (h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); };
    });
    const settle = async (ms = 900) => { await page.waitForTimeout(250); await page.evaluate(() => { __town.rp.rpUpdateAim(true); assignLights(); document.getElementById("hint").hidden = true; }); await page.waitForTimeout(ms); };
    const shot = async (name) => { await page.screenshot({ path: `${out}/mockup-roadplanner-${name}-${tag}.jpg`, type: "jpeg", quality: 85 }); };
    const phone = tag === "phone";
    for (const [h, t] of [[11, "11h00"], [22.5, "22h30"]]) {
      await page.evaluate((h) => { __town.rp.rpReset(true); __night(h); }, h);
      // the open end: the ghost to the aim runs past a tree, red on its cell
      await page.evaluate(() => __stand(31, 17, -0.3, 0.6, 35, 10));
      await settle(); await shot(`open-tree-${t}`);
      // the whole plan from above, the ghost aimed up to the hamlet
      await page.evaluate((p) => { __orbit(p ? { tx: 2.833 * 20, tz: 2.4535 * 14, dist: 95, yaw: -1.5, pitch: 0.95 } : { tx: 2.833 * 24, tz: 2.4535 * 14, dist: 74, yaw: -1.35, pitch: 0.8 }); }, phone);
      await page.waitForTimeout(300); await page.evaluate(() => __mouseAt(35, 10));
      if (phone) await page.evaluate(() => { if (document.getElementById("rp").dataset.open !== "true") document.getElementById("rpHide").click(); });
      await settle(); await shot(`overview-${t}`);
      if (phone) await page.evaluate(() => document.getElementById("rpHide").click());
      if (phone && t === "22h30") continue;
      // too steep: aimed down the bank's two layers
      await page.evaluate(() => __stand(31, 19, -0.5, -0.8, 32, 24));
      await settle(); await shot(`refuse-steep-${t}`);
      await page.evaluate(() => { __orbit({ tx: 2.833 * 30, tz: 2.4535 * 20, dist: 44, yaw: -1.2, pitch: 0.9 }); }); await page.waitForTimeout(400); await page.evaluate(() => __mouseAt(32, 24));
      await settle(); await shot(`refuse-steep-overview-${t}`);
      // a stake picked up: its two legs follow the aim
      await page.evaluate(() => { const T = __town.rp; T.RP.grab = { i: 2, orig: [23, 16], insert: false }; __orbit({ tx: 2.833 * 25, tz: 2.4535 * 14, dist: 54, yaw: -1.3, pitch: 0.95 }); }); await page.waitForTimeout(400); await page.evaluate(() => __mouseAt(25, 12));
      await settle(); await shot(`grab-overview-${t}`);
      await page.evaluate(() => __town.rp.rpCancelGrab());
      // a town's footprint
      await page.evaluate(() => __stand(33, 14, 0, 0, 39, 8));
      await settle(); await shot(`refuse-town-${t}`);
      // the bridge leg: ghost deck, posts and rails over the deep brook
      await page.evaluate(() => { __town.rp.rpSetTool("shovel"); __stand(16, 16, -0.6, 1.2, 21, 16); });
      await settle(); await shot(`bridge-${t}`);
      // width 3 from the menu
      await page.evaluate(() => { const T = __town.rp; T.rpSetTool("stakes"); T.rpMenu("width", 3); document.getElementById("radial").hidden = false; __stand(22, 18, 0, 0.5, 27, 16); });
      await settle(); await shot(`menu-width3-${t}`);
      await page.evaluate(() => { document.getElementById("radial").hidden = true; __orbit({ tx: 2.833 * 20, tz: 2.4535 * 16, dist: 52, yaw: -1.2, pitch: 0.85 }); }); await page.waitForTimeout(400); await page.evaluate(() => __mouseAt(35, 10));
      await settle(); await shot(`width3-overview-${t}`);
      // paving: Keep paving down the line, across the bridge, a lamp lit
      await page.evaluate(() => { const T = __town.rp; T.rpReset(true); T.rpSetTool("shovel"); __stand(14, 16, 0, 0, 18, 16); for (let i = 0; i < 9; i++) T.rpKeepStep(); });
      await settle(); await shot(`paving-${t}`);
      await page.evaluate(() => { const T = __town.rp; for (let i = 0; i < 6; i++) T.rpKeepStep(); __stand(26, 15, 0, 1.2, 18, 16, 1); });
      await settle(); await shot(`paved-back-${t}`);
      // edit mid-build: the bridge leg's stake moved off the paved run
      await page.evaluate(() => { const T = __town.rp; T.rpSetTool("stakes"); T.rpEditDemo(); __stand(11, 20, 0, 0.6, 15, 16); });
      await settle(); await shot(`edit-midbuild-${t}`);
      await page.evaluate(() => { __orbit({ tx: 2.833 * 14, tz: 2.4535 * 17, dist: 46, yaw: -1.45, pitch: 0.95 }); }); await page.waitForTimeout(400); await page.evaluate(() => __mouseAt(30, 16));
      await settle(); await shot(`edit-overview-${t}`);
    }
    await page.close();
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
