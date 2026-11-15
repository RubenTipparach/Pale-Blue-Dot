# Tasks

Built after `basic-smelting` lands (its station, recipe and goods table). The
owner's word is the request: "so initial game loop will be: crafting tools,
fishing, farming, herding, cooking" (2026-11-14).

## 1. Rules (`pbd-core`)

- [ ] 1.1 Hands as a digging "tool" of speed 0.25 for soft ground, nothing else. Verify: a core test that bare hands break dirt in 4x the stone shovel's time and break no rock, ore or wood.
- [ ] 1.2 Owned tools with a tier; the new-world kit owns nothing. Verify: core tests of the kit, of a higher tier replacing a lower, and of a lower being refused.
- [ ] 1.3 Break time = hardness / tier speed, and ore gated by tier. Verify: a test against M3 section 7's table.
- [ ] 1.4 Tier 0 drops: fibre from grass, loose stone from rock outcrops, and one branch log per tree by hand (stored per tree). Verify: core tests; a test that a fresh world reaches a stone pick by hand within 200 m of its spawn.

## 2. Recipes and stations (data on `basic-smelting`'s mechanism)

- [ ] 2.1 The hand recipes and the workbench recipes in the recipe table, with M3's icons in `assets/items/` (provenance). Verify: a data test that every input and output is a known good.
- [ ] 2.2 The workbench as a placed station, with its panel. Verify: a capture of the panel mid-craft.
- [ ] 2.3 The pack's Craft tab. Verify: a capture.

## 3. Saves

- [ ] 3.1 `owned` with tiers; old lines read as stone tier and owned. Verify: format tests both ways; an old save opens with all four tools.

## 4. In the game

- [ ] 4.1 Captures: a fresh world punching dirt and gathering fibre, the first planks and workbench, a stone pick made, then an iron pick. Each shot in `docs/screenshots/tool-crafting/` with its launch line.
