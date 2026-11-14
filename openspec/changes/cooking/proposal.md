# Proposal: hearth and kitchen (cooking)

## Why

The owner, 2026-11-14: **"Mock up food recipes, bakery (more farming?) stir
fry, pizza, noodles, pasta, different kinds of meat, and fish, calamari and
shrimp (more sea life?)".**

Today the game has no food:
- **The only food-like item is a caught fish.** `Item::Fish(u16)` is the
  species' index in the body's append-only roster (`pbd_core::inventory`,
  `pbd_core::fauna`), stacks to 16 and does nothing when held or used.
  `Item` has three kinds: `Block`, `Tool`, `Fish`.
- **There is no hunger, health, breath or cold.** Swimming has unlimited
  submersion (`swimming`, non-goal "Breath, oxygen, drowning"), and the HUD
  deliberately shows "no health, hunger or any status bar for a mechanic that
  does not exist" (`hotbar-and-game-hud`).
- **What the game does have**, and food could touch: walking and sprinting
  (8 and 14 m/s, half in water), digging with break times (`dig-and-place`),
  fishing with line tension and the fish's stamina (`pbd_core::fishing`), and
  lamps that light what is near them (`lighting-model`).
- **Four mockups feed a kitchen, none built.** M8 "Furrows and Greenhouses"
  (`farming.html`) grows wheat, rice, chili, turnip, sun melon, frost kale and
  glowcap. M7 "Herds, Hunters and Glowmoths" (`animals.html`) gives meat,
  milk and eggs. M25 "Seas, Rivers and Marshes" (`ocean.html`,
  `ocean-environments`) gives squid, shrimp, octopus, clams, lobster, crab,
  crayfish and kelp. M3 "Forge and Fabricator" (`crafting.html`) sets how a
  station is built, placed and used, with recipes, a queue and the pack.
- **Husbandry is being drafted alongside** (`animal-husbandry`): taming,
  pens, breeding, a domestic roster and a butcher's block. This change names
  meat cuts and products only and designs no animal.

## What Changes

One mockup, `docs/mockups/cooking.html`, "Hearth and Kitchen", its art
generator `tools/art/cooking_art.py`, and this write-up. No game code: that
waits for the owner's approval and a request (CLAUDE.md, "Write it up before
touching code").

The page has twelve tabs in four rows, one 3D view live at a time:
- **Kitchens** (3D, hex cells of 2.833 m and 1 m layers, by day, dusk and lit
  at night only by their own fires, ovens, lanterns, candles and windows): a
  farmstead's kitchen yard (campfire and spit, drying rack, quern, barrels,
  smoker, pizza oven), a town bakery (brick bake oven, kneading and pasta
  table, proving shelves, quern, churn and press, counter), an inn kitchen
  (hearth and pot, iron stove and wok, fryer, pasta and noodle table, oil
  press, icebox, larder) and a ship's galley. Interiors are cut away. Click a
  station to see what it makes and cook at it.
- **Cooking**: M3's station panel with heat, freshness and quality, the pack
  and a HUD that shows what a meal does (and the hunger bar it does not
  have, for comparison); the recipe book (every recipe with its icon,
  ingredients, station, time, heat, effect and keep); field to plate, a chain
  diagram from what grows, grazes and swims through processing to the dish.
- **Sources**: twelve new crops in M8's four-stage layout with their climates
  and seasons; meat, milk and eggs by animal and cut, raw, cooked, smoked and
  cured; fish and seafood by species, with three proposed catchables.
- **Design**: what food does, and the decisions.

What it proposes:
- **Fifteen kitchen stations in five tiers**, each built from M3's materials
  and placed like M3's: campfire, drying rack (tier 0); quern, hearth and
  pot, brine and ferment barrel, churn and cheese press (1); brick bake oven,
  smoker, pasta and noodle table, pizza oven, oil press, icebox (2); iron
  stove and wok, fryer (3); a ship's galley (4).
- **About a hundred recipes with real chains**: grain to flour to dough to
  bread, rolls, pastry, pies and cake; pizza (dough, tomato sauce, cheese,
  toppings); pasta (semolina and egg dough: spaghetti, ravioli, lasagne) and
  noodles (wheat and rice; ramen and noodle soup); stir fry (wok, prawns,
  soy, chili); meats by cut in four states; fish (grilled, smoked, salt,
  fish and chips, sushi from fish safe raw); calamari, prawn tempura and
  skewers, crab and crayfish boils, lobster, clam chowder, grilled octopus,
  mussels; soups and stews; tea, juice and ale; hardtack and cans.
- **Meals give buffs; nothing drains.** No hunger bar. A dish gives one of
  five effects for minutes of play, up to three at once: stride (sprint and
  swim), steady hands (fishing), keen (digging), glow (a small light) and
  warmth (which waits for a cold mechanic). Asked as Q1 with screenshots.
- **Freshness from played time**, stale at a food's keep and compost at
  twice it; salt, smoke, drying, brine, ice and cans keep food longer.
- **Quality of one to three stars, worked out, never rolled**: freshness,
  the right heat, practice.
- **Recipes learned** from townsfolk, books and discovery; **food sold** to
  innkeepers and stalls, with provisions proposed as a good for M12.
- **Twelve new crops for M8** (tomato, durum wheat, soy, bok choy, onion,
  garlic, sugar cane, sugar beet, herbs, potato, olive, tea), seeds only.
- **Three new catchables for M25** (sand prawn, sand clam, rock mussel).

## Capabilities

### New Capabilities

- `player/cooking`: kitchen stations, recipes, heat, freshness, quality,
  eating and its effects, and how food is saved.
- `world/crops`: the twelve crops appended to M8's roster, and perennials.

### Modified Capabilities

None. The fauna roster's three new catchables are listed in the design as a
proposal to `ocean-environments`, whose `world/fauna` delta owns that roster.

## Impact

- **Inventory and saves:** a new `Item::Food(u16)` into an append-only food
  roster, with a made-at world time and stars on the stack; caught fish stay
  `Fish`, so no save migrates. Placed stations are stored records like M3's.
- **Data:** `assets/config/recipes.ron` (M3's) gains the kitchen recipes;
  new `food.ron` (keeps, effects, durations) and crop entries.
- **HUD:** buff icons over the hotbar, only once buffs exist.
- **Systems touched by effects:** walking, fishing, digging, lights. Nothing
  else changes behaviour.
- **Generator:** none. Crops are seeds and catchables are fauna; nothing new
  is generated in the ground, so no terrain version changes.
- **Performance:** not measured. This is a mockup in a cloud session with no
  GPU; the screenshots ran on SwiftShader and say nothing about frame time.
