// The towns mockup's pixel textures, written out as PNGs for the game
// (`openspec/changes/cities-in-the-world`, the design's slice 1). The mockup
// paints each texture in the page from a seeded function; this runs those
// same functions headless and saves what they paint, so the game's town wears
// exactly the approved mockup's pixels, as committed PNG assets (CLAUDE.md).
//
//     NODE_PATH=$(npm root -g) node tools/export_town_textures.js
//
// The page loads Three.js from cdnjs. Where that host is out of reach, set
// THREE_JS to a local copy of r128's three.min.js and the request is served
// from it.
//
// It writes `assets/textures/settlement/<name>.png`, one per painted
// material, and `assets/textures/settlement/manifest.ron`: each texture's
// size and the metres one repeat of it covers on a wall or floor (the
// mockup's REP; 0 is a texture mapped per face, as half-timber is).

const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const ROOT = path.join(__dirname, "..");
const OUT = path.join(ROOT, "assets", "textures", "settlement");

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
  await page.waitForFunction(() => typeof PAINT === "object" && typeof texture === "function", null, { timeout: 60000 });
  const painted = await page.evaluate(() =>
    // The painted textures, then the ones drawn by hand (the net), each
    // marked `cut` where the mockup cuts it out where it is clear.
    [...Object.keys(PAINT), ...Object.keys(CUSTOM).filter((n) => !(n in PAINT))].map((name) => {
      const image = texture(name).image;
      return { name, w: image.width, h: image.height, rep: REP[name] ?? 1, cut: ALPHA.has(name), png: image.toDataURL("image/png") };
    })
  );
  fs.mkdirSync(OUT, { recursive: true });
  const lines = [];
  for (const t of painted) {
    fs.writeFileSync(path.join(OUT, `${t.name}.png`), Buffer.from(t.png.split(",")[1], "base64"));
    lines.push(`    (name: "${t.name}", width: ${t.w}, height: ${t.h}, repeat_m: ${Number(t.rep).toFixed(1)}${t.cut ? ", cut: true" : ""}),`);
  }
  const manifest = [
    "// The towns mockup's painted textures (docs/mockups/towns.html), exported by",
    "// tools/export_town_textures.js. `repeat_m` is the metres one repeat covers",
    "// on a wall or a floor; 0 is a texture mapped once per face. A `cut` texture",
    "// is cut out where it is clear, as the mockup's net is.",
    "(",
    "  textures: [",
    ...lines,
    "  ],",
    ")",
    "",
  ].join("\n");
  fs.writeFileSync(path.join(OUT, "manifest.ron"), manifest);
  console.log(`wrote ${painted.length} textures to ${path.relative(ROOT, OUT)}`);
  if (errors.length) console.log("page errors:", errors.join("; "));
  await browser.close();
})();
