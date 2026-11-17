// M35's New tab (`docs/mockups/waterfall-mist.html`) from above, beside and
// under a tall fall (9 layers) and a short one (2 layers), by day, for the
// game's `--capture` shots of the port (`water-flow` 6.6) to stand beside.
// On the GPU (the system Chrome, its own GPU; never SwiftShader).
// Usage: python3 -m http.server 8765 -d docs &
//        NODE_PATH=$(npm root -g) node tools/mockup_waterfall_mist_shots.js <out dir>
const path = require("path");
const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const browser = await chromium.launch({ channel: "chrome", headless: false, args: ["--enable-gpu", "--ignore-gpu-blocklist", "--window-position=0,0"] });
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  // Served over http: a file:// page cannot upload its puff sheet to WebGL.
  const url = (process.env.MOCKUP_BASE || "http://127.0.0.1:8765") + "/mockups/waterfall-mist.html#new";
  await page.goto(url);
  await page.waitForFunction(() => window.PAGE && window.PAGE.P, null, { timeout: 60000 });
  await page.waitForTimeout(2500);
  const renderer = await page.evaluate(() => window.PAGE.P.info());
  console.log("renderer:", renderer);
  // The canvas alone, held at one moment so the two sides compare.
  await page.evaluate(() => { window.PAGE.P.stop(); for (const id of ["newPanel", "ref", "flabels"]) { const e = document.getElementById(id); if (e) e.style.display = "none"; } });
  const view = await page.$("#view");
  for (const [focus, tag] of [["f9", "tall"], ["f2", "short"]]) {
    for (const v of ["above", "side", "under"]) {
      await page.evaluate(([f, v]) => { window.PAGE.P.look(v, f); window.PAGE.P.setTime(12.0); }, [focus, v]);
      await page.waitForTimeout(400);
      await view.screenshot({ path: `${out}/mockup-${tag}-${v}.png` });
    }
  }
  console.log(errors.length ? "page errors: " + errors.join(" | ") : "no page errors");
  await browser.close();
})();
