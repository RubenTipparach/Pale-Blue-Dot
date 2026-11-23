// The towns mockup's Furniture scene (`makeFurniture`, `furniture` task 1.2,
// M15 version 5): its places by day (11:00) and at 22:30, the chest panel,
// and the whole scene from above, at desktop width and at phone width.
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g)
// node tools/mockup_furniture_shots.js <towns.html> <out dir>
// SwiftShader (software GL): for a container with no GPU only (CLAUDE.md).
const fs = require("fs"); const { chromium } = require("playwright");
const SPOTS = ["outside", "room", "seated", "lying", "asleep", "chestpanel", "show-chair", "show-stool", "show-bench", "show-table", "show-bed", "show-drawers", "show-closet", "show-chest", "show-banded-chest", "show-crate"];
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const body = fs.readFileSync(process.env.THREE_JS);
  const errors = [];
  const only = process.env.SPOTS ? process.env.SPOTS.split(",") : null;
  for (const [tag, vp, spots] of [["desktop", { width: 1440, height: 900 }, only || SPOTS], ["phone", { width: 390, height: 844 }, ["room", "chestpanel"]]]) {
    const page = await browser.newPage({ viewport: vp });
    page.on("pageerror", (e) => errors.push(String(e)));
    await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
    await page.goto(`file://${html}#furniture`);
    await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "furniture", null, { timeout: 120000 });
    await page.waitForTimeout(1500);
    const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); }, h);
    const shot = async (name, fn, arg) => { await page.evaluate(fn, arg); await page.evaluate(() => assignLights()); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 85 }); };
    for (const [h, t] of [[11, "11h00"], [22.5, "22h30"]]) {
      await at(h);
      for (const spot of spots) await shot(`m15-furniture-${spot}-${t}-${tag}`, (s) => { teleport(GOTO[s]); setView("walk"); }, spot);
      if (!only) await shot(`m15-furniture-overview-${t}-${tag}`, () => { chestOpen(false); Object.assign(ORB, { ty: 2 }, SCENES.furniture.orbit); setView("overview"); });
    }
    await page.close();
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
