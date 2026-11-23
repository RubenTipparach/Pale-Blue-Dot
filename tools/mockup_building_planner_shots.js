// M6's Planner, Site and Free building tabs at desktop and phone widths, by night and by day.
// (openspec/changes/building-planner). Serve docs/mockups over http (the textures will not load
// from file://), e.g. `python3 -m http.server 8765` in docs/mockups, then:
// THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) node tools/mockup_building_planner_shots.js
//   http://127.0.0.1:8765/building.html docs/screenshots/building-planner [desktop|phone]
// SwiftShader (software GL): for a container with no GPU only (CLAUDE.md); on a desktop with a GPU
// launch with --use-angle=d3d11 --enable-gpu --ignore-gpu-blocklist instead.
const { chromium } = require("playwright"); const fs = require("fs");
(async () => {
  const [url, out, only] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--disable-dev-shm-usage"] });
  const body = fs.readFileSync(process.env.THREE_JS);
  const errors = [];
  for (const [tag, vp] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    if (only && only !== tag) continue;
    const open = async (hash) => { const pg = await browser.newPage({ viewport: vp, deviceScaleFactor: 1 });
      pg.on("pageerror", e => errors.push(tag + " " + String(e)));
      pg.on("console", m => { if (m.type() === "error") errors.push(tag + " console " + m.text()); });
      await pg.route("**/three.min.js", r => r.fulfill({ body, contentType: "text/javascript" }));
      await pg.route("https://fonts.googleapis.com/**", r => r.fulfill({ body: "", contentType: "text/css" }));
      await pg.goto(`${url}#${hash}`); await pg.waitForTimeout(1500); return pg; };
    let page = await open("planner");
    const full = tag === "phone";
    let shot; shot = async (name, wait = 700) => { await page.waitForTimeout(wait); await page.screenshot({ path: `${out}/${name}-${tag}.jpg`, type: "jpeg", quality: 84, fullPage: full }); };
    // Planner: the cottage preset; a plan with problems; the two-storey house upstairs
    await shot("planner-cottage");
    await page.evaluate(() => { const P = __planner; P.PL.plan = P.makePreset(P.PRESETS[2], 1); P.PL.storey = 1; const p = P.PL.plan; p.st[1].add("2,0"); p.stairs[0] && (p.stairs[0].d = (p.stairs[0].d + 3) % 6); p.edges = p.edges; openTab("planner"); });
    await shot("planner-checks");
    await page.evaluate(() => { const P = __planner; P.PL.plan = P.makePreset(P.PRESETS[2], 1); P.PL.storey = 1; openTab("planner"); });
    await shot("planner-upstairs");
    // Site: placing, refused, building with scaffolding, changed mid-build; night then day
    for (const night of [true, false]) {
      const t = night ? "night" : "day";
      await page.evaluate(n => { const P = __planner; P.setNight(n); document.getElementById(n ? "sNight" : "sDay").click(); document.getElementById("bclear").click(); P.newSite(P.makePreset(P.PRESETS[2], 1)); openTab("site"); }, night);
      await page.waitForTimeout(2500);
      await shot(`site-ghost-${t}`, 1200);
      await page.evaluate(() => { const S = __planner.SITE; S.origin = [-3, 3]; document.getElementById("tr").click(); });
      await shot(`site-refused-${t}`, 1200);
      await page.evaluate(() => { const S = __planner.SITE; S.origin = [0, 0]; document.getElementById("tl").click(); });
      await page.waitForTimeout(800);
      await page.evaluate(() => { const P = __planner, S = P.SITE; document.getElementById("sout").click(); for (let i = 0; i < 24; i++) P.strike(S); });
      await shot(`site-building-${t}`, 1500);
      await page.evaluate(() => { const P = __planner, S = P.SITE; document.getElementById("bfill").click(); for (let i = 0; i < 200 && S.phase === "building"; i++) P.strike(S); document.getElementById("bfill").click(); for (const b of document.querySelectorAll("#edits button")) if (b.dataset.e === "drop") b.click(); });
      await shot(`site-changed-${t}`, 1500);
    }
    // Site: the change in the planner, opened from the site
    await page.evaluate(() => { document.getElementById("sedit").click(); const P = __planner; P.PL.plan.st[0].add("2,-1"); openTab("planner"); });
    await shot("planner-editing-site");
    // Free building, night and day (a fresh page: one WebGL context at a time on SwiftShader)
    await page.close(); page = await open("builder"); await page.waitForTimeout(3000);
    await shot("free-night", 1200);
    await page.evaluate(() => document.getElementById("tDay").click());
    await shot("free-day", 1200);
    // the info modal
    await page.evaluate(() => document.getElementById("info").click());
    await page.screenshot({ path: `${out}/info-${tag}.jpg`, type: "jpeg", quality: 84 });
    await page.close();
  }
  console.log("errors:", errors.length ? errors.join("\n") : "none");
  await browser.close();
})();
