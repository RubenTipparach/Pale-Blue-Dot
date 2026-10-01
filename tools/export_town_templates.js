// The towns mockup's settlement layouts, written out as templates for the
// game (`openspec/changes/cities-in-the-world`, the design's slice 1). The
// mockup lays each settlement out by calling `building()` for every house,
// hall and hut; this runs a scene headless with `building` wrapped, and
// records what each call asked for, resolved: its cells, kit, storeys,
// doors, every outside edge that has a window, its roof and its chimney. It
// also records the ground the scene paints (lanes, the green, the pond), so
// the game's template is the approved mockup's layout, not a re-typing of it.
//
//     NODE_PATH=$(npm root -g) node tools/export_town_templates.js [scene ...]
//
// Default scene: village. Writes `assets/settlements/v1/<scene>.json`. THREE_JS
// as in `export_town_textures.js`.

const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const ROOT = path.join(__dirname, "..");
const OUT = path.join(ROOT, "assets", "settlements", "v1");
const scenes = process.argv.slice(2).length ? process.argv.slice(2) : ["village"];
// Scenes that stand on several levels, each built cell at its own height
// (`cities-in-the-world` slice 4b); every other scene is laid flat.
const TERRACED = new Set(["town", "coast", "mountain", "mounds"]);

(async () => {
  const executablePath = fs.existsSync("/opt/pw-browsers/chromium") ? "/opt/pw-browsers/chromium" : undefined;
  const browser = await chromium.launch({ executablePath }).catch(() => chromium.launch());
  const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  if (process.env.THREE_JS) {
    const body = fs.readFileSync(process.env.THREE_JS);
    await page.route("**/three.min.js", (route) => route.fulfill({ body, contentType: "text/javascript" }));
  }
  await page.goto("file://" + path.join(ROOT, "docs", "mockups", "towns.html"));
  await page.waitForFunction(() => typeof SCENES === "object" && typeof building === "function", null, { timeout: 60000 });
  fs.mkdirSync(OUT, { recursive: true });
  for (const scene of scenes) {
    const layout = await page.evaluate((scene) => {
      const found = [];
      const inner = building;
      // eslint-disable-next-line no-global-assign
      building = function (o) {
        const out = inner(o);
        const kit = KITS[o.kit || "halftimber"];
        const cells = out.cells.map(([c, r]) => [c, r]);
        const inB = (c, r) => cells.some(([a, b]) => a === c && b === r);
        const storeys = o.storeys || (kit.hut ? 1 : 2);
        const windows = [];
        if (o.window) {
          for (const [c, r] of cells) for (let d = 0; d < 6; d++) {
            const [c2, r2] = nb(c, r, d);
            if (inB(c2, r2)) continue;
            for (let s = 0; s < storeys; s++) {
              const door = (o.doors || []).find((e) => e[0] === c && e[1] === r && e[2] === d && (e[3] || 0) === s);
              if (!door && o.window(c, r, d, s)) windows.push([c, r, d, s]);
            }
          }
        }
        found.push({
          name: o.name || "",
          kit: o.kit || "halftimber",
          cells,
          base: out.base,
          storeys,
          tall: o.tall || 1,
          doors: (o.doors || []).map((e) => [e[0], e[1], e[2], e[3] || 0]),
          windows,
          roof: o.roof || kit.roof,
          pitch: o.pitch ?? kit.pitch ?? 1,
          chimney: o.chimney || null,
          stair_cells: o.stairCells || [],
        });
        return out;
      };
      try {
        resetWorld();
        SCENES[scene].build();
      } finally {
        building = inner;
      }
      const ground = [];
      for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
        const i = idx(c, r);
        ground.push({ c, r, h: TOP[i], top: TOPMAT[i], area: AREA[i] });
      }
      // The walled town's curtain wall and gates, as `buildWalls` raises
      // them: each wall cell from its ground (a gate from 4 m over it) to
      // WALL_TOP, with merlons on each edge that looks out of the town and
      // not onto more wall (`cities-in-the-world` slice 4c).
      const masonry = [];
      if (scene === "town") {
        for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
          if (!isWall(c, r)) continue;
          const base = TOP[idx(c, r)], merlons = [];
          for (let d = 0; d < 6; d++) {
            const [c2, r2] = nb(c, r, d);
            if (!(isWall(c2, r2) || inTown(c2, r2))) merlons.push(d);
          }
          masonry.push({ c, r, from: gate(c, r) ? base + 4 : base, to: WALL_TOP, merlons });
        }
      }
      const out = { scene, grid: { columns: NC, rows: NR, cell_m: W }, buildings: found, ground, lamps: STREET_LAMPS.slice() };
      if (masonry.length) out.masonry = masonry;
      return out;
    }, scene);
    if (TERRACED.has(scene)) layout.terraced = true;
    const file = path.join(OUT, `${scene}.json`);
    fs.writeFileSync(file, JSON.stringify(layout, null, 1) + "\n");
    console.log(`${scene}: ${layout.buildings.length} buildings -> ${path.relative(ROOT, file)}`);
  }
  if (errors.length) console.log("page errors:", errors.join("; "));
  await browser.close();
})();
