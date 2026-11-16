// Mission Tree mockup: the starter-missions tree (openspec/changes/starter-missions/design.md,
// "Missions are a tree that unlocks the tech" and "The full tree"), drawn as the game's Missions
// screen. Nothing moves until the viewer drags, zooms, steps or presses Play.
(function () {
  "use strict";
  const IMG = "mission-tree/items/";
  const UI = "mission-tree/ui/";

  // ---- what a mission can unlock: recipes and stations (a code the recipe list shows), or a skill
  // or branch (no recipe). new = a recipe the game does not have yet.
  const U = {
    "stone-axe": { name: "Stone axe", icon: "stone-axe", kind: "recipe" },
    felling: { name: "Felling", icon: "log", kind: "skill" },
    planks: { name: "Planks", icon: "planks", kind: "recipe" },
    rope: { name: "Rope", icon: "rope", kind: "recipe" },
    workbench: { name: "Workbench", icon: "workbench", kind: "station" },
    "stone-pickaxe": { name: "Stone pickaxe", icon: "stone-pickaxe", kind: "recipe" },
    "stone-shovel": { name: "Stone shovel", icon: "stone-shovel", kind: "recipe" },
    rod: { name: "Fishing rod", icon: "rod", kind: "recipe" },
    branches: { name: "The four branches", kind: "branch" },
    casting: { name: "Casting", icon: "rod", kind: "skill" },
    bucket: { name: "Bucket", icon: "bucket", kind: "recipe" },
    "cooking-branch": { name: "Cooking branch", icon: "campfire", kind: "branch" },
    tilling: { name: "Tilling with the shovel", icon: "stone-shovel", kind: "skill" },
    "turnip-seed": { name: "Turnip seed", icon: "turnip-seed", kind: "recipe" },
    "chili-seed": { name: "Chili seed", icon: "chili-seed", kind: "recipe" },
    "farm-dishes": { name: "Farm dishes", icon: "roast-turnip", kind: "recipe" },
    "egg-dishes": { name: "Egg dishes", icon: "fried-egg", kind: "recipe" },
    forge: { name: "Forge", icon: "forge", kind: "station" },
    bronze: { name: "Bronze ingot", icon: "bronze-ingot", kind: "recipe" },
    "bronze-axe": { name: "Bronze axe", icon: "bronze-axe", kind: "recipe" },
    "bronze-pickaxe": { name: "Bronze pickaxe", icon: "bronze-pickaxe", kind: "recipe" },
    "bronze-shovel": { name: "Bronze shovel", icon: "bronze-shovel", kind: "recipe" },
    "iron-smelting": { name: "Iron ingot", icon: "iron-ingot", kind: "recipe" },
    "iron-axe": { name: "Iron axe", icon: "iron-axe", kind: "recipe" },
    "iron-shovel": { name: "Iron shovel", icon: "iron-shovel", kind: "recipe" },
    "campfire-dishes": { name: "Campfire dishes", icon: "grilled-fish", kind: "recipe" },
    kitchen: { name: "Kitchen stations (when cooking builds them)", kind: "station" },
    torches: { name: "Torch", icon: "torch", kind: "recipe", new: true },
    lanterns: { name: "Lanterns (when they get a recipe)", icon: "lantern_post", kind: "recipe" },
  };

  const BRANCHES = [
    { key: "trunk", name: "0 · HANDS TO TOOLS" },
    { key: "fishing", name: "1 · FISHING" },
    { key: "farming", name: "2 · FARMING" },
    { key: "herding", name: "3 · HERDING" },
    { key: "mining", name: "4 · MINING & SMELTING" },
    { key: "cooking", name: "5 · COOKING" },
    { key: "light", name: "6 · LIGHT & BUILDING" },
    { key: "about", name: "7 · GETTING ABOUT" },
  ];

  // An objective: s = the card's short line, n = its count, t = the full line, nw = a new kind or
  // recipe.
  const o = (s, n, t, nw) => ({ s, n, t: t || s, nw: !!nw });

  // A strict tree (owner, 2026-11-15: "a child node cannot have multiple parents, parents can have
  // multiple child nodes. if something depends on something to be done, simply dont unlock it until
  // the dependent thing is done"). needs: the one parent, the only line drawn. after: missions that
  // must also be done before it opens, never drawn. col/row are worked out from the tree below.
  const M = [
    // 0. Trunk
    { id: "gather", b: "trunk", col: 0, row: 3, icon: "hand", title: "Gather by hand", needs: [],
      obj: [o("gather fibre", 8, "gather 8 fibre by hand"), o("gather stone", 9, "gather 9 stone by hand"), o("gather soil", 6, "gather 6 soil by hand")],
      unl: ["stone-axe"], help: "Dig soft ground with bare hands (fibre turns up in grass), and hit a tree by hand to pull down a branch." },
    { id: "axe", b: "trunk", col: 1, row: 3, icon: "stone-axe", title: "Craft an axe", needs: [["gather"]],
      obj: [o("craft a stone axe", 1, "craft stone-axe (in the pack)")],
      unl: ["felling"], help: "In the pack (I), the Craft row makes a stone axe from stone, fibre and a stick." },
    { id: "wood", b: "trunk", col: 2, row: 3, icon: "log", title: "Gather wood", needs: [["axe"]],
      obj: [o("fell trees", 2, "fell 2 trees with the axe", true), o("gather logs", 10, "gather 10 log")],
      unl: ["planks", "rope"], help: "Hold the axe and strike a trunk until the tree falls; its logs drop where it lands." },
    { id: "planks-rope", b: "trunk", col: 3, row: 3, icon: "rope", title: "Planks and rope", needs: [["wood"]],
      obj: [o("craft planks", 8, "craft 8 planks"), o("craft rope", 2, "craft 2 rope")],
      unl: ["workbench"], help: "In the pack (I), the Craft row makes planks from a log and rope from three fibre." },
    { id: "crafting-table", b: "trunk", col: 4, row: 3, icon: "workbench", title: "Build a crafting bench", needs: [["planks-rope"]],
      obj: [o("craft a workbench", 1, "craft workbench"), o("place it", 1, "place workbench")],
      unl: ["stone-pickaxe", "stone-shovel", "rod"], help: "Make a workbench from six planks in the Craft row, then place it on the ground." },
    { id: "stone-tools", b: "trunk", col: 5, row: 3, icon: "stone-pickaxe", title: "Stone tools", needs: [["crafting-table"]],
      obj: [o("craft a stone pickaxe", 1, "craft stone-pickaxe"), o("craft a stone shovel", 1, "craft stone-shovel")],
      unl: ["branches"], help: "Loose stone comes off rock outcrops by hand. At the workbench (E), make a stone pickaxe and a stone shovel." },

    // 1. Fishing
    { id: "rod", b: "fishing", col: 6, row: 2, icon: "rod", title: "Make a fishing rod", needs: [["fire"]],
      obj: [o("craft a rod", 1, "craft rod")], unl: ["casting"], help: "Make a rod at the workbench from planks, rope and fibre." },
    { id: "go-fishing", b: "fishing", col: 7, row: 2, icon: "perch", title: "Go fishing", needs: [["rod"]],
      obj: [o("cast onto water", 1), o("catch fish", 3, "catch 3 fish")], unl: ["bucket"],
      help: "Hold the rod (G), hold the left button and let go to cast the float onto water. When it goes under, click and reel." },
    { id: "bait", b: "fishing", col: 8, row: 2, icon: "stone-shovel", title: "Dig for bait", needs: [["go-fishing"]],
      obj: [o("dig bait", 3, "dig bait from wet ground"), o("catch with bait", 3, "catch 3 fish with bait")], unl: [],
      help: "Dig wet ground by the water for bait, and cast again with it on the hook." },
    { id: "three-kinds", b: "fishing", col: 9, row: 2, icon: "silverfin", title: "Three kinds of fish", needs: [["bait"]],
      obj: [o("catch species", 3, "catch 3 different species (the field guide, J, fills in)")], unl: ["cooking-branch"],
      help: "Fish in other water and at other hours; the field guide (J) fills in each new kind." },
    { id: "sea-fishing", b: "fishing", col: 10, row: 2, icon: "eel", title: "Off the shore", needs: [["three-kinds"]],
      obj: [o("catch in the sea", 3, "catch 3 fish from the sea"), o("catch in fresh water", 3, "catch 3 fish from a river or lake")], unl: [],
      help: "Cast from the shore into the sea, then from a river bank or a lake." },

    // 2. Farming
    { id: "find-seeds", b: "farming", col: 6, row: 4, icon: "chili-seed", title: "Find seeds", needs: [["fire"]],
      obj: [o("dig grass for seeds", 5), o("pick wild plants", 5, "pick 5 wild plants")], unl: ["tilling"],
      help: "Seeds turn up when you dig grass, and wild plants give them when picked (E)." },
    { id: "till", b: "farming", col: 7, row: 4, icon: "stone-shovel", title: "Break ground", needs: [["find-seeds"]],
      obj: [o("till cells", 6, "till 6 cells")], unl: [], help: "With the shovel in hand, E tills soil." },
    { id: "plant", b: "farming", col: 8, row: 4, icon: "wheat-seed", title: "Plant a field", needs: [["till"]],
      obj: [o("plant wheat seed", 6, "plant 6 wheat-seed")], unl: [], help: "Plant a seed on tilled soil with the right button." },
    { id: "water-field", b: "farming", col: 9, row: 4, icon: "water-bucket", title: "Water the field", needs: [["plant"]], after: ["go-fishing"],
      obj: [o("pour a water bucket", 1, "pour a water-bucket beside the field", true)], unl: [],
      help: "Fill the bucket at water (E), then pour it beside the field." },
    { id: "harvest", b: "farming", col: 10, row: 4, icon: "wheat", title: "Harvest", needs: [["water-field"]],
      obj: [o("harvest wheat", 6, "harvest 6 wheat")], unl: ["turnip-seed", "chili-seed"],
      help: "A crop is ripe when it is full grown: E on it harvests produce and seed." },
    { id: "three-crops", b: "farming", col: 11, row: 4, icon: "turnip", title: "A mixed field", needs: [["harvest"]],
      obj: [o("harvest turnip", 3), o("harvest chili", 3), o("harvest frost kale", 3, "harvest frost-kale (kale where it is cold)")], unl: ["farm-dishes"],
      help: "Turnip and chili seeds come with the first harvest; frost kale grows where it is cold." },

    // 3. Herding
    { id: "fence", b: "herding", col: 6, row: 5, icon: "fence", title: "Make fences", needs: [["fire"]],
      obj: [o("craft fences", 12, "craft 12 fence (new recipe, from planks and rope)", true), o("craft a gate", 1, "craft 1 gate (new recipe, from planks and rope)", true)], unl: [],
      help: "At the workbench, make fences and a gate from planks and rope." },
    { id: "tame", b: "herding", col: 7, row: 5, icon: "yard_hen", title: "Tame an animal", needs: [["fence"]],
      obj: [o("tame a yard hen", 1, "tame a Yard hen by feeding")], unl: [],
      help: "Stand still with its food in hand and let a wild animal come to you, then feed it." },
    { id: "pen", b: "herding", col: 8, row: 5, icon: "gate", title: "Build a pen", needs: [["tame"]],
      obj: [o("build a pen with a gate", 1, "build a pen with a gate round tamed animals")], unl: [],
      help: "Set fences on the cell edges round a yard, close it with a gate, and lead your animal in." },
    { id: "collect", b: "herding", col: 9, row: 5, icon: "egg", title: "Collect from the herd", needs: [["pen"]],
      obj: [o("collect eggs", 5, "collect 5 eggs")], unl: [],
      help: "A kept animal in a pen gives an egg, wool or milk once a day: E on it collects." },
    { id: "flock", b: "herding", col: 10, row: 5, icon: "wool", title: "A mixed herd", needs: [["collect"]],
      obj: [o("tame a woolback or plodder", 1, "tame a Woolback or a Plodder"), o("collect wool or milk", 3)], unl: ["egg-dishes"],
      help: "Woolbacks and plodders graze in open country; tame one as you did the hen." },

    // 4. Mining and smelting
    { id: "dig-deep", b: "mining", col: 6, row: 6, icon: "stone", title: "Dig into the hill", needs: [["fire"]],
      obj: [o("dig stone", 10, "dig 10 stone with the stone pickaxe")], unl: ["forge"],
      help: "Hold the stone pickaxe and dig into a hillside; stone comes out a cell at a time." },
    { id: "ore", b: "mining", col: 7, row: 6, icon: "tin-ore", title: "Find ore", needs: [["dig-deep"]],
      obj: [o("dig mixed ore", 4, "dig mixed-ore (copper and tin)"), o("dig coal", 3)], unl: [],
      help: "Copper and tin show in the rock as mixed ore; coal sits in dark seams." },
    { id: "forge", b: "mining", col: 8, row: 6, icon: "forge", title: "Build a forge", needs: [["ore"]],
      obj: [o("craft a forge", 1, "craft forge"), o("place it", 1, "place forge")], unl: ["bronze"],
      help: "Make a forge at the workbench and place it on the ground." },
    { id: "smelt", b: "mining", col: 9, row: 6, icon: "bronze-ingot", title: "Smelt bronze", needs: [["forge"]],
      obj: [o("smelt bronze ingots", 2, "smelt 2 bronze-ingot")], unl: ["bronze-axe", "bronze-pickaxe", "bronze-shovel"],
      help: "Put ore and coal in the forge (E) and smelt bronze." },
    { id: "bronze-tool", b: "mining", col: 10, row: 6, icon: "bronze-pickaxe", title: "Bronze tools", needs: [["smelt"]],
      obj: [o("craft a bronze pickaxe", 1, "craft bronze-pickaxe"), o("dig stone with it", 5, "dig 5 stone with it")], unl: ["iron-smelting"],
      help: "At the workbench, make a bronze pickaxe from ingots; it digs faster than stone." },
    { id: "iron", b: "mining", col: 11, row: 6, icon: "iron-pickaxe", title: "The iron age", needs: [["bronze-tool"]],
      obj: [o("smelt iron ingots", 3, "smelt 3 iron ingots"), o("craft an iron pickaxe", 1, "craft iron-pickaxe")], unl: ["iron-axe", "iron-shovel"],
      help: "Iron ore smelts hotter: feed the forge more coal." },

    // 5. Cooking
    { id: "fire", b: "trunk", col: 13, row: 3, icon: "campfire", title: "Make a fire", needs: [["stone-tools"]],
      obj: [o("craft a campfire", 1, "craft a campfire (new recipe)", true), o("place it", 1, "place the campfire")], unl: ["campfire-dishes"],
      help: "Make a campfire in the Craft row from five loose stone and two logs, and set it down near water." },
    { id: "cook-fish", b: "cooking", col: 14, row: 3, icon: "grilled-fish", title: "Cook fish", needs: [["go-fishing"]],
      obj: [o("cook grilled fish", 2, "cook 2 grilled-fish")], unl: [], help: "Open the campfire with E and grill the fish you caught." },
    { id: "bread", b: "farming", col: 15, row: 3, icon: "flatbread", title: "Bake bread", needs: [["harvest"]],
      obj: [o("cook flatbread", 2, "cook 2 flatbread from your wheat")], unl: [], help: "At the campfire, bake flatbread from wheat you grew." },
    { id: "eat", b: "cooking", col: 16, row: 3, icon: "kale-skewer", title: "Eat well", needs: [["cook-fish"]],
      obj: [o("eat a dish", 1, "eat 1 dish; its effect shows", true)], unl: [],
      help: "Put a dish on the hotbar and use it; its effect shows beside the health bar." },
    { id: "farm-meal", b: "farming", col: 17, row: 3, icon: "roast-turnip", title: "A farm meal", needs: [["three-crops"]],
      obj: [o("cook a farm dish", 1, "cook roast-turnip, kale-skewer or chili-fish")], unl: [],
      help: "At the campfire, cook a dish from your field." },
    { id: "fried-egg", b: "herding", col: 18, row: 3, icon: "fried-egg", title: "Breakfast", needs: [["collect"]],
      obj: [o("cook a fried egg", 1, "cook a fried-egg from your hen's egg")], unl: [],
      help: "At the campfire, fry an egg from your own hens." },
    { id: "feast", b: "farming", col: 19, row: 3, icon: "chili-fish", title: "A feast", needs: [["farm-meal"]],
      obj: [o("eat different dishes", 3, "eat 3 different dishes", true)], unl: ["kitchen"],
      help: "Eat three different dishes; every effect shows at once." },

    // 6. Light and building
    { id: "build", b: "light", col: 6, row: 0, icon: "stone", title: "Build a shelter", needs: [["fire"]],
      obj: [o("place blocks", 20, "place 20 blocks (planks, stone or soil)")], unl: ["torches"],
      help: "Hold planks, stone or soil and place blocks with the right button: walls and a roof." },
    { id: "torch", b: "light", col: 7, row: 0, icon: "torch", title: "Light the night", needs: [["build"]],
      obj: [o("craft torches", 4, "craft 4 torches (new recipe)", true), o("place torches", 4, "place 4 torches"), o("be there after 20:00", 1, "be beside them after 20:00", true)], unl: ["lanterns"],
      help: "Set torches round your shelter and stay by them as night falls." },

    // 7. Getting about
    { id: "map", b: "about", col: 8, row: 1, icon: "icon-map", title: "Read the map", needs: [["go-fishing"]],
      obj: [o("open the map", 1, "open the map (M)"), o("reach a marked place", 1, "reach a marked place 200 m away", true)], unl: [],
      help: "Open the map (M), mark a place 200 m off, and walk there." },
    { id: "swim", b: "about", col: 9, row: 1, icon: "sea_otter", title: "Into the water", needs: [["map"]],
      obj: [o("swim (m)", 20, "swim 20 m", true)], unl: [], help: "Walk into deep water and swim." },
    { id: "reef", b: "about", col: 10, row: 1, icon: "reef", title: "See a reef", needs: [["swim"]],
      obj: [o("reach a reef or kelp", 1, "reach a reef or kelp forest under water", true)], unl: [],
      help: "Swim down where the sea is warm and shallow." },
    { id: "town", b: "about", col: 11, row: 1, icon: "lantern_post", title: "Find a town", needs: [["reef"]],
      obj: [o("reach a town", 1, "reach a town", true)], unl: [], help: "Towns show on the map; follow the compass to one." },
    { id: "boat", b: "about", col: 12, row: 1, icon: "planks", title: "Take a boat", needs: [["town"]],
      obj: [o("board a boat", 1, "board a rowboat or a cog", true), o("travel (m)", 100, "travel 100 m")], unl: [],
      help: "Board a rowboat or a cog at a town's quay (E) and head out." },
  ];

  const byId = Object.fromEntries(M.map((m) => [m.id, m]));
  const byIdRaw = byId;
  const order = BRANCHES.flatMap((b) => M.filter((m) => m.b === b.key).map((m) => m.id));
  const unlockedBy = {};
  M.forEach((m) => m.unl.forEach((u) => (unlockedBy[u] = m.id)));

  // ---- the bot's steps: Now, each objective filling (a count of 3 or more fills in two steps), Done.
  const steps = [];
  order.forEach((id) => {
    const m = byId[id];
    steps.push({ t: "now", m: id });
    m.obj.forEach((ob, i) => {
      if (ob.n >= 3) steps.push({ t: "obj", m: id, i, v: Math.floor(ob.n / 2) });
      steps.push({ t: "obj", m: id, i, v: ob.n });
    });
    steps.push({ t: "done", m: id });
  });

  const isOpen = (m, done) => m.needs.every((g) => g.some((id) => done.has(id))) && (m.after || []).every((id) => done.has(id));
  const unlName = (u) => U[u].name.replace(/ \(.*\)$/, "");

  function stateAt(k) {
    const done = new Set();
    const prog = {};
    let cur = null;
    const log = [];
    let fresh = [];
    let lit = null;
    for (let i = 0; i < k; i++) {
      const s = steps[i];
      const m = byId[s.m];
      fresh = [];
      lit = null;
      if (s.t === "now") {
        cur = s.m;
        prog[s.m] = m.obj.map(() => 0);
        log.push({ text: "NOW: " + m.title });
      } else if (s.t === "obj") {
        prog[s.m][s.i] = s.v;
        log.push({ text: m.title + " · " + m.obj[s.i].s + " " + s.v + "/" + m.obj[s.i].n });
      } else {
        const before = new Set(M.filter((x) => !done.has(x.id) && isOpen(x, done)).map((x) => x.id));
        done.add(s.m);
        cur = null;
        lit = s.m;
        const u = m.unl.length ? " · unlocked " + m.unl.map(unlName).join(", ") : "";
        log.push({ text: "DONE: " + m.title + u, d: true });
        fresh = M.filter((x) => !done.has(x.id) && !before.has(x.id) && isOpen(x, done)).map((x) => x.id);
        if (fresh.length) log.push({ text: "OPEN: " + fresh.map((id) => byId[id].title).join(", ") });
      }
    }
    return { k, done, prog, cur, log, fresh: k > 0 && steps[k - 1].t === "done" ? fresh : [], lit };
  }

  function standing(id, st) {
    if (st.done.has(id)) return "done";
    if (st.cur === id) return "now";
    return isOpen(byId[id], st.done) ? "open" : "locked";
  }

  // ---- layout. Every card keeps one shape at every zoom (owner, 2026-11-16: "chaning shape of
  // these nodes is annoying, I wish they stayed the same size, and maybe had 2x spacing", "this
  // shape is best" of the full card): the full card, its frame 10 CSS px = 5 art px, laid out with
  // twice the gaps the native view had. A zoom level is a scale of the whole tree, in fixed steps;
  // below one the frames are smoothed rather than pixel-sampled, so a 9-slice never tears.
  const CARD = { cw: 236, ch: 188, gx: 220, gy: 208, lanes: true };
  const SCALES = [0.1, 0.14, 0.2, 0.3, 0.42, 0.6, 0.8, 1, 1.4, 2];
  const LEVELS = SCALES.map(() => CARD);
  const OPEN = 7;
  let lv = OPEN;
  let CW, CH, GX, GY, OX = 40, OY;
  const setLevel = (n) => {
    lv = n;
    ({ cw: CW, ch: CH, gx: GX, gy: GY } = LEVELS[n]);
    OY = LEVELS[n].lanes ? 90 : 24;
  };
  setLevel(OPEN);
  const px = (c) => OX + c * (CW + GX);
  const py = (r) => OY + r * (CH + GY);
  {
    const kids = {};
    M.forEach((m) => { const p = m.needs.flat()[0]; if (p) (kids[p] = kids[p] || []).push(m); });
    const rank = (m) => BRANCHES.findIndex((b) => b.key === m.b);
    // the same branch's child first, so a branch reads as one straight line
    Object.entries(kids).forEach(([pid, list]) => {
      const pb = byIdRaw[pid].b;
      list.sort((a, b) => (b.b === pb) - (a.b === pb) || rank(a) - rank(b));
    });
    let next = 0;
    const place = (m, depth) => {
      m.col = depth;
      const list = kids[m.id] || [];
      if (!list.length) { m.row = next++; return; }
      list.forEach((k, i) => place(k, depth + 1));
      m.row = list[0].row;
    };
    M.filter((m) => !m.needs.length).forEach((m) => place(m, 0));
  }
  const cols = Math.max(...M.map((m) => m.col)) + 1;
  const rows = Math.max(...M.map((m) => m.row)) + 1;
  let W = 0, H = 0;
  const sizeAt = (n) => {
    const L = LEVELS[n], oy = L.lanes ? 90 : 24;
    return [OX + cols * (L.cw + L.gx) - L.gx + OX, oy + rows * (L.ch + L.gy) - L.gy + 40];
  };

  const $ = (s) => document.querySelector(s);
  const view = $("#view"), world = $("#world"), svg = $("#edges");
  // ---- hover: the full card beside the pointer, at full size, while the tree is zoomed out
  const tip = document.createElement("div");
  tip.className = "card tip";
  tip.setAttribute("aria-hidden", "true");
  document.body.appendChild(tip);
  let tipFor = null;
  function hideTip() { tip.style.display = "none"; tipFor = null; }
  function showTip(id, x, y) {
    const el = cards[id];
    if (tipFor !== id) {
      tip.className = el.className.replace(/\b(sel|fresh)\b/g, "") + " tip";
      tip.innerHTML = el.innerHTML;
      tipFor = id;
    }
    tip.style.display = "flex";
    const w = tip.offsetWidth, h = tip.offsetHeight;
    const left = x + 18 + w > innerWidth ? x - 18 - w : x + 18;
    const top = Math.min(Math.max(8, y - h / 2), innerHeight - h - 8);
    tip.style.left = left + "px";
    tip.style.top = top + "px";
  }


  // lane labels above each branch's first card
  const lanes = BRANCHES.map((b) => {
    const first = M.find((m) => m.b === b.key);
    const el = document.createElement("div");
    el.className = "lane";
    el.textContent = b.name;
    world.appendChild(el);
    return { el, first };
  });

  // edges
  const edges = [];
  M.forEach((m) => {
    m.needs.forEach((g) => g.forEach((from) => edges.push({ from, to: m.id, kind: g.length > 1 ? "or" : "need" })));
  });
  // One line per child: out of the parent's side, down the gap between the two columns, into the
  // child's side. A subtree owns its rows, so no line crosses a card or another subtree.
  edges.forEach((e) => {
    const p = document.createElementNS("http://www.w3.org/2000/svg", "path");
    p.setAttribute("fill", "none");
    e.el = p;
    svg.appendChild(p);
  });

  // cards
  const cards = {};
  M.forEach((m) => {
    const el = document.createElement("div");
    el.className = "card";
    el.dataset.id = m.id;
    el.tabIndex = 0;
    el.setAttribute("role", "button");
    world.appendChild(el);
    cards[m.id] = el;
  });

  // Size and place every card, line and plaque for the level.
  function relayout() {
    [W, H] = sizeAt(lv);
    // Zoomed out, a card is its mission's icon, big, in the same frame (owner, 2026-11-16: "it
    // should be big icons when zoomed out"); hovering one shows the full card.
    world.className = "lv4" + (SCALES[lv] < 1 ? " small" : "") + (SCALES[lv] <= 0.6 ? " icons" : "");
    hideTip();
    world.style.width = W + "px";
    world.style.height = H + "px";
    svg.setAttribute("width", W);
    svg.setAttribute("height", H);
    svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
    lanes.forEach(({ el, first }) => {
      el.style.left = px(first.col) + "px";
      el.style.top = py(first.row) - 46 + "px";
    });
    edges.forEach((e) => {
      const a = byId[e.from], b = byId[e.to];
      const x1 = px(a.col) + CW, y1 = py(a.row) + CH / 2, x2 = px(b.col), y2 = py(b.row) + CH / 2;
      const xm = x1 + GX / 2;
      e.el.setAttribute("d", a.row === b.row ? `M${x1},${y1} L${x2},${y2}` : `M${x1},${y1} L${xm},${y1} L${xm},${y2} L${x2},${y2}`);
    });
    M.forEach((m) => {
      const el = cards[m.id];
      el.style.left = px(m.col) + "px";
      el.style.top = py(m.row) + "px";
      el.style.width = CW + "px";
      el.style.height = CH + "px";
    });
  }
  relayout();

  const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  const icon = (name, cls) => `<img src="${IMG}${name}.png" alt="" class="${cls || ""}">`;
  const check = (on, cls) => `<img src="${UI}stone-check-${on ? "on" : "off"}.png" alt="${on ? "done" : ""}" class="ck ${cls || ""}">`;
  const needText = (m) => "after " + [...m.needs.flat(), ...(m.after || [])].map((id) => byId[id].title).join(" + ");
  const branchName = (m) => BRANCHES.find((b) => b.key === m.b).name.replace(/^\d · /, "");
  const WORD = { done: "DONE", now: "NOW", open: "OPEN", locked: "LOCKED" };

  function counts(m, st, s) {
    if (s === "done") return m.obj.map((ob) => ob.n);
    return st.prog[m.id] || m.obj.map(() => 0);
  }

  function unlockIcon(u, lit, sm) {
    const d = U[u];
    const tip = esc(d.name);
    if (!d.icon) return `<span class="chip${lit ? "" : " none"}" title="${tip}">${esc(unlName(u))}</span>`;
    return `<span class="slot sm ${lit ? "on" : "off"}" title="${tip}">${icon(d.icon, lit ? "" : "sil")}</span>`;
  }

  let st = null;
  let selected = null;

  function renderCard(m) {
    const s = standing(m.id, st);
    const el = cards[m.id];
    el.className = "card st-" + s + (selected === m.id ? " sel" : "") + (st.fresh.includes(m.id) ? " fresh" : "");
    const c = counts(m, st, s);
    const sub = s === "locked" ? needText(m) : branchName(m);
    const unlocks = m.unl.length
      ? m.unl.map((u) => unlockIcon(u, s === "done")).join("")
      : `<span class="chip none">—</span>`;
    el.setAttribute("aria-label", `${m.title}, ${WORD[s].toLowerCase()}`);
    el.innerHTML =
      `<div class="hd"><span class="slot">${icon(m.icon)}</span><div class="tt"><b>${esc(m.title)}</b><small>${esc(sub)}</small></div>` +
      (s === "done" ? check(true, "big") : `<span class="badge">${WORD[s]}</span>`) + `</div>` +
      `<ul class="obj">${m.obj
        .map((ob, i) => `<li class="${c[i] >= ob.n ? "full" : ""}">${check(c[i] >= ob.n)}<span>${esc(ob.s)}</span>${ob.nw ? '<span class="nw">NEW</span>' : ""}<em>${c[i]}/${ob.n}</em></li>`)
        .join("")}</ul>` +
      `<div class="un${st.lit === m.id ? " litrow" : ""}"><span class="lbl">UNLOCKS</span>${unlocks}</div>`;
    if (st.lit === m.id) el.querySelectorAll(".un .slot, .un .chip").forEach((x) => x.classList.add("lit"));
  }

  function renderEdges() {
    edges.forEach((e) => {
      const a = st.done.has(e.from);
      const b = standing(e.to, st);
      let cls = "e-" + (a && (b === "done") ? "done" : a ? "open" : "locked");
      if (e.kind === "or") cls += " e-or";
      if (e.kind === "uses") cls += " e-uses";
      if (selected && (e.to === selected || e.from === selected) && e.kind !== "uses") cls += " e-hl";
      e.el.setAttribute("class", cls);
    });
  }

  function renderHud() {
    const hud = $("#hud");
    const last = st.log[st.log.length - 1];
    if (st.cur) {
      const m = byId[st.cur];
      const c = st.prog[m.id];
      const i = m.obj.findIndex((ob, j) => c[j] < ob.n);
      const ob = m.obj[i < 0 ? m.obj.length - 1 : i];
      hud.textContent = `${ob.s} ${c[i < 0 ? m.obj.length - 1 : i]}/${ob.n}`;
      hud.className = "hud";
    } else if (last && last.d) {
      hud.textContent = last.text.split(" · ")[0];
      hud.className = "hud done";
    } else if (last) {
      hud.textContent = last.text.split(" · ")[0];
      hud.className = "hud";
    } else hud.textContent = "";
    const lines = st.log.slice(-5);
    $("#log").innerHTML = lines.map((l) => `<div class="${l.d ? "d" : ""}">${esc(l.text)}</div>`).join("");
    $("#stepNo").textContent = `step ${st.k}/${steps.length}`;
    $("#stepBtn").disabled = st.k >= steps.length;
  }

  // ---- side panel
  let tab = "m";
  function renderPanel() {
    const panel = $("#panel");
    if (!selected && tab === "m") { panel.classList.remove("open"); return; }
    panel.classList.add("open");
    $("#tabM").setAttribute("aria-selected", tab === "m");
    $("#tabR").setAttribute("aria-selected", tab === "r");
    $("#tabM").disabled = !selected;
    const body = $("#pbody");
    if (tab === "r") { body.innerHTML = recipesHtml(); return; }
    const m = byId[selected];
    const s = standing(m.id, st);
    const c = counts(m, st, s);
    const next = M.filter((x) => x.needs.some((g) => g.includes(m.id)));
    const users = M.filter((x) => (x.uses || []).includes(m.id));
    body.innerHTML =
      `<div class="who"><span class="slot">${icon(m.icon)}</span><div><h2>${esc(m.title)}</h2><div class="sub">${esc(branchName(m))} · ${WORD[s]}</div></div></div>` +
      (s === "locked" ? `<p class="help" style="margin-top:8px">Opens ${esc(needText(m))}.</p>` : "") +
      `<h3>OBJECTIVES</h3><ul class="pobj">${m.obj
        .map((ob, i) => `<li>${check(c[i] >= ob.n)}<span>${esc(ob.t)}${ob.nw ? '<span class="nw">NEW</span>' : ""}</span><em>${c[i]}/${ob.n}</em><span class="pb"><i style="width:${Math.round((100 * c[i]) / ob.n)}%"></i></span></li>`)
        .join("")}</ul>` +
      `<h3>HELP</h3><p class="help">${esc(m.help)}</p>` +
      `<h3>UNLOCKS</h3>` +
      (m.unl.length
        ? `<div class="ulist">${m.unl
            .map((u) => {
              const d = U[u];
              const lit = s === "done";
              return `<div>${d.icon ? `<span class="slot sm ${lit ? "on" : "off"}">${icon(d.icon, lit ? "" : "sil")}</span>` : `<span class="slot sm ${lit ? "on" : "off"}"></span>`}<span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}<small>${lit ? "unlocked" : "locked until this is done"}</small></span></div>`;
            })
            .join("")}</div>`
        : `<p class="help">Nothing: this one is for the play.</p>`) +
      (m.needs.length ? `<h3>NEEDS</h3><div class="mlinks">${m.needs.map((g) => g.map((id) => `<button data-go="${id}">${esc(byId[id].title)}</button>`).join(" or ")).join(" + ")}</div>` : "") +
      (users.length ? `<h3>FEEDS</h3><div class="mlinks">${users.map((x) => `<button data-go="${x.id}">${esc(x.title)}</button>`).join("")}</div>` : "") +
      (m.uses ? `<h3>USES</h3><div class="mlinks">${m.uses.map((id) => `<button data-go="${id}">${esc(byId[id].title)}</button>`).join("")}</div>` : "") +
      (next.length ? `<h3>OPENS NEXT</h3><div class="mlinks">${next.map((x) => `<button data-go="${x.id}">${esc(x.title)}</button>`).join("")}</div>` : "");
  }

  function recipesHtml() {
    const groups = [["recipe", "RECIPES"], ["station", "STATIONS"], ["skill", "SKILLS"], ["branch", "BRANCHES"]];
    return groups
      .map(([k, label]) => {
        const rows = Object.keys(U).filter((u) => U[u].kind === k && unlockedBy[u]);
        if (!rows.length) return "";
        return `<div class="rgroup">${label}</div>` + rows
          .map((u) => {
            const d = U[u];
            const by = byId[unlockedBy[u]];
            const lit = st.done.has(by.id);
            return `<button class="rrow ${lit ? "" : "locked"}" data-go="${by.id}"><span class="slot sm ${lit ? "on" : "off"}">${d.icon ? icon(d.icon, lit ? "" : "sil") : ""}</span><span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}<small>${lit ? "unlocked" : esc(by.title)}</small></span></button>`;
          })
          .join("");
      })
      .join("");
  }

  function render() {
    M.forEach(renderCard);
    renderEdges();
    renderHud();
    renderPanel();
  }

  function setStep(k) {
    st = stateAt(Math.max(0, Math.min(steps.length, k)));
    render();
  }

  function select(id) {
    selected = id;
    tab = "m";
    render();
  }

  $("#pbody").addEventListener("click", (e) => {
    const b = e.target.closest("[data-go]");
    if (b) select(b.dataset.go);
  });
  $("#closePanel").addEventListener("click", () => { selected = null; tab = "m"; render(); });
  $("#tabM").addEventListener("click", () => { tab = "m"; render(); });
  $("#tabR").addEventListener("click", () => { tab = "r"; render(); });
  $("#recipesBtn").addEventListener("click", () => { tab = "r"; render(); });

  // ---- pan and zoom: only by the viewer's hand. The wheel, the buttons and a pinch step through
  // the levels about the pointer; the pan is snapped to whole device pixels. On a screen whose
  // device pixel ratio does not make 2 CSS px a whole number of device pixels (1.25, say), the
  // world is scaled once by a fixed factor that does, never by the zoom.
  let tx = 0, ty = 0;
  const dpr = () => window.devicePixelRatio || 1;
  const unit = () => SCALES[lv] * Math.max(1, Math.round(2 * dpr())) / (2 * dpr());
  const snap = (v) => Math.round(v * dpr()) / dpr();
  function apply() {
    // Bounds (owner, 2026-11-16: "a max bounds so the user doesnt get lost"): the tree may slide
    // only until its edge reaches the middle of the view, so some of it is always in sight.
    // The pull is toward the nearest card, so at least 40 px of a card always stays in view.
    const q = unit(), r = view.getBoundingClientRect();
    let cx = (r.width / 2 - tx) / q, cy = (r.height / 2 - ty) / q;
    const hx = (r.width / 2 - 40) / q, hy = (r.height / 2 - 40) / q;
    let best = null;
    for (const m of M) {
      const x0 = px(m.col), y0 = py(m.row);
      const dx = Math.max(x0 - cx, 0, cx - (x0 + CW)), dy = Math.max(y0 - cy, 0, cy - (y0 + CH));
      const over = Math.max(dx - hx, 0) + Math.max(dy - hy, 0);
      if (!best || over < best.over) best = { over, x0, y0 };
      if (!over) break;
    }
    if (best && best.over) {
      cx = Math.min(Math.max(cx, best.x0 - hx), best.x0 + CW + hx);
      cy = Math.min(Math.max(cy, best.y0 - hy), best.y0 + CH + hy);
      tx = r.width / 2 - cx * q;
      ty = r.height / 2 - cy * q;
    }
    tx = snap(tx);
    ty = snap(ty);
    world.style.transform = `translate(${tx}px,${ty}px)` + (q !== 1 ? ` scale(${q})` : "");
    view.style.backgroundPosition = `${tx}px ${ty}px`;
  }
  // Step to level n, keeping the point at (cx, cy) between the same cards.
  function stepTo(n, cx, cy) {
    n = Math.max(0, Math.min(LEVELS.length - 1, n));
    if (n === lv) return;
    // The layout is the same at every level; only the scale changes, so the world point under
    // (cx, cy) is read at the old scale and put back under it at the new one.
    const q0 = unit();
    const wx = (cx - tx) / q0, wy = (cy - ty) / q0;
    setLevel(n);
    relayout();
    const q1 = unit();
    tx = cx - wx * q1;
    ty = cy - wy * q1;
    apply();
  }
  function fit() {
    const r = view.getBoundingClientRect();
    const pad = 12;
    const bottomRoom = Math.min(130, r.height * 0.22); // the log sits at the foot
    let n = LEVELS.length - 1;
    while (n > 0) {
      const [w, h] = sizeAt(n);
      const qn = SCALES[n] * Math.max(1, Math.round(2 * dpr())) / (2 * dpr());
      if (w * qn <= r.width - pad * 2 && h * qn <= r.height - pad - bottomRoom) break;
      n--;
    }
    setLevel(n);
    relayout();
    const q = unit();
    tx = (r.width - W * q) / 2;
    ty = pad + Math.max(0, (r.height - pad - bottomRoom - H * q) / 2);
    apply();
  }
  let spin = 0;
  view.addEventListener("wheel", (e) => {
    e.preventDefault();
    const r = view.getBoundingClientRect();
    spin += e.deltaMode === 1 ? e.deltaY * 40 : e.deltaY;
    const steps = Math.trunc(spin / 100);
    if (!steps) return;
    spin -= steps * 100;
    stepTo(lv - steps, e.clientX - r.left, e.clientY - r.top);
  }, { passive: false });

  const ptrs = new Map();
  let downCard = null, moved = 0, pinch = null;
  view.addEventListener("pointerdown", (e) => {
    view.setPointerCapture(e.pointerId);
    ptrs.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (ptrs.size === 1) {
      const c = e.target.closest(".card");
      downCard = c ? c.dataset.id : null;
      moved = 0;
      view.classList.add("drag");
    } else if (ptrs.size === 2) {
      const [a, b] = [...ptrs.values()];
      pinch = { d: Math.hypot(a.x - b.x, a.y - b.y) };
      downCard = null;
    }
  });
  view.addEventListener("pointermove", (e) => {
    if (ptrs.size || !world.classList.contains("icons") || e.pointerType === "touch") { hideTip(); return; }
    const card = e.target.closest && e.target.closest(".card");
    const id = card && Object.keys(cards).find((k) => cards[k] === card);
    if (id) showTip(id, e.clientX, e.clientY); else hideTip();
  });
  view.addEventListener("pointerleave", hideTip);
  view.addEventListener("pointermove", (e) => {
    const p = ptrs.get(e.pointerId);
    if (!p) return;
    const dx = e.clientX - p.x, dy = e.clientY - p.y;
    p.x = e.clientX; p.y = e.clientY;
    if (ptrs.size === 1) {
      moved += Math.abs(dx) + Math.abs(dy);
      tx += dx; ty += dy;
      apply();
    } else if (ptrs.size === 2 && pinch) {
      const [a, b] = [...ptrs.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      const r = view.getBoundingClientRect();
      if (d / pinch.d > 1.3 || d / pinch.d < 1 / 1.3) {
        stepTo(lv + (d > pinch.d ? 1 : -1), (a.x + b.x) / 2 - r.left, (a.y + b.y) / 2 - r.top);
        pinch.d = d;
      }
      moved = 99;
    }
  });
  function up(e) {
    if (!ptrs.has(e.pointerId)) return;
    ptrs.delete(e.pointerId);
    if (ptrs.size < 2) pinch = null;
    if (ptrs.size === 0) {
      view.classList.remove("drag");
      if (moved < 6 && e.type === "pointerup") {
        if (downCard) select(downCard);
      }
      downCard = null;
    }
  }
  view.addEventListener("pointerup", up);
  view.addEventListener("pointercancel", up);
  world.addEventListener("keydown", (e) => {
    const c = e.target.closest(".card");
    if (c && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); select(c.dataset.id); }
  });

  $("#fitBtn").addEventListener("click", fit);
  const centre = () => { const r = view.getBoundingClientRect(); return [r.width / 2, r.height / 2]; };
  $("#zoomIn").addEventListener("click", () => stepTo(lv + 1, ...centre()));
  $("#zoomOut").addEventListener("click", () => stepTo(lv - 1, ...centre()));

  // ---- the bot run: off until the viewer presses Play
  let timer = null;
  const speedSel = $("#speed"), playBtn = $("#playBtn");
  function stop() {
    clearInterval(timer);
    timer = null;
    playBtn.textContent = "Play";
    playBtn.setAttribute("aria-pressed", "false");
  }
  function play() {
    if (+speedSel.value === 0) speedSel.value = "420";
    if (st.k >= steps.length) setStep(0);
    clearInterval(timer);
    timer = setInterval(() => {
      if (st.k >= steps.length) { stop(); return; }
      setStep(st.k + 1);
    }, +speedSel.value);
    playBtn.textContent = "Pause";
    playBtn.setAttribute("aria-pressed", "true");
  }
  playBtn.addEventListener("click", () => (timer ? stop() : play()));
  speedSel.addEventListener("change", () => {
    if (+speedSel.value === 0) stop();
    else if (timer) play();
  });
  $("#stepBtn").addEventListener("click", () => { stop(); setStep(st.k + 1); });
  $("#resetBtn").addEventListener("click", () => { stop(); setStep(0); });

  // ---- the info modal
  const info = $("#info");
  $("#infoBtn").addEventListener("click", () => info.showModal());
  $("#infoClose").addEventListener("click", () => info.close());
  info.addEventListener("click", (e) => { if (e.target === info) info.close(); });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !info.open && (selected || tab === "r")) { selected = null; tab = "m"; render(); }
  });

  // ---- a world partway through: the bot in the middle of "Go fishing" (cast, one fish of three)
  const sample = steps.findIndex((s) => s.t === "obj" && s.m === "go-fishing" && s.i === 1) + 1;
  // centre a mission at a level (used by the stills; the page never calls it on its own)
  function focus(id, level) {
    if (level !== undefined) { setLevel(Math.max(0, Math.min(LEVELS.length - 1, level))); relayout(); }
    const m = byId[id], r = view.getBoundingClientRect(), q = unit();
    const room = $("#panel").classList.contains("open") && r.width > 640 ? $("#panel").offsetWidth : 0;
    tx = (r.width - room) / 2 - (px(m.col) + CW / 2) * q;
    ty = r.height / 2 - (py(m.row) + CH / 2) * q;
    apply();
  }
  // test hooks for the stills
  window.missionTree = { steps: steps.length, missions: M.length, unlocks: M.reduce((n, m) => n + m.unl.length, 0), setStep, select, fit, focus, sample, levels: LEVELS.length, level: () => lv };
  setStep(sample);
  // Open at full size on the mission the bot is playing; Fit shows the whole tree.
  focus("go-fishing", OPEN);
})();
