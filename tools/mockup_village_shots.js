// The towns mockup's village from the same spots as the game's Holbrook shots
// (tools/capture_holbrook.sh). Usage: THREE_JS=<path to three.min.js r128>
// NODE_PATH=$(npm root -g) node tools/mockup_village_shots.js <out dir>
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const out = process.argv[2];
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = []; page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto("file:///home/user/Pale-Blue-Dot/docs/mockups/towns.html#village");
  await page.waitForFunction(() => typeof SCENE !== "undefined" && SCENE === "village", null, { timeout: 60000 });
  await page.waitForTimeout(1500);
  await page.evaluate(() => { document.getElementById("tod").value = 11; setTime(11); assignLights(); for (const s of document.querySelectorAll(".panel")) s.style.display = "none"; for (const d of DOORS) { d.open = true; d.solid.off = true; } });
  const shot = async (name, fn) => { await page.evaluate(fn); await page.waitForTimeout(900); await page.screenshot({ path: `${out}/${name}.png` }); };
  await shot("mockup-overview", () => { setView("overview"); });
  const door = () => { const [A, B] = edgeEnds(11, 16, 1), [nx, nz] = dvec(1); return { mx: (A[0] + B[0]) / 2, mz: (A[1] + B[1]) / 2, nx, nz }; };
  await shot("mockup-outside", `(() => { const d = (${door})(); teleport({ x: d.mx + d.nx * 3.5, z: d.mz + d.nz * 3.5, y: 1, look: [d.mx, d.mz] }); setView("walk"); })()`);
  await shot("mockup-inside", `(() => { const d = (${door})(); teleport({ x: d.mx - d.nx * 1.6, z: d.mz - d.nz * 1.6, y: 1, look: [d.mx + d.nx * 9, d.mz + d.nz * 9] }); setView("walk"); })()`);
  await shot("mockup-lane", () => { teleport({ x: cx(20, 17), z: cz(20, 17), y: 1, look: [cx(21, 17), cz(21, 17)] }); setView("walk"); });
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
