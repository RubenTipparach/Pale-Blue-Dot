// The towns mockup's harbour (`makeCoast`), for the game's slice 4d shots
// to stand beside: its overview by day and at night, and its own views of a
// fish hut, a boathouse, the quay, the market, the shipyard and the cog, and
// a rowboat and a canoe close up. Usage: THREE_JS=<path to three.min.js r128>
// NODE_PATH=$(npm root -g) node tools/mockup_coast_shots.js <out dir> [names]
// where `names`, comma-separated, takes only those shots.
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const only = process.argv[3] ? process.argv[3].split(",") : null;
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto("file:///home/user/Pale-Blue-Dot/docs/mockups/towns.html#coast");
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "coast", null, { timeout: 60000 });
  await page.waitForTimeout(1500);
  const at = (h) => page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; for (const d of DOORS) { d.open = true; d.solid.off = true; } }, h);
  const shot = async (name, fn, arg) => { if (only && !only.includes(name)) return; await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.png` }); };
  // A spot's name goes in as the page function's argument: `evaluate`
  // sends the function's source, not what it closes over.
  const walk = (spot) => { teleport(GOTO[spot]); setView("walk"); };
  // The first moored boat of a kind (its solid's label), seen along it from
  // off one end, 9 m away and `pitch` above, as the game's chase view of
  // one: the overview's orbit, closed in on it.
  const astern = ([label, pitch]) => {
    const s = SOLIDS.find((s) => s.label === label);
    const c = s.pts.reduce((a, p) => [a[0] + p[0] / s.pts.length, a[1] + p[1] / s.pts.length], [0, 0]);
    const end = s.pts.reduce((a, p) => (Math.hypot(p[0] - c[0], p[1] - c[1]) > Math.hypot(a[0] - c[0], a[1] - c[1]) ? p : a));
    setView("overview"); marker.visible = false;
    Object.assign(ORB, { tx: c[0], tz: c[1], ty: 0.3, dist: 9, pitch, yaw: Math.atan2(c[1] - end[1], c[0] - end[0]) });
  };
  await at(11);
  await shot("mockup-coast-overview-11h00", () => { setView("overview"); });
  await shot("mockup-coast-stilts-11h00", walk, "stilts");
  await shot("mockup-coast-boathouse-11h00", walk, "boathouse");
  // The game's harbour retakes and the cog's shots stand beside these.
  await shot("mockup-coast-quay-11h00", walk, "quay");
  await shot("mockup-coast-rowboat-11h00", astern, ["Rowboat", 0.42]);
  // The canoes lie under the fish huts: from higher, to clear their roofs.
  await shot("mockup-coast-canoe-11h00", astern, ["Canoe", 0.9]);
  await shot("mockup-coast-market-11h00", walk, "market");
  await shot("mockup-coast-shipyard-11h00", walk, "shipyard");
  await shot("mockup-coast-cog-11h00", walk, "cog");
  await shot("mockup-coast-castle-11h00", walk, "castle");
  await at(22.5);
  await shot("mockup-coast-overview-22h30", () => { setView("overview"); });
  await shot("mockup-coast-quay-22h30", walk, "quay");
  await shot("mockup-coast-market-22h30", walk, "market");
  await shot("mockup-coast-castle-22h30", walk, "castle");
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
