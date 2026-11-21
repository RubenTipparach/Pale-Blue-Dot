// The towns mockup's Roads tab (`makeRoads`, townsfolk task 8.1): each of
// its spots by day and at 22:30, the whole scene from above, and a walk up
// the hill road that counts the eye's jumps (there must be none).
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g)
// node tools/mockup_roads_shots.js <towns.html> <out dir>
const fs = require("fs"); const { chromium } = require("playwright");
const SPOTS = ["crossroads", "hillfoot", "climb", "hilltop", "ford", "bridge", "gravel"];
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file://${html}#roads`);
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "roads", null, { timeout: 90000 });
  await page.waitForTimeout(1500);
  // The walk: from the foot of the hill road to the turning place, the walker
  // pressing forward and turned along each cell of the road in turn.
  const walk = await page.evaluate(() => {
    const T = window.__town;
    const climb = ROAD_CLIMB;
    T.teleport({ x: cx(...climb[0]), z: cz(0, climb[0][1]), y: TOP[idx(...climb[0])] });
    T.P.jolts.length = 0;
    let maxRise = 0, prev = T.P.feet, steps = 0;
    for (let k = 1; k < climb.length; k++) {
      const tx = cx(...climb[k]), tz = cz(0, climb[k][1]);
      for (let n = 0; n < 400; n++) {
        T.P.yaw = Math.atan2(tz - T.P.z, tx - T.P.x);
        T.tick(1 / 120, { fwd: 1, side: 0, jump: false, sprint: false, down: false });
        maxRise = Math.max(maxRise, T.P.feet - prev); prev = T.P.feet; steps++;
        if (Math.hypot(tx - T.P.x, tz - T.P.z) < 0.3) break;
      }
    }
    return { climbed: T.P.feet - TOP[idx(...climb[0])], jolts: T.P.jolts.length, maxRisePerTick: maxRise, ticks: steps };
  });
  console.log("walk up the hill road:", JSON.stringify(walk));
  const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; }, h);
  const shot = async (name, fn, arg) => { await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 85 }); };
  for (const [h, tag] of [[11, "11h00"], [22.5, "22h30"]]) {
    await at(h);
    for (const spot of SPOTS) await shot(`mockup-roads-${spot}-${tag}`, (s) => { teleport(GOTO[s]); setView("walk"); }, spot);
    await shot(`mockup-roads-overview-${tag}`, () => { setView("overview"); });
  }
  // the info modal, open, at a phone's width
  await page.setViewportSize({ width: 390, height: 844 });
  await page.evaluate(() => { for (const s of document.querySelectorAll(".panel")) s.style.display = ""; document.getElementById("infoOpen").click(); });
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${out}/mockup-roads-info-phone.jpg`, type: "jpeg", quality: 85 });
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
