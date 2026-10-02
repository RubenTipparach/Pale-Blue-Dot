// The towns mockup's desert town (`makeDesert`), for the game's slice 4e
// shots to stand beside: the plaza toward the oasis by day and at 22:30, a
// sandstone house's roof stair from its foot, its roof, a domed house, the
// caravan hall and the whole town from above, each from where the game's
// shot stands. Usage:
// THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g) node
// tools/mockup_desert_shots.js <out dir> [names]
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const only = process.argv[3] ? process.argv[3].split(",") : null;
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto("file:///home/user/Pale-Blue-Dot/docs/mockups/towns.html#desert");
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "desert", null, { timeout: 60000 });
  await page.waitForTimeout(1500);
  const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; for (const d of DOORS) { d.open = true; d.solid.off = true; } }, h);
  const shot = async (name, fn, arg) => { if (only && !only.includes(name)) return; await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.png` }); };
  // A spot's name, or a cell to stand in and one to look at, goes in as the
  // page function's argument: `evaluate` sends the function's source, not
  // what it closes over.
  const walk = (spot) => { teleport(GOTO[spot]); setView("walk"); };
  const cells = ([[c, r], [lc, lr]]) => { teleport({ x: cx(c, r), z: cz(c, r), y: 2, look: [cx(lc, lr), cz(lc, lr)] }); setView("walk"); };
  for (const [h, tag] of [[11, "11h00"], [22.5, "22h30"]]) {
    await at(h);
    await shot(`mockup-desert-plaza-${tag}`, walk, "plaza");
    await shot(`mockup-desert-roofstair-${tag}`, walk, "roofStair");
    await shot(`mockup-desert-roof-${tag}`, walk, "roof");
    await shot(`mockup-desert-domed-${tag}`, cells, [[16, 14], [14, 14]]);
    await shot(`mockup-desert-hall-${tag}`, cells, [[32, 23], [35, 23]]);
    await shot(`mockup-desert-overview-${tag}`, () => { setView("overview"); });
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
