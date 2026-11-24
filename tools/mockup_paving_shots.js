// The towns mockup's Paving scene (`makePaving`, townsfolk design 39 to 41,
// M15 version 4): its places by day (11:00) and at 22:30, the whole scene
// from above, at desktop width and at phone width.
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g)
// node tools/mockup_paving_shots.js <towns.html> <out dir>
// SwiftShader (software GL): for a container with no GPU only (CLAUDE.md).
const fs = require("fs"); const { chromium } = require("playwright");
const SPOTS = ["worksite", "grinder", "mixer", "paved", "firststake", "stake", "marked", "bend", "preview"];
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const body = fs.readFileSync(process.env.THREE_JS);
  const errors = [];
  for (const [tag, vp, spots] of [["desktop", { width: 1440, height: 900 }, SPOTS], ["phone", { width: 390, height: 844 }, ["firststake", "grinder", "bend"]]]) {
    const page = await browser.newPage({ viewport: vp });
    page.on("pageerror", (e) => errors.push(String(e)));
    await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
    await page.goto(`file://${html}#paving`);
    await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "paving", null, { timeout: 120000 });
    await page.waitForTimeout(1500);
    const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); }, h);
    const shot = async (name, fn, arg) => { await page.evaluate(fn, arg); await page.evaluate(() => assignLights()); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 85 }); };
    for (const [h, t] of [[11, "11h00"], [22.5, "22h30"]]) {
      await at(h);
      for (const spot of spots) await shot(`m15-paving-${spot}-${t}-${tag}`, (s) => { teleport(GOTO[s]); setView("walk"); }, spot);
      await shot(`m15-paving-overview-${t}-${tag}`, () => { Object.assign(ORB, { ty: 2 }, SCENES.paving.orbit); setView("overview"); });
    }
    await page.close();
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
