// The world map mockup's sites layer and site editor, driven headless
// (`openspec/changes/city-sites` tasks 1.2 and 1.3). It serves
// `docs/mockups` on a local port, opens the map, and:
//
// 1. checks the sites loaded from `world-map/sites.json`;
// 2. takes a screenshot at world zoom and one close in, where the
//    footprints show;
// 3. turns the editor on, drags a site, and checks that the RON the editor
//    writes pins it at its new place and strikes its old id.
//
//     NODE_PATH=$(npm root -g) node tools/mockup_sites_test.js [screenshot dir]
//
// It exits non-zero on a failed check. Chromium is the pre-installed one
// when there is one (/opt/pw-browsers/chromium).

const http = require("http");
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const ROOT = path.join(__dirname, "..", "docs", "mockups");
const OUT = process.argv[2] || null;
const TYPES = { ".html": "text/html", ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg", ".js": "text/javascript" };

function serve() {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      const file = path.join(ROOT, decodeURIComponent(req.url.split("?")[0]));
      if (!file.startsWith(ROOT) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end(); return; }
      const type = TYPES[path.extname(file)] || "application/octet-stream";
      res.writeHead(200, { "Content-Type": type.startsWith("text/") || type.endsWith("json") ? type + "; charset=utf-8" : type });
      fs.createReadStream(file).pipe(res);
    });
    server.listen(0, "127.0.0.1", () => resolve(server));
  });
}

function check(ok, what) {
  if (!ok) { console.error(`FAIL: ${what}`); process.exitCode = 1; }
  else console.log(`ok: ${what}`);
}

(async () => {
  const server = await serve();
  const url = `http://127.0.0.1:${server.address().port}/world-map.html`;
  const executablePath = fs.existsSync("/opt/pw-browsers/chromium") ? "/opt/pw-browsers/chromium" : undefined;
  const browser = await chromium.launch({ executablePath }).catch(() => chromium.launch());
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  await page.goto(url);
  await page.waitForFunction(() => document.getElementById("placeCount").textContent.includes("("), null, { timeout: 30000 });
  await page.waitForTimeout(1500);
  const count = await page.$eval("#placeCount", (e) => e.textContent);
  const json = JSON.parse(fs.readFileSync(path.join(ROOT, "world-map", "sites.json"), "utf8"));
  check(count === `(${json.sites.length})`, `the page lists the ${json.sites.length} sites of sites.json (${count})`);
  if (OUT) await page.screenshot({ path: path.join(OUT, "mockup-world.png") });

  // Close in on the capital: its footprint is drawn.
  const capital = json.sites.find((s) => s.capital);
  const buttons = await page.$$("#places button");
  let target = null;
  for (const b of buttons) if ((await b.textContent()).startsWith(capital.name)) target = b;
  check(!!target, `the capital, ${capital.name}, is in the places list`);
  await target.click();
  for (let k = 0; k < 4; k++) await page.click("#zoomIn");
  await page.waitForTimeout(1200);
  if (OUT) await page.screenshot({ path: path.join(OUT, "mockup-close.png") });

  // The editor: drag the capital 60 px east and read the RON.
  await page.check("#editSites");
  const box = await page.$eval("#view", (c) => { const r = c.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; });
  await page.mouse.move(box.x, box.y);
  await page.mouse.down();
  await page.mouse.move(box.x + 30, box.y, { steps: 4 });
  await page.mouse.move(box.x + 60, box.y, { steps: 4 });
  await page.mouse.up();
  await page.waitForTimeout(300);
  const ron = await page.$eval("#ron", (t) => t.value);
  const pin = ron.split("\n").find((l) => l.includes(`name: Some("${capital.name}")`));
  check(!!pin, `the RON pins ${capital.name}`);
  if (pin) {
    const lon = Number(/lon: (-?[\d.]+)/.exec(pin)[1]);
    const lat = Number(/lat: (-?[\d.]+)/.exec(pin)[1]);
    check(lon > capital.lon + 0.01, `its new longitude ${lon.toFixed(4)} is east of ${capital.lon.toFixed(4)}`);
    check(Math.abs(lat - capital.lat) < 0.05, `its latitude stays at ${capital.lat.toFixed(2)}`);
    check(/capital: true/.test(pin), "it is still the capital");
  }
  check(new RegExp(`strikes: \\[[^\\]]*\\b${capital.id}\\b`).test(ron), `the RON strikes its old id ${capital.id}`);
  if (OUT) await page.screenshot({ path: path.join(OUT, "mockup-edit.png") });
  check(errors.length === 0, `no page errors${errors.length ? ": " + errors.join("; ") : ""}`);
  console.log(ron);
  await browser.close();
  server.close();
})();
