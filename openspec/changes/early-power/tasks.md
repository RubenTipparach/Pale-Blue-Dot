# Tasks

Planning only. Nothing below the mockup starts until the owner asks for it
(CLAUDE.md, "Write it up before touching code").

## 0. The write-up and the mockup
- [x] 0.1 This change: the owner's words, the tree, the recipes, the mills,
      the lines, no batteries, the first appliances, `power.ron`'s shape.
- [x] 0.2 M33 version 10: the POWER tab (`#power`) and the MILLS tab
      (`#mills`, one lazy 3D view, lit at night, still until Turn); icons by
      `tools/art/early_power_icons.py`, sails and wheel textures by
      `tools/art/early_power_art.py`.
- [x] 0.3 Screenshots in `docs/screenshots/early-power/` (headless,
      SwiftShader, said so); the tracker's M33 card and roadmap step 4a
      updated in the repo.
- [ ] 0.4 The owner approves the mockup for build on the tracker.
- [ ] 0.5 The step's video, for the owner's batch.

## 1. Data
- [ ] 1.1 `assets/config/power.ron` with its validation; a test that loads it
      and refuses a bad row.
- [ ] 1.2 The recipes of design 3 in `recipes.ron`, names in the locale file.
- [ ] 1.3 The Power branch of design 2 in `infrastructure_missions.ron`
      (`infrastructure-missions` task 3.5), not a file or record of its own.

## 2. Rules (`pbd-core`)
- [ ] 2.1 The windmill's wind share and rise rule; tests at 2, 5, 9 and 22 m/s.
- [ ] 2.2 The water mill's output from flow; the sluice.
- [ ] 2.3 The network: poles in reach, joins, supply and demand per tick, the
      share; a test that a shut sluice stops every appliance the same tick.

## 3. In the world (`pbd-app`)
- [ ] 3.1 The windmill, water mill, poles, wire and lamps as textured,
      smooth-shaded models, lit as the terrain is.
- [ ] 3.2 Placement messages; the station panels show the wind or the kW.

## 4. Proof
- [ ] 4.1 Before-and-after screenshots; the bot plays the Power branch.
