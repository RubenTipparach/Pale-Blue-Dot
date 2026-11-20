// Screenshots of M25's "Beavers and otters" tab (ocean.html#critters; ocean-environments design B28).
// Usage: THREE_JS=<path to three.min.js r128> NODE_PATH=$(npm root -g) node tools/mockup_critters_shots.js \
//          <docs/mockups dir> <out dir> <shots.json>
// shots.json is a list of {name, q (query, default "?shot"), w, h, pre, run (seconds stepped without drawing), js,
// wait (ms), log (an expression printed), png, live (let the clock run in real time)}. Each shot opens the page afresh, so the seeded animals start the same.
// Chromium on SwiftShader (no GPU): the frames show poses, never smoothness.
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const [dir, out, spec] = process.argv.slice(2); const shots = JSON.parse(fs.readFileSync(spec, "utf8"));
  const only = process.env.ONLY ? process.env.ONLY.split(",") : null;
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--allow-file-access-from-files"] });
  const errors = [], body = fs.readFileSync(process.env.THREE_JS);
  for (const s of shots) {
    if (only && !only.includes(s.name)) continue;
    const page = await browser.newPage({ viewport: { width: s.w || 1280, height: s.h || 800 } });
    page.on("pageerror", e => errors.push(s.name + ": " + String(e)));
    page.on("console", m => { if (m.type() === "error") errors.push(s.name + " console: " + m.text()); });
    await page.route("**/three.min.js", r => r.fulfill({ body, contentType: "text/javascript" }));
    await page.route("https://fonts.googleapis.com/**", r => r.fulfill({ body: "", contentType: "text/css" }));
    await page.goto(`file://${dir}/ocean.html${s.q || "?shot"}#critters`);
    await page.waitForFunction(() => window.__critters, null, { timeout: 90000 });
    if (!s.live) await page.evaluate(() => window.__critters.freeze(true));   // the clock moves only in the fixed steps
    if (s.pre) await page.evaluate(s.pre);
    if (s.run) await page.evaluate(r => window.__critters.run(r), s.run);
    if (s.js) await page.evaluate(s.js);
    await page.waitForTimeout(s.wait || 700);
    if (s.log) console.log(s.name, JSON.stringify(await page.evaluate(s.log)));
    await page.screenshot({ path: `${out}/${s.name}.${s.png ? "png" : "jpg"}`, type: s.png ? "png" : "jpeg", quality: s.png ? undefined : 88 });
    await page.close();
  }
  console.log("errors:", errors.length ? "\n" + errors.join("\n") : "none");
  await browser.close();
})();
