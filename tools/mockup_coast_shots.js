// The towns mockup's harbour (`makeCoast`), for the game's slice 4d shots
// to stand beside: its overview by day and at night, and its own views of a
// fish hut and a boathouse. Usage: THREE_JS=<path to three.min.js r128>
// NODE_PATH=$(npm root -g) node tools/mockup_coast_shots.js <out dir>
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto("file:///home/user/Pale-Blue-Dot/docs/mockups/towns.html#coast");
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "coast", null, { timeout: 60000 });
  await page.waitForTimeout(1500);
  const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; for (const d of DOORS) { d.open = true; d.solid.off = true; } }, h);
  const shot = async (name, fn) => { await page.evaluate(fn); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.png` }); };
  await at(11);
  await shot("mockup-coast-overview-11h00", () => { setView("overview"); });
  await shot("mockup-coast-stilts-11h00", () => { teleport(GOTO.stilts); setView("walk"); });
  await shot("mockup-coast-boathouse-11h00", () => { teleport(GOTO.boathouse); setView("walk"); });
  await at(22.5);
  await shot("mockup-coast-overview-22h30", () => { setView("overview"); });
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
