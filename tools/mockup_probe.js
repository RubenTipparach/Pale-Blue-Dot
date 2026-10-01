// Render named views of the towns mockup: node tools/mockup_probe.js <out dir> <scene> <name> <js> [<name> <js> ...]
// Env: THREE_JS=<three.min.js r128>, PAGE=<page> (default docs/mockups/towns.html). Each <js> runs in the page after load, at 11:00; a non-undefined result is printed.
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const [out, scene, ...rest] = process.argv.slice(2);
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 900, height: 600 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file://${process.env.PAGE || require("path").resolve(__dirname, "../docs/mockups/towns.html")}#${scene}`);
  await page.waitForFunction((s) => typeof SCENE !== "undefined" && SCENE === s && document.getElementById("loading").hidden, scene, { timeout: 90000 });
  await page.waitForTimeout(800);
  await page.evaluate(() => { for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; document.getElementById("tod").value = 11; setTime(11); assignLights(); });
  for (let i = 0; i + 1 < rest.length; i += 2) {
    const name = rest[i], js = rest[i + 1];
    const r = await page.evaluate(js);
    if (r !== undefined) console.log(name, JSON.stringify(r));
    await page.waitForTimeout(700);
    await page.screenshot({ path: `${out}/${name}.png` });
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
