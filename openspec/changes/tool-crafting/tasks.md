# Tasks

Built on `farming-and-smelting` (`claude/phase-1`): its station, recipe and
goods table. The owner's word is the request: "so initial game loop will be:
crafting tools, fishing, farming, herding, cooking" (2026-11-14).

## 1. Rules (`pbd-core`)

- [x] 1.1 Hands as a digging "tool" of speed 0.25 for soft ground and placed lights, nothing else. Verify: a core test that bare hands break dirt in 4x the stone shovel's time and break no rock, ore or wood.
- [x] 1.2 Owned tools with a tier; the new-world kit owns nothing. Verify: core tests of the kit, of a higher tier replacing a lower, and of a lower being refused.
- [x] 1.3 Break time = matrix time / tier speed, and ore gated by tier. Verify: a test against M3 section 7's table.
- [x] 1.4 Tier 0 drops: fibre from grass, loose stone from earth, one branch log per tree by hand and more with an axe (stored per tree); the CPU tree rule mirrors the shader's. Verify: core tests; a route test from empty hands to a stone pick; a ground test of trees, grass and earth within 200 m of the spawn.

## 2. Recipes and stations (data on `farming-and-smelting`'s mechanism)

- [x] 2.1 The hand recipes and the workbench recipes in the recipe table, tool outputs with tiers, M3's icons in `assets/items/` (provenance). Verify: a data test that every input and output is a known good.
- [x] 2.2 The workbench as a placed station (code 20, drawn as wood with a plank top), with its panel. Verify: a capture of the panel mid-craft.
- [x] 2.3 The pack's Craft row (M29). Verify: a capture.

## 3. Saves

- [x] 3.1 `owned` with tiers; old lines read as stone tier and owned; new worlds marked in the identity. Verify: format tests both ways; an old save opens with all four tools; a new world's save reopens with none.

## 4. In the game

- [x] 4.1 The tool picker names tiers; the held tool shows its tier.
- [x] 4.2 Captures: a fresh world punching dirt by hand, the Craft row, the workbench panel mid-craft, a stone pick in hand, an iron pick in hand. Each shot in `docs/screenshots/tool-crafting/` with its launch line.
- [ ] 4.3 Video for the owner's batch (deferred with the others).
