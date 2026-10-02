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
const TERRACED = new Set(["town", "coast", "mountain", "mounds", "desert", "tundra"]);
// Scenes that stand on the sea (`cities-in-the-world` slice 4d): the
// template's 0 m is the sea's surface, every height is a whole layer, and
// what stands over the water (piers, stilts) is written as well.
const SEA = new Set(["coast"]);
// Scenes whose wild areas keep the planet's ground (`cities-in-the-world`
// 4e and 4g): every other cell is built, whatever its top.
const WILD = { desert: ["The dunes"], tundra: ["The tundra"] };
// Their tops on the terrain's; a top they do not name keeps the planet's.
const TOPS = { desert: { sand: "sand", sand2: "sand", flag: "stone" }, tundra: { snow: "snow", ice: "snow" } };
// Scenes whose dressing, lanterns and fires are written (4d, 4e, 4g).
const DRESSED = new Set(["coast", "desert", "tundra"]);
// Scenes whose outside stairs and open doorways are written (4e): the
// sandstone houses' flights to their roofs, the caravan hall's arch.
const STAIRED = new Set(["desert", "tundra"]);

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
    const layout = await page.evaluate(({ scene, SEA_SCENES, DRESSED_SCENES, STAIRED_SCENES }) => {
      const found = [];
      const inner = building;
      // The newels and walls a scene raises outside any building(): a stair
      // tower's and the keep's (`cities-in-the-world` slice 4c).
      const newels = [], edges = [];
      let depth = 0;
      const innerNewel = newelStair, innerEdge = edgeWall;
      // The sea's pieces (slice 4d): piers on piles (a `bridge` with piles),
      // a stilt house's deck and porch stair, and the lanterns.
      const sea = SEA_SCENES.includes(scene);
      const dressed = DRESSED_SCENES.includes(scene);
      // Only the scenes slice 4e brought write their outside stairs and
      // their open doorways, so the others re-export byte for byte.
      const staired = STAIRED_SCENES.includes(scene);
      const piers = [], lanterns = [], boats = [];
      const innerBridge = bridge, innerStilt = stiltHouse, innerLantern = lantern, innerBoat = boat;
      // A scene's fires (4e, 4g): braziers, torches and fire pits, each where
      // it stands; and its outside stairs, each a straight flight from its
      // foot to its head.
      const fires = [], stairs = [];
      const innerBrazier = brazier, innerTorch = torch, innerFirePit = firePit, innerStairRun = stairRun, innerCactus = cactus, innerScatter = scatter;
      let inStilt = 0, inScatter = 0;
      // The tundra's castle and camp (4g): its keep, its towers and its
      // curtain wall from their own calls (the town's are found by its area
      // names), and its igloos, a building each.
      const keeps = [], towers = [], curtains = [], igloos = [];
      const innerRingKeep = ringKeep, innerWallTower = wallTower, innerCurtain = curtainWall, innerIgloo = igloo;
      // eslint-disable-next-line no-global-assign
      ringKeep = function (kc, kr, o) {
        if (staired) keeps.push({ c: kc, r: kr, name: o.name || "Keep", kit: o.out, spire: o.spire || 2.4 });
        return innerRingKeep(kc, kr, o);
      };
      // eslint-disable-next-line no-global-assign
      wallTower = function (c, r, wallDir, name, o = {}) {
        if (staired) towers.push({ c, r, name, kit: o.mat || "stone", spire: o.roofH ?? 3.2 });
        return innerWallTower(c, r, wallDir, name, o);
      };
      // eslint-disable-next-line no-global-assign
      curtainWall = function (cells, top, gates, mat, topMat, isOutside) {
        if (staired) {
          const inSet = (c, r) => cells.some(([a, b]) => a === c && b === r);
          for (const [c, r] of cells) {
            const base = TOP[idx(c, r)], gated = gates.some(([a, b]) => a === c && b === r), merlons = [];
            for (let d = 0; d < 6; d++) {
              const [c2, r2] = nb(c, r, d);
              if (!inSet(c2, r2) && isOutside(c2, r2)) merlons.push(d);
            }
            // The mockup's gate arch springs 3.6 m up; the town's 4.
            curtains.push({ c, r, from: gated ? Math.round(base + 3.6) : base, to: top, merlons });
          }
          curtains.material = { wall: mat, top: topMat };
        }
        return innerCurtain(cells, top, gates, mat, topMat, isOutside);
      };
      // eslint-disable-next-line no-global-assign
      igloo = function (c, r, door, name = "Igloo") {
        const out = innerIgloo(c, r, door, name);
        if (staired) igloos.push({ c, r, door, name, base: TOP[idx(c, r)] });
        return out;
      };
      // eslint-disable-next-line no-global-assign
      brazier = function (x, y, z) { if (dressed && !sea) fires.push({ kind: "brazier", x, y, z }); return innerBrazier(x, y, z); };
      // eslint-disable-next-line no-global-assign
      torch = function (x, y, z, h) { if (dressed && !sea) fires.push({ kind: "torch", x, y, z }); return innerTorch(x, y, z, h); };
      // eslint-disable-next-line no-global-assign
      firePit = function (x, y, z) { if (dressed && !sea) fires.push({ kind: "fire", x, y, z }); return innerFirePit(x, y, z); };
      // eslint-disable-next-line no-global-assign
      stairRun = function (A, B, o = {}) {
        if (staired && !inStilt && !depth) stairs.push({ from: A.slice(), to: B.slice(), width_m: o.width ?? 1.1, material: o.m || "stone" });
        return innerStairRun(A, B, o);
      };
      // A cactus the scene stands, not one its scatter strews over the wild:
      // those are the planet's own flora.
      // eslint-disable-next-line no-global-assign
      scatter = function (...a) { inScatter++; try { return innerScatter(...a); } finally { inScatter--; } };
      // eslint-disable-next-line no-global-assign
      cactus = function (x, y, z, s = 1) {
        if (dressed && !sea && !inScatter) dressing.push({ kind: "cactus", x, y, z, scale: s });
        return innerCactus(x, y, z, s);
      };
      // A boat on the water (not beached, not still in a boathouse): its kind,
      // where it lies and its heading, the bow along (cos, sin) in (x, z)
      // (`cities-in-the-world` task 4.2b).
      // eslint-disable-next-line no-global-assign
      boat = function (type, x, z, ang, o = {}) {
        if (sea && !o.beached && !o.still) boats.push({ kind: type, x, z, heading: ang });
        // A boat on land is dressing: keel up on trestles on the beach, or on
        // the sand in a boathouse (task 4.2c).
        else if (sea) dressing.push({ kind: "boat", boat: type, x, y: o.beached ? o.y ?? 0 : (o.wl ?? 0) - BOATS[type].draft, z, angle: ang, beached: !!o.beached });
        return innerBoat(type, x, z, ang, o);
      };
      // eslint-disable-next-line no-global-assign
      bridge = function (A, B, o = {}) {
        if (sea && o.piles !== undefined) piers.push({ from: A.slice(), to: B.slice(), width_m: o.width ?? 2 });
        if (sea && o.label === "The slip") slip = { from: A.slice(), to: B.slice(), width_m: o.width ?? 1.2 };
        if (sea && o.label === "Gangplank") gangplank = { from: A.slice(), to: B.slice(), width_m: o.width ?? 1.2 };
        return innerBridge(A, B, o);
      };
      // The harbour's dressing (task 4.2c), each thing where the mockup puts
      // it, after its own `fitOut`: the leaf calls are wrapped, and a flag
      // says which dressing call they are inside. The cog's barrel and
      // crate are the cog's (`sail-the-cog`). What stands in a house (a
      // cooper's barrels) is its furniture, not the town's dressing.
      const dressing = [];
      let slip = null, inside = 0, // in the cog or a house
        inStall = null, inCrate = 0, potFoot = null;
      const innerHouse = townHouse, innerBox = box, innerSolidBox = solidBox, innerSolidCyl = solidCyl, innerCog = cog,
        innerStall = marketStall, innerCrate = crate, innerNetRack = netRack, innerFishRack = fishRack, innerPots = lobsterPots;
      // A pile of pots: each pot, and how far over the pile's foot it sits.
      // eslint-disable-next-line no-global-assign
      lobsterPots = function (x, y, z, n) { potFoot = y; try { return innerPots(x, y, z, n); } finally { potFoot = null; } };
      // The cog (`sail-the-cog` design 6, step 1): where it lies, its
      // heading and the side its gangway opens on.
      let cogAt = null, gangplank = null;
      // eslint-disable-next-line no-global-assign
      cog = function (x, z, ang, gangSide, o = {}) {
        if (sea) cogAt = { x, z, heading: ang, gang_side: gangSide };
        inside++; try { return innerCog(x, z, ang, gangSide, o); } finally { inside--; }
      };
      // eslint-disable-next-line no-global-assign
      townHouse = function (...a) { inside++; try { return innerHouse(...a); } finally { inside--; } };
      // eslint-disable-next-line no-global-assign
      marketStall = function (c, r, cloth, y, goods) {
        const stall = { kind: "stall", x: cx(c, r), y, z: cz(c, r), cloth, goods: [] };
        inStall = stall;
        try { return innerStall(c, r, cloth, y, goods); } finally { inStall = null; if (dressed) dressing.push(stall); }
      };
      // eslint-disable-next-line no-global-assign
      crate = function (...a) { inCrate++; try { return innerCrate(...a); } finally { inCrate--; } };
      // eslint-disable-next-line no-global-assign
      netRack = function (x, y, z, ang) {
        if (sea) dressing.push({ kind: "net_rack", x, y, z, angle: ang });
        return innerNetRack(x, y, z, ang);
      };
      // eslint-disable-next-line no-global-assign
      fishRack = function (x, y, z, ang) {
        if (sea) dressing.push({ kind: "fish_rack", x, y, z, angle: ang });
        return innerFishRack(x, y, z, ang);
      };
      // eslint-disable-next-line no-global-assign
      box = function (m, x, y0, z, sx, sy, sz, ang = 0, uvf) {
        if (dressed && inStall && sy === 0.14) inStall.goods.push({ material: m, x, z });
        if (sea && !inside && m === "timber" && sx === 0.07 && sy === 2.6) dressing.push({ kind: "oar", x, y: y0, z });
        return innerBox(m, x, y0, z, sx, sy, sz, ang, uvf);
      };
      // eslint-disable-next-line no-global-assign
      solidBox = function (m, x, y0, z, sx, sy, sz, ang = 0, opt = {}) {
        if (sea && !inside && opt.label === "Lobster pots") dressing.push({ kind: "pot", x, y: potFoot ?? y0, z, angle: ang, lift_m: y0 - (potFoot ?? y0) });
        else if (dressed && !inside && inCrate) dressing.push({ kind: "crate", x, y: y0, z, angle: ang, side_m: sx });
        return innerSolidBox(m, x, y0, z, sx, sy, sz, ang, opt);
      };
      // eslint-disable-next-line no-global-assign
      solidCyl = function (m, x, y0, z, r, h, opt = {}) {
        if (sea && !inside && m === "barrel") dressing.push({ kind: "barrel", x, y: y0, z });
        else if (sea && opt.label === "Bollard") dressing.push({ kind: "bollard", x, y: y0, z, radius_m: r, height_m: h });
        return innerSolidCyl(m, x, y0, z, r, h, opt);
      };
      // eslint-disable-next-line no-global-assign
      stiltHouse = function (o) {
        inStilt++;
        let out;
        try { out = innerStilt(o); } finally { inStilt--; }
        if (sea) {
          const b = found[found.length - 1];
          b.stilts = { deck: o.deckCells.map(([c, r]) => [c, r]), porch: o.stairEdge.slice(0, 3), foot_m: o.footY };
        }
        return out;
      };
      // eslint-disable-next-line no-global-assign
      lantern = function (x, y, z, post = true) {
        if (dressed) lanterns.push([x, y, z]);
        return innerLantern(x, y, z, post);
      };
      // eslint-disable-next-line no-global-assign
      newelStair = function (c, r, entry, y0, yTop, o = {}) {
        if (depth === 0) newels.push({ c, r, entry, y0, yTop, wallTop: o.wallTop ?? yTop + 2.6, exits: (o.exits || []).map((e) => [e.d, e.y]) });
        return innerNewel(c, r, entry, y0, yTop, o);
      };
      // eslint-disable-next-line no-global-assign
      edgeWall = function (c, r, d, y0, y1, o = {}) {
        if (depth === 0) edges.push({ c, r, d, y0, openings: (o.openings || []).map((p) => ({ door: !!p.door, win: !!p.win })) });
        return innerEdge(c, r, d, y0, y1, o);
      };
      // eslint-disable-next-line no-global-assign
      building = function (o) {
        depth++;
        let out;
        try {
          out = inner(o);
        } finally {
          depth--;
        }
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
        const open = [];
        if (o.skipWall) for (const [c, r] of cells) for (let d = 0; d < 6; d++) {
          const [c2, r2] = nb(c, r, d);
          if (!inB(c2, r2) && o.skipWall(c, r, d)) open.push([c, r, d]);
        }
        found.push({
          name: o.name || "",
          kit: o.kit || "halftimber",
          cells,
          base: sea ? Math.round(out.base) : out.base,
          storeys,
          tall: o.tall || 1,
          doors: (o.doors || []).map((e) => [e[0], e[1], e[2], e[3] || 0]),
          windows,
          roof: o.roof || kit.roof,
          pitch: o.pitch ?? kit.pitch ?? 1,
          chimney: o.chimney || null,
          stair_cells: o.stairCells || [],
          ...(open.length ? { open } : {}),
          // A walked flat roof's parapet, but at these edges (4e): where an
          // outside stair comes up.
          ...(o.parapetGaps ? { parapet_gaps: o.parapetGaps.map((g) => g.slice(0, 3)) } : {}),
          // Doorways with no leaf, the mockup's "open" doors (4e).
          ...(staired && (o.doors || []).some((e) => e[4] === "open") ? { archways: o.doors.filter((e) => e[4] === "open").map((e) => [e[0], e[1], e[2], e[3] || 0]) } : {}),
        });
        return out;
      };
      try {
        resetWorld();
        SCENES[scene].build();
      } finally {
        building = inner;
        newelStair = innerNewel;
        edgeWall = innerEdge;
        bridge = innerBridge;
        stiltHouse = innerStilt;
        lantern = innerLantern;
        brazier = innerBrazier;
        torch = innerTorch;
        firePit = innerFirePit;
        stairRun = innerStairRun;
        cactus = innerCactus;
        scatter = innerScatter;
        ringKeep = innerRingKeep;
        wallTower = innerWallTower;
        curtainWall = innerCurtain;
        igloo = innerIgloo;
        boat = innerBoat;
        cog = innerCog;
        townHouse = innerHouse;
        marketStall = innerStall;
        crate = innerCrate;
        netRack = innerNetRack;
        fishRack = innerFishRack;
        lobsterPots = innerPots;
        box = innerBox;
        solidBox = innerSolidBox;
        solidCyl = innerSolidCyl;
      }
      // A stair tower is its newel's one cell; the keep is its newel's cell
      // and the ring round it, its doors and windows as its walls were cut.
      if (scene === "town") for (const n of newels) {
        const name = AREA[idx(n.c, n.r)];
        const newel = { entry: n.entry, top_m: n.yTop - n.y0, wall_top_m: n.wallTop - n.y0, exits: n.exits.map(([d, y]) => [d, y - n.y0]) };
        if (/stair tower$/.test(name)) {
          found.push({ name, kit: "tower", cells: [[n.c, n.r]], base: n.y0, storeys: 1, tall: 1, doors: [[n.c, n.r, n.entry, 0]], windows: [], roof: "cone", pitch: 1, chimney: null, stair_cells: [[n.c, n.r]], newel });
          continue;
        }
        const ring = [0, 1, 2, 3, 4, 5].map((d) => nb(n.c, n.r, d));
        if (!ring.every(([c, r]) => AREA[idx(c, r)] === "The keep")) continue;
        const cells = [[n.c, n.r], ...ring];
        const doors = [], windows = [];
        for (const w of edges) {
          if (!cells.some(([a, b]) => a === w.c && b === w.r)) continue;
          const st = Math.round((w.y0 - n.y0) / STOREY);
          for (const p of w.openings) {
            if (p.door) doors.push([w.c, w.r, w.d, st]);
            else if (p.win) windows.push([w.c, w.r, w.d, st]);
          }
        }
        found.push({ name: "The keep", kit: "keep", cells, base: n.y0, storeys: Math.round((n.yTop - n.y0) / STOREY), tall: 1, doors, windows, roof: "flat", parapet: true, pitch: 1, chimney: null, stair_cells: [[n.c, n.r]], newel });
      }
      // The tundra's castle (4g): a keep or tower is its newel's cell, the
      // keep's ring round it too, its doors and windows as its walls were
      // cut, the spire over it as high as the call asked.
      const newelAt = (c, r) => newels.find((n) => n.c === c && n.r === r);
      const newelDef = (n, spire) => ({ entry: n.entry, top_m: n.yTop - n.y0, wall_top_m: n.wallTop - n.y0, exits: n.exits.map(([d, y]) => [d, y - n.y0]), spire_m: spire });
      for (const k of keeps) {
        const n = newelAt(k.c, k.r);
        if (!n) continue;
        const cells = [[n.c, n.r], ...[0, 1, 2, 3, 4, 5].map((d) => nb(n.c, n.r, d))];
        const doors = [], windows = [];
        for (const w of edges) {
          if (!cells.some(([a, b]) => a === w.c && b === w.r)) continue;
          const st = Math.round((w.y0 - n.y0) / STOREY);
          for (const p of w.openings) {
            if (p.door) doors.push([w.c, w.r, w.d, st]);
            else if (p.win) windows.push([w.c, w.r, w.d, st]);
          }
        }
        found.push({ name: k.name, kit: k.kit, cells, base: n.y0, storeys: Math.round((n.yTop - n.y0) / STOREY), tall: 1, doors, windows, roof: "flat", parapet: true, pitch: 1, chimney: null, stair_cells: [[n.c, n.r]], newel: newelDef(n, k.spire) });
      }
      for (const t of towers) {
        const n = newelAt(t.c, t.r);
        if (!n) continue;
        found.push({ name: t.name, kit: t.kit, cells: [[n.c, n.r]], base: n.y0, storeys: 1, tall: 1, doors: [[n.c, n.r, n.entry, 0]], windows: [], roof: "cone", pitch: 1, chimney: null, stair_cells: [[n.c, n.r]], newel: newelDef(n, t.spire) });
      }
      for (const g of igloos) {
        found.push({ name: g.name, kit: "igloo", cells: [[g.c, g.r]], base: g.base, storeys: 1, tall: 1, doors: [[g.c, g.r, g.door, 0]], windows: [], roof: "igloo", pitch: 1, chimney: null, stair_cells: [] });
      }
      const ground = [];
      // A frozen lake (4g) stands on whole layers, its ice over them.
      const frozen = [];
      let iceTop = 0;
      for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
        const i = idx(c, r);
        let h = sea ? Math.round(TOP[i]) : TOP[i];
        if (staired && TOPMAT[i] === "ice") {
          frozen.push([c, r]);
          iceTop = TOP[i];
          h = Math.floor(TOP[i]);
        }
        ground.push({ c, r, h, top: TOPMAT[i], area: AREA[i] });
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
      if (curtains.length) {
        out.masonry = curtains.slice();
        out.masonry_material = curtains.material;
      }
      if (frozen.length) out.frozen = { top_m: iceTop, cells: frozen };
      if (!sea && dressed) {
        if (lanterns.length) out.lanterns = lanterns;
        if (fires.length) out.fires = fires;
        if (stairs.length) out.stairs = stairs;
        if (dressing.length) out.dressing = dressing;
      }
      if (sea) {
        out.sea = true;
        out.piers = piers;
        // The one on the cog's stern is the cog's (`sail-the-cog`).
        out.lanterns = lanterns.filter(([, y]) => y < 2);
        out.boats = boats;
        // The mole's light: the round solid the scene labels so.
        const light = SOLIDS.find((x) => x.type === "circ" && x.label === "The light");
        if (light) out.light = { x: light.x, z: light.z, radius_m: light.r, base_m: Math.round(light.y0), top_m: light.y1 };
        out.dressing = dressing;
        // The shipyard: its hull in frame where the scene's solid labels it,
        // the slip, and the stack of planks.
        const hull = SOLIDS.find((x) => x.label === "A hull in frame");
        const planks = SOLIDS.find((x) => x.label === "Planks");
        const mid = (pts) => pts.reduce(([a, b], [x, z]) => [a + x / pts.length, b + z / pts.length], [0, 0]);
        if (cogAt && gangplank) out.cog = { ...cogAt, gangplank };
        if (hull && slip && planks) {
          const [hx, hz] = mid(hull.pts), [px, pz] = mid(planks.pts);
          out.shipyard = { x: hx, z: hz, slip, planks: [px, planks.y0, pz] };
        }
      }
      return out;
    }, { scene, SEA_SCENES: [...SEA], DRESSED_SCENES: [...DRESSED], STAIRED_SCENES: [...STAIRED] });
    if (WILD[scene]) layout.wild = WILD[scene];
    if (TOPS[scene]) layout.tops = TOPS[scene];
    if (TERRACED.has(scene)) layout.terraced = true;
    const file = path.join(OUT, `${scene}.json`);
    fs.writeFileSync(file, JSON.stringify(layout, null, 1) + "\n");
    console.log(`${scene}: ${layout.buildings.length} buildings -> ${path.relative(ROOT, file)}`);
  }
  if (errors.length) console.log("page errors:", errors.join("; "));
  await browser.close();
})();
