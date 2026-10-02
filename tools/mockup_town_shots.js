// The towns mockup's desert town (`makeDesert`, slice 4e) and tundra camp
// (`makeTundra`, slice 4g), for the game's shots to stand beside: each
// scene's own spots by day and at 22:30, and the whole scene from above.
// Each game shot stands where the mockup's does (the app's
// `every_desert_lays_and_cuts` and `every_tundra_camp_lays_and_cuts` print
// where). Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root
// -g) node tools/mockup_town_shots.js <scene> <out dir> [names]
const fs = require("fs"); const { chromium } = require("playwright");
// A spot is a `GOTO` name, or a cell to stand in and a cell to look at.
const VIEWS = {
  desert: { plaza: "plaza", roofstair: "roofStair", roof: "roof", domed: [[16, 14], [14, 14]], hall: [[32, 23], [35, 23]] },
  tundra: { camp: "camp", igloo: "igloo", longhouse: "longhouse", gate: "gate", keeproof: "keepRoof", lake: "lake" },
};
(async () => {
  const [scene, out] = process.argv.slice(2);
  const only = process.argv[4] ? process.argv[4].split(",") : null;
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file:///home/user/Pale-Blue-Dot/docs/mockups/towns.html#${scene}`);
  await page.waitForFunction((s) => typeof SCENE !== "undefined" && SCENE === s, scene, { timeout: 60000 });
  await page.waitForTimeout(1500);
  const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; for (const d of DOORS) { d.open = true; d.solid.off = true; } }, h);
  const shot = async (name, fn, arg) => { if (only && !only.includes(name)) return; await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.png` }); };
  // The spot goes in as the page function's argument: `evaluate` sends the
  // function's source, not what it closes over.
  const go = (spot) => {
    if (typeof spot === "string") teleport(GOTO[spot]);
    else { const [[c, r], [lc, lr]] = spot; teleport({ x: cx(c, r), z: cz(c, r), y: TOP[idx(c, r)], look: [cx(lc, lr), cz(lc, lr)] }); }
    setView("walk");
  };
  for (const [h, tag] of [[11, "11h00"], [22.5, "22h30"]]) {
    await at(h);
    for (const [name, spot] of Object.entries(VIEWS[scene])) await shot(`mockup-${scene}-${name}-${tag}`, go, spot);
    await shot(`mockup-${scene}-overview-${tag}`, () => { setView("overview"); });
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
