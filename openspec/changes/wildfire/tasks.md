# Tasks

Phase 4, with `survival-mode` (owner, 2026-11-26). The write-up and the
mockup first; game code waits on their approval and a request to build.

## 1. Mockup for the owner

- [ ] 1.1 A forest fire spreading at night in the dry season through trees and grass, lit by its own flames, wind and rain on sliders, the rain putting it out, a dug break holding, and its scar by day (`docs/mockups/survival.html#fire`; drafted 2026-11-26, screenshots `docs/screenshots/survival-mode/mockup/`). Verify: the owner approves the look and the pace.
- [ ] 1.2 Decision 12 (the owner, 2026-11-26: "Let's not spread fire too much for now , consider lots of factors like dryness of plants"): the Fire tab runs `plant_dry` (design 2), the slower spread and the tight limits (design 3, 5), with a dryness overlay, a green river band, green and cured patches and a season control (spring, summer, dry season); screenshots `fire-dryness.png` and `fire-spring-green.png` beside the night, rain and day-after shots (revised 2026-11-26). Verify: the owner approves the size and the pace (survey G9).

## 2. The weather

- [ ] 2.1 `ground_wet` in `pbd_core::atmosphere` (design 2), stepped with it, additive in `to_bytes`. Verify: a test that soaked ground dries in about a world day of summer sun, and an atmosphere saved before restores.
- [ ] 2.2 The fire weather and `plant_dry` (design 2, decision 12), derived and never saved, each factor read from its source. Verify: a test per row of design 2's table of examples; a cell below 0.5 never catching; spring-green grass never catching on a test patch; the dry season half a year apart across the equator.

## 3. The rule in the core

- [ ] 3.1 `pbd_core::fire`: the fuel table from the generator (design 3), the `blaze` record and its lines, the one-second spread in cell-key order with seeded hashes, reading each neighbour's `plant_dry` (design 5), no ember throw, the limits (2 blazes, 300 cells, 40 burning). Verify: the design's Verification tests, decision 12's worked examples inside their ranges, a pentagon's five neighbours, and two runs and a reload mid-fire agreeing cell for cell.
- [ ] 3.2 Stopping (design 7): rain, water, bare and green breaks, grazed pasture, the bucket, the yield set; a burnt cell unburnable until its grass is whole (design 8). Verify: a test each; a player's field and fence untouched.
- [ ] 3.3 The scar and regrowth (design 8), charcoal from a snag. Verify: tests by day.
- [ ] 3.4 `assets/config/fire.ron` (design 11), validated, the one source of defaults.

## 4. In the game

- [ ] 4.1 The server ticks blazes in Survival worlds only; ignition by a lit torch (design 6). Verify: an app test that a torch on late-summer grass starts a blaze and on spring-green grass does not; an Explorer world never makes one.
- [ ] 4.2 Flames, smoke, embers and the fire's light in the cell light field; the burnt-cell table for the ground and tree draws. Verify: `--capture` shots at night at the front, the scar the next day and at day 30, in `docs/screenshots/wildfire/`; on the owner's hardware, `perf_suite.py` before and after with a forced blaze (not in a cloud session).
- [ ] 4.3 Torches in the rain (design 9): `lamp_out`, the held torch, relighting. Verify: an app test with forced rain, a roof keeping one lit, a reload keeping one out; Explorer's never going out.
- [ ] 4.4 Danger to the living (design 10). Verify: an app test that standing in fire hurts and animals flee.
- [ ] 4.5 The dry strike, once `lightning-fire` is built (design 6), with its quiet spell (10 world days within 2 km). Verify: a `--strike-at` test in a dry place and a wet one, and a second strike in the quiet spell starting nothing.
- [ ] 4.6 Multiplayer: two clients, one fire. Verify: the two-machine procedure (`multiplayer` design 20) shows the same scar.

## 5. Close

- [ ] 5.1 Screenshots of each state for the owner; the video task waits for the batch (CLAUDE.md).
- [ ] 5.2 Move `world/fire` into `openspec/specs/` requirement by requirement with the test that proves each; `openspec validate --all`.
