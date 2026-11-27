// Mission Tree mockup: the starter-missions tree (openspec/changes/starter-missions/design.md,
// "Missions are a tree that unlocks the tech" and "The full tree") and, on the Town tab, the town
// tree (openspec/changes/town-missions-tree/design.md, decision 2), each drawn as the game's
// Missions screen; on the Survival tab, the starter tree as a Survival world shows it: the game's
// trunk to the fire, then the paths a player picks in the path picker
// (openspec/changes/survival-mode/design.md, decision 18); on the Infrastructure tab, the
// INFRASTRUCTURE tree (openspec/changes/infrastructure-missions/design.md, decision 2), with
// early power (openspec/changes/early-power/design.md, decision 2) as its Power branch. The open
// tree is the page's #anchor (#starter, #town, #survival, #infrastructure; #power, the tab's
// name before version 12, redirects to #infrastructure). Nothing moves until the viewer drags,
// zooms, steps or presses Play.
(function () {
  "use strict";
  const IMG = "mission-tree/items/";
  const UI = "mission-tree/ui/";
  // #power was this tab's anchor before version 12: it keeps working, as #infrastructure
  if (location.hash === "#power") history.replaceState(null, "", "#infrastructure");
  const HASH = location.hash.slice(1);
  const TREES = ["town", "survival", "infrastructure"];
  const TREE = TREES.includes(HASH) ? HASH : "starter";

  // ---- what a mission can unlock: recipes and stations (a code the recipe list shows), or a skill
  // or branch (no recipe). new = a recipe the game does not have yet.
  const S_U = {
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

  const S_BRANCHES = [
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
  const S_M = [
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

  // ---- the town tree (town-missions-tree design 2): a trunk and five branches (Carpentry from `furniture`), a strict tree, every
  // mission paying coins. Its unlocks are recipes and stations the game does not have yet (NEW). Version 12: the Roads branch
  // moved to the infrastructure tree (owner, 2026-11-27: "Maybe move roads over to this section"); Carts hang off
  // "A job from the board", which unlocks the wheel (infrastructure-missions design 3, town-missions-tree design 13).
  const T_U = {
    grinder: { name: "Grinder", icon: "grinder", kind: "station", new: true },
    "crushed-stone": { name: "Crushed stone", icon: "crushed-stone", kind: "recipe", new: true },
    mixer: { name: "Mixer", icon: "mixer", kind: "station", new: true },
    "mixed-gravel": { name: "Gravel (at the mixer)", icon: "gravel", kind: "recipe", new: true },
    "survey-stakes": { name: "Survey stakes", icon: "survey-stakes", kind: "recipe", new: true },
    wheel: { name: "Wheel", icon: "wheel", kind: "recipe", new: true },
    "hand-cart": { name: "Hand cart", icon: "hand-cart", kind: "recipe", new: true },
    "horse-cart": { name: "Horse cart", icon: "horse-cart", kind: "recipe", new: true },
    "banded-chest": { name: "Iron-banded chest", icon: "banded-chest", kind: "recipe", new: true },
    closet: { name: "Closet", icon: "closet", kind: "recipe", new: true },
  };
  const T_BRANCHES = [
    { key: "town", name: "0 · TOWN" },
    { key: "trade", name: "1 · TRADE" },
    { key: "jobs", name: "2 · JOBS" },
    { key: "carts", name: "3 · CARTS" },
    { key: "houses", name: "4 · HOUSES & LOGS" },
    { key: "carpentry", name: "5 · CARPENTRY" },
  ];
  const T_M = [
    // 0. Town: opens the first time you reach a town
    { id: "meet", b: "town", icon: "talk", title: "Meet the townsfolk", needs: [], coins: 5,
      obj: [o("talk to people", 3, "talk to 3 people", true)], unl: [], help: "Walk up to people in town and talk with F." },
    { id: "first-trade", b: "town", icon: "coin", title: "Buy and sell", needs: [["meet"]], coins: 10,
      obj: [o("sell goods", 5, "sell 5 goods to a trader"), o("buy a good", 1, "buy 1 good from a trader")], unl: [],
      help: "Talk to a town's trader and pick TRADE." },
    { id: "first-job", b: "town", icon: "board", title: "A job from the board", needs: [["first-trade"]], coins: 15,
      obj: [o("hand in a job", 1, "take a job from a board, do it and hand it in", true)], unl: ["wheel"],
      help: "Take a job from a town's board, do it and hand it in." },

    // 1. Trade
    { id: "trade-more", b: "trade", icon: "coin", title: "Trade in earnest", needs: [["first-job"]], coins: 20,
      obj: [o("sell goods", 20, "sell 20 goods in any town")], unl: [], help: "Sell what you grow, catch and make." },
    { id: "deliver", b: "trade", icon: "parcel", title: "Carry a parcel", needs: [["trade-more"]], coins: 20,
      obj: [o("deliver a parcel", 1, "deliver 1 parcel to another town")], unl: [], help: "Take a delivery job and carry the parcel to the town it names." },
    { id: "goodwill", b: "trade", icon: "heart", title: "A friend in town", needs: [["deliver"]], coins: 25,
      obj: [o("goodwill step", 1, "reach goodwill step 1 in any town", true)], unl: [], help: "Jobs and fair trade make a town like you." },

    // 2. Jobs
    { id: "three-jobs", b: "jobs", icon: "board", title: "Three jobs", needs: [["first-job"]], coins: 15,
      obj: [o("hand in jobs", 3, "hand in 3 jobs", true)], unl: [], help: "Boards post new jobs every few days." },
    { id: "lost", b: "jobs", icon: "villager", title: "Bring them home", needs: [["three-jobs"]], coins: 30,
      obj: [o("walk a villager home", 1, "find a lost villager and walk them home")], unl: [], help: "Find them, talk with F, and lead them back inside the walls." },
    { id: "mend", b: "jobs", icon: "planks", title: "Mend a building", needs: [["lost"]], coins: 25,
      obj: [o("mend a wall", 1, "mend 1 broken wall with planks")], unl: [], help: "Bring two planks to the gap and press F." },
    { id: "lamps", b: "jobs", icon: "lantern_post", title: "Light the lamps", needs: [["mend"]], coins: 15, pend: true,
      obj: [o("light town lamps", 4, "light 4 town lamps at dusk")], unl: [], help: "Light the town's lamps as the sun goes down." },

    // 3. Carts (land-transport design 9, 10), off the board's first job since version 12
    { id: "wheels", b: "carts", icon: "wheel", title: "Make wheels", needs: [["first-job"]], coins: 10,
      obj: [o("craft wheels", 2, "craft 2 wheels at the workbench", true)], unl: ["hand-cart"], help: "Wheels take planks and an iron tyre." },
    { id: "hand-cart", b: "carts", icon: "hand-cart", title: "A hand cart", needs: [["wheels"]], coins: 15,
      obj: [o("craft a hand cart", 1, "craft a hand cart", true), o("set it down", 1, "place the hand cart")], unl: [],
      help: "Set the cart down, then press F at its handles to pull it." },
    { id: "haul", b: "carts", icon: "hand-cart", title: "Haul a load", needs: [["hand-cart"]], coins: 20,
      obj: [o("haul goods (m)", 100, "move a loaded cart 100 m", true)], unl: [], help: "Put goods in the cart's hold (C) and pull it. Roads are faster." },
    { id: "horse", b: "carts", icon: "horse", title: "Buy a horse", needs: [["haul"]], coins: 25,
      obj: [o("buy a horse", 1, "buy a horse at a stable")], unl: ["horse-cart"], help: "Stables in the larger towns sell horses." },
    { id: "horse-cart", b: "carts", icon: "horse-cart", title: "Hitch up", needs: [["horse"]], coins: 25,
      obj: [o("craft a horse cart", 1, "craft a horse cart", true), o("hitch your horse", 1, "hitch your horse to it", true)], unl: [],
      help: "Lead your horse to the cart's shafts and press F." },
    { id: "road-trip", b: "carts", icon: "horse-cart", title: "On the road", needs: [["horse-cart"]], coins: 50,
      obj: [o("drive on road (m)", 500, "drive the horse cart 500 m on road", true)], unl: [], help: "Off the road a cart is four times slower." },

    // 4. Houses and logs
    { id: "house", b: "houses", icon: "house", title: "A house of your own", needs: [["first-job"]], coins: 40,
      obj: [o("build walls", 6, "build 6 walls", true), o("build a door", 1, "build 1 door", true), o("fit a roof", 1, "fit a roof over them", true)], unl: [],
      help: "Build walls, a door and a roof on the hex grid." },
    { id: "logs", b: "houses", icon: "log", title: "Float logs", needs: [["house"]], coins: 20,
      obj: [o("float logs", 4, "float 4 logs down a river until they stop", true)], unl: [],
      help: "Drop logs in a river; they stop where the water stops." },

    // 5. Carpentry (furniture design 12): the carpenter's bench and the furniture made at it
    { id: "carpenter", b: "carpentry", icon: "carpenter", title: "A carpenter's bench", needs: [["first-job"]], coins: 10,
      obj: [o("carpenter's bench", 1, "make a carpenter's bench at the workbench", true), o("place it", 1, "place the carpenter's bench")], unl: [],
      help: "Make a carpenter's bench at the workbench and set it down." },
    { id: "chair", b: "carpentry", icon: "chair", title: "Make a chair", needs: [["carpenter"]], coins: 10,
      obj: [o("chair", 1, "make a chair at the carpenter's bench", true)], unl: [], help: "Make a chair at the carpenter's bench." },
    { id: "sit", b: "carpentry", icon: "stool", title: "Take a seat", needs: [["chair"]], coins: 5,
      obj: [o("sit", 1, "sit on a chair, stool or bench", true)], unl: [], help: "Press F at a chair, stool or bench to sit." },
    { id: "table", b: "carpentry", icon: "table", title: "Make a table", needs: [["sit"]], coins: 10,
      obj: [o("table", 1, "make a table at the carpenter's bench", true)], unl: [], help: "Make a table at the carpenter's bench." },
    { id: "bed", b: "carpentry", icon: "bed", title: "Make a bed", needs: [["table"]], coins: 15,
      obj: [o("bed", 1, "make a bed from planks and wool", true)], unl: [], help: "A bed takes planks and wool." },
    { id: "lie", b: "carpentry", icon: "bed", title: "Lie in a bed", needs: [["bed"]], coins: 5,
      obj: [o("lie down", 1, "lie down on a bed", true)], unl: [], help: "Press F at a bed to lie down." },
    { id: "chest", b: "carpentry", icon: "chest", title: "Make a chest", needs: [["lie"]], coins: 15,
      obj: [o("chest", 1, "make a chest from planks and rope", true)], unl: ["banded-chest"], help: "A chest takes planks and rope." },
    { id: "furnish", b: "carpentry", icon: "drawers", title: "Furnish a room", needs: [["chest"]], coins: 30,
      obj: [o("furniture", 4, "place 4 pieces of furniture", true)], unl: ["closet"],
      help: "Place four pieces of furniture with the build tools." },
    { id: "find", b: "carpentry", icon: "banded-chest", title: "Look in a town chest", needs: [["furnish"]], coins: 15,
      obj: [o("take", 1, "take something from a chest in a town house", true)], unl: [],
      help: "Open a chest in a town house and take something." },
  ];

  // ---- the Survival tab (survival-mode design 18, version 11): the starter tree as a Survival
  // world shows it. The owner, 2026-11-27: "Survival: consist of the starter tree today then (in the
  // game) then branch off to all survival stuff like eating and drinking, hunting etc.", and "after
  // making a fire present players with the tree and show them options on what branch to take. Then
  // when they click on that topic show them the tree branch, then the tree continues along their
  // chosen path". So the trunk is the game's (missions.ron: gather, pack, axe, wood, planks and
  // rope, the bench, stone tools, the fire, the chest), and every branch after "Make a fire" is a
  // path the player picks. Survival missions are marked mode: survival and never shown in an
  // Explorer world. Unlocks marked NEW are recipes the game does not have yet. Icons are stand-ins
  // (tools/mockup_survival_icons.py).
  const SV_U = Object.assign({}, S_U, {
    straw: { name: "Straw", icon: "straw", kind: "recipe", new: true },
    still: { name: "Still", icon: "still", kind: "station", new: true },
    "meat-stew": { name: "Meat stew", icon: "meat-stew", kind: "recipe", new: true },
    bedroll: { name: "Bedroll", icon: "bedroll", kind: "recipe", new: true },
    "tanning-rack": { name: "Tanning rack", icon: "tanning-rack", kind: "station", new: true },
    bow: { name: "Bow", icon: "bow", kind: "recipe", new: true },
    "stone-arrows": { name: "Stone arrows", icon: "arrows", kind: "recipe", new: true },
    "bone-arrows": { name: "Bone arrows", icon: "arrows", kind: "recipe", new: true },
    "bronze-hunting-axe": { name: "Bronze hunting axe", icon: "hunting-axe", kind: "recipe", new: true },
    "bronze-sword": { name: "Bronze sword", icon: "sword", kind: "recipe", new: true },
    "bronze-spear": { name: "Bronze spear", icon: "bronze-spear", kind: "recipe", new: true },
    "bronze-arrows": { name: "Bronze arrows", icon: "arrows", kind: "recipe", new: true },
    "iron-sword": { name: "Iron sword (with the iron age)", icon: "iron-sword", kind: "recipe", new: true },
    "straw-hat": { name: "Straw hat", icon: "straw-hat", kind: "recipe", new: true },
    "wool-cloak": { name: "Wool cloak", icon: "wool-cloak", kind: "recipe", new: true },
  });
  // The paths after the fire, in the order the picker lays them out. icon and line are the
  // picker's card: the line is game text, short and plain.
  const SV_BRANCHES = [
    { key: "trunk", name: "0 · HANDS TO TOOLS" },
    { key: "eat-drink", name: "1 · EAT AND DRINK", icon: "water-drop", line: "Fresh water, wild food and a hot meal." },
    { key: "hunting", name: "2 · HUNTING", icon: "stone-spear", line: "A spear and a hunt, then a bow and blades." },
    { key: "fishing", name: "3 · FISHING", icon: "rod", line: "A rod, bait and fish." },
    { key: "farming", name: "4 · FARMING", icon: "wheat", line: "Seeds, a field and bread." },
    { key: "shelter", name: "5 · SHELTER AND WEATHER", icon: "moon", line: "The night, the heat and the cold." },
    { key: "water", name: "6 · IN THE WATER", icon: "breath", line: "Swim far and hold your breath." },
    { key: "herding", name: "7 · HERDING", icon: "yard_hen", line: "Fences, a tame animal and eggs." },
    { key: "mining", name: "8 · MINING & SMELTING", icon: "bronze-ingot", line: "Ore, a forge and bronze." },
    { key: "light", name: "9 · LIGHT & BUILDING", icon: "torch", line: "Walls, a roof and torches." },
    { key: "cooking", name: "10 · COOKING", icon: "grilled-fish", line: "Cook what you catch." },
    { key: "about", name: "11 · GETTING ABOUT", icon: "icon-map", line: "The map, a reef, a town and a boat." },
  ];
  const SV_ONLY = [
    // 1. Eat and drink: fresh water only (owner, 2026-11-27: "Keep in mind salt water is undrinkable")
    { id: "drink", b: "eat-drink", icon: "water-drop", title: "Drink water", needs: [["fire"]],
      obj: [o("drink fresh water", 3, "drink 3 sips of fresh water: a river, a lake, rain or a still", true)], unl: ["straw", "still"],
      help: "Crouch at a river or lake and hold use. Sea water is salt." },
    { id: "forage", b: "eat-drink", icon: "fruit", title: "Something to eat", needs: [["drink"]],
      obj: [o("eat raw food", 2, "eat 2 raw foods: fruit or an egg")], unl: [], help: "Pick fruit from a tree, or take an egg from a nest." },
    { id: "cook-meal", b: "eat-drink", icon: "fried-egg", title: "A hot meal", needs: [["forage"]],
      obj: [o("cook a dish", 1, "cook 1 dish at the campfire"), o("eat it", 1, "eat it")], unl: [], help: "Fry an egg or grill fish at the campfire." },
    { id: "straw", b: "eat-drink", icon: "straw", title: "A cactus straw", needs: [["drink"]],
      obj: [o("craft a straw", 1, "craft a straw (in the pack)")], unl: [], help: "Two fibre make a straw." },
    { id: "cactus", b: "eat-drink", icon: "cactus", title: "Cactus juice", needs: [["straw"]],
      obj: [o("sip a cactus", 3, "drink 3 sips from a cactus", true)], unl: ["straw-hat"], help: "Hold the straw to a desert cactus." },
    { id: "still", b: "eat-drink", icon: "still", title: "Water from the sea", needs: [["drink"]],
      obj: [o("set a still by a fire", 1, "set a still beside a lit campfire", true), o("drink from it", 3, "drink 3 sips from the still", true)], unl: [],
      help: "Fill the still at the sea. The fire boils the salt out." },
    // 2. Hunting: the spear needs a log, so it is open from the start of a Survival world
    { id: "spear", b: "hunting", icon: "stone-spear", title: "Make a spear", needs: [["fire"]],
      obj: [o("craft a stone spear", 1, "craft a stone spear (in the pack)")], unl: [], help: "In the pack, the Craft row makes a spear from a log, stone and fibre." },
    { id: "hunt", b: "hunting", icon: "hare", title: "Hunt", needs: [["spear"]],
      obj: [o("kill", 1, "kill a wild hen, hare or goat", true)], unl: [], help: "Hunt with the spear. Thrown, it hits harder." },
    { id: "cook-meat", b: "hunting", icon: "roast-meat", title: "Cook meat", needs: [["hunt"]],
      obj: [o("cook roast meat", 2, "cook 2 roast meat at the campfire")], unl: ["meat-stew"], help: "Put raw meat on the campfire." },
    { id: "dress", b: "hunting", icon: "hide", title: "Dress a kill", needs: [["hunt"]],
      obj: [o("take a hide", 1, "take 1 hide from a kill", true)], unl: ["tanning-rack"], help: "Use a blade on the kill before it is gone." },
    { id: "tan", b: "hunting", icon: "tanning-rack", title: "Tan a hide", needs: [["dress"]],
      obj: [o("make leather", 1, "make 1 leather at the tanning rack")], unl: ["bow", "stone-arrows"], help: "Hang a hide on the tanning rack." },
    { id: "bow", b: "hunting", icon: "bow", title: "Bow and arrows", needs: [["tan"]],
      obj: [o("craft a bow", 1, "craft a bow"), o("craft arrows", 8, "craft 8 arrows")], unl: ["bone-arrows"], help: "Make the bow at the workbench." },
    { id: "long-shot", b: "hunting", icon: "arrows", title: "A long shot", needs: [["bow"]],
      obj: [o("kill from 20 m", 1, "kill an animal with an arrow from 20 m or more", true)], unl: ["bronze-hunting-axe"], help: "Draw fully and aim above it." },
    { id: "hunting-axe", b: "hunting", icon: "hunting-axe", title: "A hunting axe", needs: [["long-shot"]], after: ["smelt"],
      obj: [o("craft it", 1, "craft a bronze hunting axe"), o("dress a kill", 1, "dress a kill with it", true)],
      unl: ["bronze-sword", "bronze-spear", "bronze-arrows"], help: "A kill dressed with it gives its hide and more meat." },
    { id: "sword", b: "hunting", icon: "sword", title: "A sword", needs: [["hunting-axe"]],
      obj: [o("craft a sword", 1, "craft a bronze sword")], unl: ["iron-sword"], help: "Make it at the workbench from bronze." },
    { id: "fend-off", b: "hunting", icon: "fox", title: "Drive off a predator", needs: [["sword"]],
      obj: [o("drive off", 1, "make a predator flee, or kill it", true)], unl: [], help: "Foxes come at night. Strike back." },
    // 5. Shelter and weather: the night, heat and cold, and fire (wildfire)
    { id: "first-night", b: "shelter", icon: "moon", title: "The first night", needs: [["fire"]],
      obj: [o("see the dawn", 1, "be out past 20:00 and alive at 06:00", true)], unl: ["bedroll"], help: "Keep warm, fed and watered until morning." },
    { id: "warm-up", b: "shelter", icon: "snowflake", title: "Warm by the fire", needs: [["first-night"]],
      obj: [o("warm up", 1, "warm from cold to fine by a fire", true)], unl: ["wool-cloak"], help: "Stand close to a campfire." },
    { id: "dress-weather", b: "shelter", icon: "wool-cloak", title: "Dress for the weather", needs: [["warm-up"]],
      obj: [o("wear", 1, "wear a straw hat or a wool cloak", true)], unl: [], help: "Put it on in the pack." },
    { id: "cool-down", b: "shelter", icon: "sun", title: "Out of the heat", needs: [["first-night"]],
      obj: [o("cool down", 1, "cool from hot to fine", true)], unl: [], help: "Stand in the shade or get in the water." },
    { id: "relight", b: "shelter", icon: "torch-out", title: "Relight a torch", needs: [["first-night"]], after: ["torch"],
      obj: [o("relight", 1, "relight a torch the rain put out", true)], unl: [], help: "Touch it to a fire." },
    { id: "put-out", b: "shelter", icon: "water-bucket", title: "Put out a fire", needs: [["relight"]],
      obj: [o("put out", 3, "put out 3 burning cells with water", true)], unl: [], help: "Fires start in the dry season." },
    // 6. In the water
    { id: "long-swim", b: "water", icon: "water-bucket", title: "A long swim", needs: [["fire"]],
      obj: [o("swim 100 m", 100, "swim 100 m without being winded")], unl: [], help: "Swim steady. Sprinting tires you fast." },
    { id: "hold-breath", b: "water", icon: "breath", title: "Hold your breath", needs: [["long-swim"]],
      obj: [o("stay under 15 s", 1, "stay under for 15 s and surface", true)], unl: [], help: "Dive and come up before your breath runs out." },
    { id: "soft-landing", b: "water", icon: "splash", title: "Into the water", needs: [["hold-breath"]],
      obj: [o("fall into water", 1, "fall 10 layers or more into deep water", true)], unl: [], help: "Deep water breaks a fall." },
  ];
  const SV_M = TREE === "survival" ? (() => {
    const copy = S_M.map((m) => Object.assign({}, m));
    const pack = { id: "pack", b: "trunk", icon: "chest", title: "Open your pack", needs: [["gather"]],
      obj: [o("open the pack", 1)], unl: [], help: "Press Tab to open your pack and craft with what you gathered." };
    const storage = { id: "storage", b: "trunk", icon: "chest", title: "A chest", needs: [["fire"]],
      obj: [o("make a chest", 1), o("set it down", 1), o("store a stack", 1)], unl: [],
      help: "At the workbench (F), make a chest from eight planks. Set it down, open it with F and put a stack in it." };
    copy.splice(1, 0, pack);
    copy.find((m) => m.id === "axe").needs = [["pack"]];
    copy.splice(copy.findIndex((m) => m.id === "fire") + 1, 0, storage);
    return copy.concat(SV_ONLY);
  })() : [];

  // ---- the Power branch (early-power design 2 and 3), drawn on the Infrastructure tab since version 12
  // (infrastructure-missions design 2): first drafted as a tree of its own, opened by the starter tree's
  // "The iron age"; now hung off the infrastructure tree's "Build a grinder", with the iron age as a
  // hidden gate across trees (xafter). Windmills grind food, the water mill makes power, lines carry it,
  // and there is no battery: an appliance runs only while the wheel turns. rec = the recipe: station
  // and inputs. Icons are stand-ins (tools/art/early_power_icons.py).
  const r = (at, ins, out) => ({ at, ins, out });
  const P_U = {
    millstone: { name: "Millstone", icon: "millstone", kind: "recipe", new: true, rec: r("workbench", "2 stone blocks, 4 stone", "1 millstone") },
    "copper-wire": { name: "Copper wire", icon: "copper-wire", kind: "recipe", new: true, rec: r("workbench", "1 copper ingot", "4 copper wire") },
    dynamo: { name: "Dynamo", icon: "dynamo", kind: "recipe", new: true, rec: r("workbench", "8 copper wire, 2 iron ingot, 1 axle", "1 dynamo") },
    windmill: { name: "Windmill", icon: "windmill", kind: "station", new: true, rec: r("workbench", "24 planks, 6 log, 4 rope, 4 wooden gear, 1 axle, 2 millstone, 6 fibre", "1 windmill") },
    flour: { name: "Flour", icon: "flour", kind: "recipe", new: true, rec: r("windmill", "3 wheat", "2 flour") },
    loaf: { name: "Loaf", icon: "loaf", kind: "recipe", new: true, rec: r("campfire", "2 flour", "1 loaf") },
    "feed-mash": { name: "Feed mash", icon: "feed-mash", kind: "recipe", new: true, rec: r("windmill", "1 wheat, 1 turnip", "2 feed mash") },
    "water-mill": { name: "Water mill", icon: "water-mill", kind: "station", new: true, rec: r("workbench", "20 planks, 4 log, 2 wooden gear, 1 axle, 1 dynamo, 8 stone blocks", "1 water mill") },
    "power-pole": { name: "Power pole", icon: "power-pole", kind: "recipe", new: true, rec: r("workbench", "1 log, 2 copper wire", "1 power pole") },
    "electric-lamp": { name: "Electric lamp", icon: "electric-lamp", kind: "recipe", new: true, rec: r("workbench", "1 iron ingot, 1 copper wire", "2 electric lamp") },
    motor: { name: "Motor", icon: "motor", kind: "recipe", new: true, rec: r("workbench", "6 copper wire, 2 iron ingot", "1 motor") },
    bellows: { name: "Bellows", icon: "bellows", kind: "recipe", new: true, rec: r("workbench", "2 leather, 3 planks, 1 motor", "1 bellows") },
  };
  // The two recipes early power opened with: since version 12 they open with the infrastructure tree,
  // listed with its first mission.
  const P_OPEN = {
    grinder: { name: "Grinder", icon: "grinder", rec: r("workbench", "10 stone, 6 planks, 2 iron ingot", "1 grinder") },
    "wooden-gear": { name: "Wooden gear", icon: "wooden-gear", new: true, rec: r("workbench", "3 planks", "1 wooden gear") },
    axle: { name: "Axle", icon: "axle", new: true, rec: r("workbench", "1 log, 1 iron ingot", "1 axle") },
  };
  const P_BRANCHES = [
    { key: "mill", name: "4 · POWER" },
    { key: "wind", name: "4 · WIND" },
    { key: "water", name: "4 · WATER" },
    { key: "lines", name: "4 · LINES" },
    { key: "appl", name: "4 · APPLIANCES" },
  ];
  const P_M = [
    // 4. Power, its trunk Millwright: off "Build a grinder", after the starter tree's "The iron age"
    { id: "gears", b: "mill", icon: "wooden-gear", title: "Cut gears", needs: [["grinder"]], xafter: ["The iron age (starter tree)"],
      obj: [o("craft wooden gears", 4, "craft 4 wooden gears at the workbench", true), o("craft an axle", 1, "craft 1 axle at the workbench", true)],
      unl: ["millstone", "copper-wire", "dynamo"], help: "A gear takes planks; an axle takes a log and an iron ingot." },
    { id: "millstones", b: "mill", icon: "millstone", title: "Dress millstones", needs: [["gears"]],
      obj: [o("craft millstones", 2, "craft 2 millstones at the workbench", true)], unl: ["windmill"], help: "Dress stone blocks into millstones at the workbench." },
    // 1. Wind: the windmill grinds food, and turns a grinder on its floor
    { id: "windmill", b: "wind", icon: "windmill", title: "Raise a windmill", needs: [["millstones"]],
      obj: [o("craft a windmill", 1, "craft a windmill", true), o("set it on a rise", 1, "set it on a rise", true)], unl: ["flour"],
      help: "Set the windmill on a rise. It needs the wind." },
    { id: "grind", b: "wind", icon: "flour", title: "Grind grain", needs: [["windmill"]],
      obj: [o("grind wheat", 12, "grind 12 wheat at the windmill", true)], unl: ["loaf"], help: "Put wheat in the windmill (F). No wind, no flour." },
    { id: "loaf", b: "wind", icon: "loaf", title: "Bake a loaf", needs: [["grind"]],
      obj: [o("cook loaves", 2, "cook 2 loaves at the campfire")], unl: ["feed-mash"], help: "Put flour on the campfire." },
    { id: "mash", b: "wind", icon: "feed-mash", title: "Mash for the herd", needs: [["loaf"]],
      obj: [o("grind feed mash", 6, "grind 6 feed mash at the windmill", true), o("feed the herd", 3, "feed mash to the herd 3 times")], unl: [],
      help: "Grind wheat and turnip together. Fed mash, the herd gives sooner." },
    { id: "wind-grinder", b: "wind", icon: "grinder", title: "Stone by the wind", needs: [["windmill"]],
      obj: [o("set a grinder in the mill", 1, "set a grinder on the windmill's floor", true), o("crush stone", 9, "crush 9 stone with it")], unl: [],
      help: "A grinder on the mill floor runs on the wind." },
    // 2. Water: the water mill makes power
    { id: "dynamo", b: "water", icon: "dynamo", title: "Wind a dynamo", needs: [["gears"]],
      obj: [o("craft copper wire", 8, "craft 8 copper wire"), o("craft a dynamo", 1, "craft a dynamo", true)], unl: ["water-mill"],
      help: "Draw copper ingots into wire, then wind it on an axle." },
    { id: "water-mill", b: "water", icon: "water-mill", title: "Build a water mill", needs: [["dynamo"]],
      obj: [o("craft a water mill", 1, "craft a water mill", true), o("set it on moving water", 1, "set it on a flowing river", true)], unl: ["power-pole"],
      help: "Set it on a river that flows. Still water will not turn it." },
    { id: "turn", b: "water", icon: "bolt", title: "Turn the wheel", needs: [["water-mill"]],
      obj: [o("make power (kWh)", 6, "make 6 kWh at the water mill", true)], unl: [], help: "Open the sluice (F). Faster water, more power." },
    // 3. Lines: poles carry it; no battery holds it
    { id: "poles", b: "lines", icon: "power-pole", title: "Set poles", needs: [["turn"]],
      obj: [o("craft poles", 4, "craft 4 power poles", true), o("set poles", 4, "set 4 poles in a run from the mill", true)], unl: ["electric-lamp"],
      help: "Poles join within 4 cells of each other." },
    { id: "line", b: "lines", icon: "copper-wire", title: "String a line", needs: [["poles"]],
      obj: [o("join the mill (cells)", 12, "join the mill to a pole 12 cells away", true)], unl: ["motor", "bellows"],
      help: "Run poles out from the mill. Anything within a cell of a pole is on the line." },
    { id: "lamps", b: "lines", icon: "electric-lamp", title: "Lamps on the line", needs: [["line"]],
      obj: [o("light lamps at night", 3, "light 3 electric lamps from the line at night")], unl: [], help: "Set lamps by a pole. They light while the wheel turns." },
    { id: "sluice", b: "lines", icon: "sluice", title: "When the wheel stops", needs: [["lamps"]],
      obj: [o("shut the sluice at night", 1, "shut the sluice at night", true), o("see the lamps go out", 1, "see the lamps on the line go out")], unl: [],
      help: "Nothing stores power. When the wheel stops, the line stops." },
    // 4. Appliances: stations that run faster on the line
    { id: "motor-grinder", b: "appl", icon: "motor", title: "A grinder on the line", needs: [["line"]],
      obj: [o("fit a motor", 1, "fit a motor to a grinder", true), o("crush stone on the line", 20, "crush 20 stone on the line")], unl: [],
      help: "A grinder with a motor works twice as fast while the wheel turns." },
    { id: "bellows", b: "appl", icon: "bellows", title: "Bellows at the forge", needs: [["motor-grinder"]],
      obj: [o("fit bellows", 1, "fit bellows to a forge", true), o("smelt on the line", 4, "smelt 4 ingots on the line")], unl: [],
      help: "Bellows on the line make the forge faster." },
    { id: "motor-mixer", b: "appl", icon: "mixer", title: "A mixer on the line", needs: [["bellows"]], after: ["mixer"],
      obj: [o("fit a motor", 1, "fit a motor to a mixer", true), o("mix gravel on the line", 16, "mix 16 gravel on the line")], unl: [],
      help: "Too much on one line and everything slows. Build a second mill." },
  ];
  // ---- the Infrastructure tab (infrastructure-missions design 2, version 12): phase 4a's tree, opened by
  // the town tree's "A job from the board". A trunk (Stoneworks) and four branches: Roads, Civil works,
  // Town and Power (early power, above). Coins as the town tree's; Power pays none. Icons for the new
  // goods, stations and blueprints are stand-ins (tools/art/infrastructure_icons.py).
  const I_U = Object.assign({
    "crushed-stone": { name: "Crushed stone", icon: "crushed-stone", kind: "recipe", rec: r("grinder", "2 stone", "3 crushed stone") },
    mixer: { name: "Mixer", icon: "mixer", kind: "station", rec: r("workbench", "8 planks, 2 rope, 1 iron ingot, 1 bucket", "1 mixer") },
    "mixed-gravel": { name: "Gravel (at the mixer)", icon: "gravel", kind: "recipe", rec: r("mixer", "3 crushed stone, 1 sand block", "4 gravel") },
    "survey-stakes": { name: "Survey stakes", icon: "survey-stakes", kind: "recipe", rec: r("hand", "1 planks, 1 fibre", "4 survey stakes") },
    milestone: { name: "Milestone", icon: "signpost", kind: "recipe", new: true, rec: r("mason's bench", "2 dressed stone", "1 milestone") },
    "dressed-stone": { name: "Dressed stone", icon: "dressed-stone", kind: "recipe", new: true, rec: r("mason's bench", "1 stone block", "1 dressed stone") },
    mortar: { name: "Mortar", icon: "mortar", kind: "recipe", new: true, rec: r("mixer", "2 sand, 1 crushed stone, 1 water bucket", "4 mortar") },
    "bp-well": { name: "Well", icon: "well", kind: "blueprint", new: true, rec: r("drafting table", "12 dressed stone, 4 mortar, 2 log, 1 rope, 1 bucket", "1 cell") },
    "bp-retaining-wall": { name: "Retaining wall", icon: "retaining-wall", kind: "blueprint", new: true, rec: r("drafting table", "6 dressed stone, 2 mortar a cell", "1 to 12 cells") },
    "bp-culvert": { name: "Culvert", icon: "culvert", kind: "blueprint", new: true, rec: r("drafting table", "10 dressed stone, 4 mortar, 4 gravel", "1 road cell") },
    "bp-mill-race": { name: "Mill race", icon: "mill-race", kind: "blueprint", new: true, rec: r("drafting table", "4 dressed stone, 1 mortar a cell, 1 sluice board", "3 to 16 cells") },
    "bp-stone-bridge": { name: "Stone bridge", icon: "stone-bridge", kind: "blueprint", new: true, rec: r("drafting table", "14 dressed stone, 6 mortar, 2 gravel a cell", "3 to 10 cells") },
    "bp-ferry": { name: "Ferry", icon: "ferry", kind: "blueprint", new: true, rec: r("drafting table", "8 planks, 4 log, 30 rope, 2 posts", "7 to 20 cells") },
    "bp-market": { name: "Market stall", icon: "market", kind: "blueprint", new: true, rec: r("drafting table", "10 planks, 4 log, 4 wool, 2 dressed stone", "2 cells") },
  }, P_U);
  const I_BRANCHES = [
    { key: "stone", name: "0 · STONEWORKS" },
    { key: "roads", name: "1 · ROADS" },
    { key: "civil", name: "2 · CIVIL WORKS" },
    { key: "town", name: "3 · TOWN" },
  ].concat(P_BRANCHES);
  const I_M = [
    // 0. Stoneworks, the trunk: moved from the town tree's Roads (owner, 2026-11-27); opens with "A job from the board"
    { id: "grinder", b: "stone", icon: "grinder", title: "Build a grinder", needs: [], coins: 10,
      obj: [o("craft a grinder", 1, "craft a grinder at the workbench", true), o("place it", 1, "place the grinder")], unl: ["crushed-stone"],
      help: "Make a grinder at the workbench and set it down." },
    { id: "crush", b: "stone", icon: "crushed-stone", title: "Crush stone", needs: [["grinder"]], coins: 10,
      obj: [o("make crushed stone", 9, "make 9 crushed stone at the grinder", true)], unl: ["mixer"],
      help: "Put stone in the grinder (F)." },
    { id: "mixer", b: "stone", icon: "mixer", title: "Build a mixer", needs: [["crush"]], coins: 10,
      obj: [o("craft a mixer", 1, "craft a mixer at the workbench", true), o("place it", 1, "place the mixer")], unl: ["mixed-gravel"],
      help: "Make a mixer at the workbench and set it down." },
    { id: "gravel", b: "stone", icon: "gravel", title: "Mix gravel", needs: [["mixer"]], coins: 10,
      obj: [o("make gravel", 8, "make 8 gravel at the mixer from crushed stone and sand", true)], unl: ["survey-stakes"],
      help: "Put crushed stone and sand in the mixer (F)." },
    // 1. Roads: moved from the town tree, and a road between two towns
    { id: "mark", b: "roads", icon: "survey-stakes", title: "Mark a road", needs: [["gravel"]], coins: 10,
      obj: [o("drive stakes", 2, "drive 2 survey stakes", true), o("mark cells", 8, "mark 8 cells for paving", true)], unl: [],
      help: "Hold survey stakes and drive one at each turn of your road." },
    { id: "pave", b: "roads", icon: "stone-shovel", title: "Pave it", needs: [["mark"]], coins: 30,
      obj: [o("pave cells", 8, "pave 8 marked cells with gravel", true)], unl: [],
      help: "Hold the shovel, carry gravel, and press F on a marked cell." },
    { id: "join", b: "roads", icon: "lantern_post", title: "Join a town", needs: [["pave"]], coins: 60,
      obj: [o("pave to a town or road", 1, "pave a road that joins a town or a road of the world", true)], unl: [],
      help: "Mark and pave until your road meets a town or a road." },

    { id: "two-towns", b: "roads", icon: "signpost", title: "Town to town", needs: [["join"]], coins: 80,
      obj: [o("join two towns", 1, "pave a road that joins two towns", true)], unl: ["milestone"],
      help: "Pave a road from one town to another." },

    // 2. Civil works: blueprints of the planner's civil kit, set out and built piece by piece
    { id: "mason", b: "civil", icon: "mason-bench", title: "A mason's bench", needs: [["gravel"]], coins: 10,
      obj: [o("mason's bench", 1, "craft a mason's bench at the workbench", true), o("place it", 1, "place the mason's bench")],
      unl: ["dressed-stone", "mortar", "bp-well"], help: "Make a mason's bench at the workbench and set it down." },
    { id: "well", b: "civil", icon: "well", title: "Dig a well", needs: [["mason"]], coins: 20,
      obj: [o("set out a well", 1, "set out a well from the drafting table", true), o("finish the well", 1, "build every piece of the well", true)],
      unl: ["bp-retaining-wall"], help: "Set out a well at the drafting table and build it." },
    { id: "retaining", b: "civil", icon: "retaining-wall", title: "Hold the bank", needs: [["well"]], coins: 25,
      obj: [o("wall (cells)", 4, "finish a retaining wall 4 cells long", true)], unl: ["bp-culvert", "bp-mill-race"],
      help: "A retaining wall keeps a cut bank from falling in." },
    { id: "culvert", b: "civil", icon: "culvert", title: "Under the road", needs: [["retaining"]], coins: 25,
      obj: [o("finish a culvert", 1, "finish a culvert where a road crosses a stream", true)], unl: ["bp-stone-bridge"],
      help: "Set a culvert where your road meets a stream." },
    { id: "stone-bridge", b: "civil", icon: "stone-bridge", title: "A stone bridge", needs: [["culvert"]], coins: 60,
      obj: [o("build the bridge", 1, "finish a stone bridge of 3 cells or more over a river", true)], unl: ["bp-ferry"],
      help: "A stone bridge spans up to 10 cells." },
    { id: "ferry", b: "civil", icon: "ferry", title: "A ferry", needs: [["stone-bridge"]], coins: 30,
      obj: [o("rig a ferry", 1, "rig a ferry over a river too wide to bridge", true), o("cross on it", 1, "cross the river on the ferry", true)], unl: [],
      help: "Too wide to bridge? Rig a ferry and pull it across with F." },
    { id: "race", b: "civil", icon: "mill-race", title: "Cut a mill race", needs: [["retaining"]], after: ["water-mill"], coins: 40,
      obj: [o("finish the race", 1, "finish a mill race to a water mill", true), o("run the mill full", 1, "run the water mill at full output", true)], unl: [],
      help: "A race brings the river to the wheel faster." },

    // 3. Town: a town grown on the grid
    { id: "plot", b: "town", icon: "plot", title: "Lay a plot", needs: [["grinder"]], coins: 15,
      obj: [o("set out a site", 1, "set out a site from a blueprint beside a town", true)], unl: ["bp-market"],
      help: "Pick a blueprint at the drafting table and set it out by a town." },
    { id: "raise", b: "town", icon: "house", title: "Raise a building", needs: [["plot"]], coins: 30,
      obj: [o("finish a site", 1, "build every piece of the site", true)], unl: [],
      help: "Bring the pieces and build the site with the mallet." },
    { id: "street", b: "town", icon: "gravel", title: "A street to the door", needs: [["raise"]], coins: 20,
      obj: [o("pave (cells)", 6, "pave 6 cells from its door to a town street", true)], unl: [],
      help: "Mark and pave from the door to the street." },
    { id: "ask", b: "town", icon: "market", title: "What the town needs", needs: [["street"]], coins: 50,
      obj: [o("what it asks", 1, "raise the building a town's board asks for", true)], unl: [],
      help: "Towns post the buildings they need on the board." },
    { id: "grow", b: "town", icon: "town-tier", title: "The town grows", needs: [["ask"]], coins: 80,
      obj: [o("a town grows", 1, "a town you built for reaches its next size", true)], unl: [],
      help: "Build what a town needs and it grows." },

  ].concat(P_M);
  // the moved missions are in the game: their objectives are counted today, so none is marked NEW
  ["grinder", "crush", "mixer", "gravel", "mark", "pave", "join"].forEach((id) => I_M.find((m) => m.id === id).obj.forEach((ob) => (ob.nw = false)));
  const INF = TREE === "infrastructure";
  const U = TREE === "town" ? T_U : TREE === "survival" ? SV_U : INF ? I_U : S_U;
  const BRANCHES = TREE === "town" ? T_BRANCHES : TREE === "survival" ? SV_BRANCHES : INF ? I_BRANCHES : S_BRANCHES;
  const M = TREE === "town" ? T_M : TREE === "survival" ? SV_M : INF ? I_M : S_M;

  const byId = Object.fromEntries(M.map((m) => [m.id, m]));
  const byIdRaw = byId;
  let order = BRANCHES.flatMap((b) => M.filter((m) => m.b === b.key).map((m) => m.id));
  const unlockedBy = {};
  M.forEach((m) => m.unl.forEach((u) => (unlockedBy[u] = m.id)));
  const isOpen = (m, done) => m.needs.every((g) => g.some((id) => done.has(id))) && (m.after || []).every((id) => done.has(id));
  const unlName = (u) => U[u].name.replace(/ \(.*\)$/, "");

  // ---- the bot's steps: Now, each objective filling (a count of 3 or more fills in two steps), Done.
  const steps = [];
  const playMission = (id, nowDone) => {
    const m = byId[id];
    if (!nowDone) steps.push({ t: "now", m: id });
    m.obj.forEach((ob, i) => {
      if (ob.n >= 3) steps.push({ t: "obj", m: id, i, v: Math.floor(ob.n / 2) });
      steps.push({ t: "obj", m: id, i, v: ob.n });
    });
    steps.push({ t: "done", m: id });
  };

  // ---- the Survival tab's paths (survival-mode design 18, version 11). When "Make a fire" is done
  // the picker shows every path that has a mission open; the one picked is the current path, and
  // its first open mission (file order) is Now. The other paths stay open and count. When the path
  // has nothing open (done, or its next mission waits on another branch) the picker shows again.
  const PATHS = TREE === "survival" ? SV_BRANCHES.slice(1).map((b) => b.key) : [];
  const pathBranch = (key) => BRANCHES.find((b) => b.key === key);
  const pathMissions = (key) => M.filter((m) => m.b === key && !m.pend);
  const pathGoal = (key, done) => pathMissions(key).find((m) => !done.has(m.id) && isOpen(m, done));
  function offers(done) {
    const out = [];
    PATHS.forEach((key) => {
      const ms = pathMissions(key);
      const left = ms.filter((m) => !done.has(m.id));
      if (!left.length) return;
      const next = left.find((m) => isOpen(m, done));
      const n = ms.length - left.length;
      if (next) { out.push({ key, next: next.id, n, of: ms.length }); return; }
      if (!n) return; // not reached yet
      const w = left.find((m) => m.needs.every((g) => g.some((id) => done.has(id))));
      out.push({ key, wait: w ? (w.after || []).filter((id) => !done.has(id)).map((id) => byId[id].title).join(", ") : "", n, of: ms.length });
    });
    return out;
  }
  // The bot's own order of paths; a viewer's click at a pick replaces the bot's choice there.
  const BOT_PLAN = ["eat-drink", "hunting", "shelter", "water", "fishing", "cooking", "farming", "herding", "mining", "light", "about"];
  const forced = {};
  function buildSteps() {
    steps.length = 0;
    if (TREE !== "survival" && !INF) {
      order.forEach((id) => { if (!byId[id].pend) playMission(id); });
      return;
    }
    if (INF) {
      // branch by branch, but a mission waits for its hidden gates (the mill race for the water mill)
      const done = new Set(), left = order.filter((id) => !byId[id].pend);
      while (left.length) {
        const i = left.findIndex((id) => isOpen(byId[id], done));
        if (i < 0) break;
        const id = left.splice(i, 1)[0];
        playMission(id);
        done.add(id);
      }
      return;
    }
    const done = new Set();
    let picked = false; // the path step has already made its first mission Now
    const finish = (id) => { playMission(id, picked); picked = false; done.add(id); };
    // the trunk, as the game has it, to the fire; the water bar warns on the way (no mission)
    const trunk = M.filter((m) => m.b === "trunk" && m.id !== "storage").map((m) => m.id);
    trunk.forEach((id) => {
      finish(id);
      if (id === "wood") {
        steps.push({ t: "note", warn: "Thirsty", log: "Water at half: Thirsty" });
        steps.push({ t: "note", warn: "Salt water", log: "Tried the sea: Salt water, nothing taken" });
        steps.push({ t: "note", log: "Drank at a river: water full" });
      }
    });
    let picks = 0, path = null, last = null;
    for (let guard = 0; guard < 400; guard++) {
      const g = path && pathGoal(path, done);
      if (g) { finish(g.id); continue; }
      const off = offers(done);
      const live = off.filter((x) => x.next);
      if (live.length) {
        const want = forced[picks];
        const pick = live.find((x) => x.key === want) || BOT_PLAN.map((k) => live.find((x) => x.key === k)).find(Boolean) || live[0];
        steps.push({ t: "pick", n: picks, choose: pick.key, after: last });
        steps.push({ t: "path", n: picks, b: pick.key, m: pathGoal(pick.key, done).id });
        picked = true;
        picks++;
        last = path = pick.key;
        continue;
      }
      // no path has anything open: the trunk's tail, then whatever is left, in file order
      const rest = M.find((m) => !m.pend && !done.has(m.id) && isOpen(m, done));
      if (!rest) break;
      path = null;
      finish(rest.id);
    }
  }
  buildSteps();

  function stateAt(k) {
    const done = new Set();
    const prog = {};
    let cur = null;
    const log = [];
    let fresh = [];
    let lit = null;
    let coins = 0;
    let path = null, picker = null, warn = null;
    for (let i = 0; i < k; i++) {
      const s = steps[i];
      const m = byId[s.m];
      fresh = [];
      lit = null;
      picker = null;
      warn = null;
      if (s.t === "note") {
        warn = s.warn || null;
        log.push({ text: s.log, w: !!s.warn });
      } else if (s.t === "pick") {
        path = null;
        picker = { n: s.n, sel: s.choose, after: s.after, offers: offers(done) };
        const waits = s.after && picker.offers.some((x) => x.key === s.after && !x.next);
        log.push({ text: s.after ? (waits ? "PATH WAITS: " : "PATH DONE: ") + pathBranch(s.after).name.replace(/^\d+ · /, "") + " · choose a path" : "THE FIRE IS LIT · choose a path" });
      } else if (s.t === "path") {
        path = s.b;
        cur = s.m;
        prog[s.m] = m.obj.map(() => 0);
        log.push({ text: "PATH: " + pathBranch(s.b).name.replace(/^\d+ · /, "") + " · NOW: " + m.title });
      } else if (s.t === "now") {
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
        coins += m.coins || 0;
        log.push({ text: "DONE: " + m.title + (m.coins ? " · +" + m.coins + " coins" : "") + u, d: true });
        fresh = M.filter((x) => !done.has(x.id) && !before.has(x.id) && isOpen(x, done)).map((x) => x.id);
        if (fresh.length) log.push({ text: "OPEN: " + fresh.map((id) => byId[id].title).join(", ") });
      }
    }
    return { k, coins, done, prog, cur, log, fresh: k > 0 && steps[k - 1].t === "done" ? fresh : [], lit, path, picker, warn };
  }

  function standing(id, st) {
    if (st.done.has(id)) return "done";
    if (byId[id].pend) return "pending";
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
    return { el, first, key: b.key };
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
  const needText = (m) => "after " + [...m.needs.flat(), ...(m.after || [])].map((id) => byId[id].title).concat(m.xafter || []).join(" + ");
  const branchName = (m) => BRANCHES.find((b) => b.key === m.b).name.replace(/^\d+ · /, "");
  const WORD = { done: "DONE", now: "NOW", open: "OPEN", locked: "LOCKED", pending: "NOT YET" };
  const coinChip = (m) => m.coins ? `<span class="coins" title="${m.coins} coins">${icon("coin", "ci")}${m.coins}</span>` : "";

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
      `<div class="un${st.lit === m.id ? " litrow" : ""}"><span class="lbl">UNLOCKS</span>${unlocks}${coinChip(m)}</div>`;
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
    let last = st.log[st.log.length - 1];
    // the Survival tab: the HUD line names what was done, not the list of what opened
    if (TREE === "survival" && last && /^OPEN: /.test(last.text) && st.log.length > 1) last = st.log[st.log.length - 2];
    // the Survival tab: the chosen path over the mission line, a need's warning, the picker's ask
    const pathLine = st.path ? `<small>${esc(pathBranch(st.path).name.replace(/^\d+ · /, ""))}</small>` : "";
    if (st.warn) {
      hud.textContent = st.warn;
      hud.className = "hud warn";
    } else if (st.picker) {
      hud.textContent = "Choose a path";
      hud.className = "hud";
    } else if (st.cur) {
      const m = byId[st.cur];
      const c = st.prog[m.id];
      const i = m.obj.findIndex((ob, j) => c[j] < ob.n);
      const ob = m.obj[i < 0 ? m.obj.length - 1 : i];
      hud.innerHTML = pathLine + esc(`${ob.s} ${c[i < 0 ? m.obj.length - 1 : i]}/${ob.n}`);
      hud.className = "hud";
    } else if (last && last.d) {
      hud.textContent = last.text.split(" · ")[0];
      hud.className = "hud done";
    } else if (last) {
      hud.textContent = last.text.split(" · ")[0];
      hud.className = "hud";
    } else hud.textContent = "";
    const lines = st.log.slice(-5);
    $("#log").innerHTML = lines.map((l) => `<div class="${l.d ? "d" : l.w ? "w" : ""}">${esc(l.text)}</div>`).join("");
    $("#stepNo").textContent = `step ${st.k}/${steps.length}` + (TREE === "town" || INF ? ` · ${st.coins} coins` : "");
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
      (s === "pending" ? `<p class="help" style="margin-top:8px">Not in this build: the game does not count this yet.</p>` : "") +
      `<h3>OBJECTIVES</h3><ul class="pobj">${m.obj
        .map((ob, i) => `<li>${check(c[i] >= ob.n)}<span>${esc(ob.t)}${ob.nw ? '<span class="nw">NEW</span>' : ""}</span><em>${c[i]}/${ob.n}</em><span class="pb"><i style="width:${Math.round((100 * c[i]) / ob.n)}%"></i></span></li>`)
        .join("")}</ul>` +
      `<h3>HELP</h3><p class="help">${esc(m.help)}</p>` +
      (INF && !m.needs.length ? `<h3>OPENS WITH THE TREE</h3><div class="ulist">${Object.values(P_OPEN).map((d) => `<div><span class="slot sm on">${icon(d.icon)}</span><span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}${recLine(d)}</span></div>`).join("")}</div>` : "") +
      (m.coins ? `<h3>REWARD</h3><div class="ulist"><div><span class="slot sm ${s === "done" ? "on" : "off"}">${icon("coin")}</span><span>${m.coins} coins<small>${s === "done" ? "paid" : "paid when this is done"}</small></span></div></div>` : "") +
      `<h3>UNLOCKS</h3>` +
      (m.unl.length
        ? `<div class="ulist">${m.unl
            .map((u) => {
              const d = U[u];
              const lit = s === "done";
              return `<div>${d.icon ? `<span class="slot sm ${lit ? "on" : "off"}">${icon(d.icon, lit ? "" : "sil")}</span>` : `<span class="slot sm ${lit ? "on" : "off"}"></span>`}<span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}<small>${lit ? "unlocked" : "locked until this is done"}</small>${recLine(d)}</span></div>`;
            })
            .join("")}</div>`
        : `<p class="help">Nothing: this one is for the play.</p>`) +
      (m.needs.length ? `<h3>NEEDS</h3><div class="mlinks">${m.needs.map((g) => g.map((id) => `<button data-go="${id}">${esc(byId[id].title)}</button>`).join(" or ")).join(" + ")}${(m.after || []).map((id) => ` + <button data-go="${id}">${esc(byId[id].title)}</button>`).join("")}${(m.xafter || []).map((t) => ` + <span class="xneed">${esc(t)}</span>`).join("")}</div>` : "") +
      (users.length ? `<h3>FEEDS</h3><div class="mlinks">${users.map((x) => `<button data-go="${x.id}">${esc(x.title)}</button>`).join("")}</div>` : "") +
      (m.uses ? `<h3>USES</h3><div class="mlinks">${m.uses.map((id) => `<button data-go="${id}">${esc(byId[id].title)}</button>`).join("")}</div>` : "") +
      (next.length ? `<h3>OPENS NEXT</h3><div class="mlinks">${next.map((x) => `<button data-go="${x.id}">${esc(x.title)}</button>`).join("")}</div>` : "");
  }

  // A recipe's station and inputs, where the tree gives them (the Power tab).
  const recLine = (d) => d.rec ? `<small class="rec">${esc(d.rec.at)}: ${esc(d.rec.ins)} → ${esc(d.rec.out)}</small>` : "";
  function recipesHtml() {
    const opens = INF ? `<div class="rgroup">WHEN THE TREE OPENS</div>` + Object.values(P_OPEN)
      .map((d) => `<div class="rrow"><span class="slot sm on">${icon(d.icon)}</span><span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}<small>with A job from the board</small>${recLine(d)}</span></div>`).join("") : "";
    const groups = [["recipe", "RECIPES"], ["station", "STATIONS"], ["blueprint", "BLUEPRINTS"], ["skill", "SKILLS"], ["branch", "BRANCHES"]];
    return opens + groups
      .map(([k, label]) => {
        const rows = Object.keys(U).filter((u) => U[u].kind === k && unlockedBy[u]);
        if (!rows.length) return "";
        return `<div class="rgroup">${label}</div>` + rows
          .map((u) => {
            const d = U[u];
            const by = byId[unlockedBy[u]];
            const lit = st.done.has(by.id);
            return `<button class="rrow ${lit ? "" : "locked"}" data-go="${by.id}"><span class="slot sm ${lit ? "on" : "off"}">${d.icon ? icon(d.icon, lit ? "" : "sil") : ""}</span><span>${esc(d.name)}${d.new ? '<span class="nw">NEW</span>' : ""}<small>${lit ? "unlocked" : esc(by.title)}</small>${recLine(d)}</span></button>`;
          })
          .join("");
      })
      .join("");
  }

  // ---- the path picker (Survival tab): the stone frame and oak trim of the side panel, a card a
  // path. The bot's choice is marked; a click picks that path instead.
  let pickerHidden = false;
  function renderPicker() {
    const el = $("#picker");
    if (!el) return;
    const p = TREE === "survival" && st.picker && !pickerHidden ? st.picker : null;
    el.hidden = !p;
    lanes.forEach((l) => l.el.classList.toggle("chosen", !!st.path && l.key === st.path));
    if (!p) return;
    const said = (key) => { const t = pathBranch(key).name.replace(/^\d+ · /, "").toLowerCase(); return t[0].toUpperCase() + t.slice(1); };
    const waits = p.after && p.offers.find((x) => x.key === p.after && !x.next);
    $("#pkSub").textContent = !p.after ? "The fire is lit." : waits ? said(p.after) + " waits on " + waits.wait + "." : said(p.after) + " is done.";
    $("#pkGrid").innerHTML = p.offers.map((x) => {
      const b = pathBranch(x.key);
      const foot = x.next ? (x.n ? "Next: " : "First: ") + byId[x.next].title : "Waits on " + x.wait;
      return `<button class="pcard${x.key === p.sel ? " bot" : ""}${x.next ? "" : " wait"}" data-path="${x.key}"${x.next ? "" : " disabled"}>` +
        `<span class="slot">${icon(b.icon, x.next ? "" : "sil")}</span>` +
        `<span class="pct"><span class="pth"><b>${esc(b.name.replace(/^\d+ · /, ""))}</b><em>${x.n}/${x.of}</em></span><span>${esc(b.line)}</span><small>${esc(foot)}</small></span>` +
        `</button>`;
    }).join("");
  }

  function render() {
    M.forEach(renderCard);
    renderEdges();
    renderHud();
    renderPanel();
    renderPicker();
  }

  function setStep(k, follow) {
    st = stateAt(Math.max(0, Math.min(steps.length, k)));
    pickerHidden = false;
    render();
    // stepping into a picked path pans the tree to it (never on its own: only Step, Play or a click)
    if (follow && st.k > 0 && steps[st.k - 1].t === "path") focusPath(st.path);
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
      setStep(st.k + 1, true);
    }, +speedSel.value);
    playBtn.textContent = "Pause";
    playBtn.setAttribute("aria-pressed", "true");
  }
  playBtn.addEventListener("click", () => (timer ? stop() : play()));
  speedSel.addEventListener("change", () => {
    if (+speedSel.value === 0) stop();
    else if (timer) play();
  });
  $("#stepBtn").addEventListener("click", () => { stop(); setStep(st.k + 1, true); });
  $("#resetBtn").addEventListener("click", () => { stop(); setStep(0); });

  // ---- the info modal
  const info = $("#info");
  $("#infoBtn").addEventListener("click", () => info.showModal());
  $("#infoClose").addEventListener("click", () => info.close());
  info.addEventListener("click", (e) => { if (e.target === info) info.close(); });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !info.open && (selected || tab === "r")) { selected = null; tab = "m"; render(); }
  });

  // ---- a world partway through: the bot in the middle of "Go fishing" (cast, one fish of three), or on
  // the Town tab in the middle of "Mark a road" (both stakes driven, four cells of eight marked)
  const SAMPLE = TREE === "town" ? { m: "haul", i: 0 } : TREE === "survival" ? { m: "fire", pick: true } : INF ? { m: "retaining", i: 0 } : { m: "go-fishing", i: 1 };
  // the Survival tab opens on the picker, the moment the fire is lit
  const sample = SAMPLE.pick ? steps.findIndex((s) => s.t === "pick") + 1 : steps.findIndex((s) => s.t === "obj" && s.m === SAMPLE.m && s.i === SAMPLE.i) + 1;
  // centre a mission at a level (used by the stills; the page never calls it on its own)
  function focus(id, level) {
    if (level !== undefined) { setLevel(Math.max(0, Math.min(LEVELS.length - 1, level))); relayout(); }
    const m = byId[id], r = view.getBoundingClientRect(), q = unit();
    const room = $("#panel").classList.contains("open") && r.width > 640 ? $("#panel").offsetWidth : 0;
    tx = (r.width - room) / 2 - (px(m.col) + CW / 2) * q;
    ty = r.height / 2 - (py(m.row) + CH / 2) * q;
    apply();
  }
  // Pan (eased, about half a second) to a picked path: its next mission at a level where the branch
  // reads, with its lane lit. Only ever called from a click, Step or Play.
  function focusPath(key) {
    if (!key) return;
    const g = pathGoal(key, st.done) || pathMissions(key)[0];
    world.classList.add("pan");
    // the whole branch in view at the largest level that holds it (full cards from 0.8 up), else
    // its next mission at 0.8
    const ms = pathMissions(key);
    const c0 = Math.min(...ms.map((m) => m.col)), c1 = Math.max(...ms.map((m) => m.col));
    const r0 = Math.min(...ms.map((m) => m.row)), r1 = Math.max(...ms.map((m) => m.row));
    const r = view.getBoundingClientRect();
    let n = 6;
    for (let k = LEVELS.length - 1; k >= 6; k--) {
      const q = SCALES[k] * Math.max(1, Math.round(2 * dpr())) / (2 * dpr());
      const w = (c1 - c0 + 1) * (CARD.cw + CARD.gx) * q, h = ((r1 - r0 + 1) * (CARD.ch + CARD.gy) + 60) * q;
      if (w <= r.width - 40 && h <= r.height - 150) { n = k; break; }
    }
    setLevel(n); relayout();
    const q = unit();
    const bw = (c1 - c0 + 1) * (CW + GX) - GX, bh = (r1 - r0 + 1) * (CH + GY) - GY;
    if (bw * q <= r.width - 40 && bh * q <= r.height - 150) {
      tx = r.width / 2 - (px(c0) + bw / 2) * q;
      ty = (r.height - 130) / 2 + 10 - (py(r0) - 46 + (bh + 46) / 2) * q;
      apply();
    } else {
      // too big for one view: its first column at the left and its lane at the top, so the
      // branch runs on to the right and down
      tx = 60 - px(c0) * q;
      tx = Math.min(tx, r.width - 60 - (px(g.col) + CW) * q);
      ty = 24 - (py(r0) - 46) * q;
      ty = Math.min(ty, r.height - 150 - (py(g.row) + CH) * q);
      apply();
    }
    clearTimeout(focusPath.t);
    focusPath.t = setTimeout(() => world.classList.remove("pan"), 520);
  }
  if (TREE === "survival") {
    $("#pathsBtn").hidden = false;
    $("#pkGrid").addEventListener("click", (e) => {
      const b = e.target.closest("[data-path]");
      if (!b || !st.picker || b.disabled) return;
      stop();
      const n = st.picker.n, k = st.k;
      if (b.dataset.path !== st.picker.sel) { forced[n] = b.dataset.path; buildSteps(); }
      setStep(k + 1, true);
    });
    $("#pkLater").addEventListener("click", () => { pickerHidden = true; render(); });
    // the Paths button opens the picker for review: the next time the bot is shown it, or the last
    $("#pathsBtn").addEventListener("click", () => {
      stop();
      if (st.picker) { pickerHidden = false; render(); return; }
      let i = steps.findIndex((s, j) => j >= st.k && s.t === "pick");
      if (i < 0) i = steps.map((s) => s.t).lastIndexOf("pick");
      setStep(i + 1);
      fit();
    });
  }
  // test hooks for the stills
  const pickAt = (n) => steps.findIndex((s) => s.t === "pick" && s.n === n) + 1;
  const stepOf = (t, m) => steps.findIndex((s) => s.t === t && s.m === m) + 1;
  window.missionTree = { tree: TREE, get steps() { return steps.length; }, missions: M.length, unlocks: M.reduce((n, m) => n + m.unl.length, 0), setStep, select, fit, focus, focusPath, pickAt, stepOf,
    pick: (key) => { const b = document.querySelector(`#pkGrid [data-path="${key}"]`); if (b) b.click(); return !!b; },
    state: () => ({ k: st.k, path: st.path, cur: st.cur, picker: st.picker && st.picker.offers.map((x) => x.key + (x.next ? "" : "(waits)")) }),
    stepList: () => steps.map((s) => s.t + ":" + (s.m || s.b || s.choose || s.log || "")), sample, levels: LEVELS.length, level: () => lv };
  setStep(sample);
  // Open at full size on the mission the bot is playing; Fit shows the whole tree.
  focus(SAMPLE.m, OPEN);

  // ---- the two trees, one at a time: the open one is the page's #anchor
  document.querySelectorAll("[data-tree]").forEach((b) => {
    b.setAttribute("aria-selected", String(b.dataset.tree === TREE));
  });
  // #crew is the CREW tab (crew.js), drawn over this tree: it keeps its anchor.
  const OVER = ["crew", "mills"]; // tabs drawn over this tree (crew.js, mills.js) keep their anchor
  if (location.hash.slice(1) !== TREE && !OVER.includes(location.hash.slice(1))) history.replaceState(null, "", "#" + TREE);
  window.addEventListener("hashchange", () => {
    if (OVER.includes(location.hash.slice(1))) return;
    const h = location.hash.slice(1);
    if (h === "power") { history.replaceState(null, "", "#infrastructure"); if (INF) return; }
    const want = h === "power" ? "infrastructure" : TREES.includes(h) ? h : "starter";
    if (want !== TREE) location.reload();
  });
})();
