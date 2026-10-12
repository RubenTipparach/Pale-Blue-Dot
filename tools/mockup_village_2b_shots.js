// The towns mockup's village from the same spots as the game's slice 2b
// shots (tools/capture_holbrook_2b.sh): the Fieldstone house's newel from the
// room beside it, below and above; the first half-timbered house's flight,
// from before its foot and from its landing; and the Fieldstone house's door,
// shut and open. Each stair is found by the rule the game's cutter follows
// (`cities-in-the-world` slice 2b). Usage: THREE_JS=<three.min.js r128>
// NODE_PATH=$(npm root -g) node tools/mockup_village_2b_shots.js <out dir>
// HOUR=22.5 takes them at another hour (11 by default), and SUFFIX=-22h30 is
// added to every name.
const fs = require("fs"); const path = require("path"); const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const root = path.resolve(__dirname, "..");
  const village = JSON.parse(fs.readFileSync(`${root}/assets/settlements/v1/village.json`, "utf8"));
  const stone = village.buildings.find(b => b.name === "Fieldstone house");
  const timbered = village.buildings.find(b => b.stair_cells.length === 2);
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file://${root}/docs/mockups/towns.html#village`);
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "village", null, { timeout: 60000 });
  await page.waitForTimeout(1500);
  const hour = Number(process.env.HOUR || 11), suffix = process.env.SUFFIX || "";
  await page.evaluate((h) => { document.getElementById("tod").value = h; setTime(h); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; }, hour);
  const shot = async (name, fn, arg) => { await page.evaluate(fn, arg); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}${suffix}.png` }); };
  // The newel: its entry is the first edge onto another of the house's cells
  // that is not the door's; stand 2.6 m out through it, look back at it.
  const newel = ({ b, up, pitch }) => {
    const [c, r] = b.stair_cells[0], door = b.doors[0], own = (a, q) => b.cells.some(x => x[0] === a && x[1] === q);
    let entry = 0; for (let d = 0; d < 6; d++) { const [a, q] = nb(c, r, d); if (own(a, q) && !(a === door[0] && q === door[1])) { entry = d; break; } }
    const [ux, uz] = dvec(entry), x = cx(c, r), z = cz(c, r);
    teleport({ x: x + ux * 2.6, z: z + uz * 2.6, y: 1 + up, look: [x, z] }); P.pitch = pitch; setView("walk");
  };
  // The flight climbs along +x from its first cell's far flat.
  const flight = ({ b, landing, pitch }) => {
    const [c, r] = b.stair_cells[0], x0 = cx(c, r) - W / 2, z0 = cz(c, r);
    if (landing) teleport({ x: x0 + 2 * W - 0.35, z: z0, y: 4, look: [x0 - 5, z0] });
    else teleport({ x: x0 - 1.4, z: z0, y: 1, look: [x0 + 5, z0] });
    P.pitch = pitch; setView("walk");
  };
  const door = ({ b, open }) => {
    for (const d of DOORS) { d.open = open; d.solid.off = open; }
    const [c, r, dd] = b.doors[0], [A, B] = edgeEnds(c, r, dd), [nx, nz] = dvec(dd), mx = (A[0] + B[0]) / 2, mz = (A[1] + B[1]) / 2;
    teleport({ x: mx + nx * 3.5, z: mz + nz * 3.5, y: 1, look: [mx, mz] }); P.pitch = 0; setView("walk");
  };
  await page.evaluate(() => { for (const d of DOORS) { d.open = true; d.solid.off = true; } });
  await shot("mockup-2b-newel-below", `(${newel})(${JSON.stringify({ b: stone, up: 0, pitch: 0.35 })})`);
  await shot("mockup-2b-newel-above", `(${newel})(${JSON.stringify({ b: stone, up: 3, pitch: -0.6 })})`);
  await shot("mockup-2b-flight-foot", `(${flight})(${JSON.stringify({ b: timbered, landing: false, pitch: 0.35 })})`);
  await shot("mockup-2b-flight-landing", `(${flight})(${JSON.stringify({ b: timbered, landing: true, pitch: -0.5 })})`);
  await shot("mockup-2b-door-shut", `(${door})(${JSON.stringify({ b: stone, open: false })})`);
  await shot("mockup-2b-door-open", `(${door})(${JSON.stringify({ b: stone, open: true })})`);
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
