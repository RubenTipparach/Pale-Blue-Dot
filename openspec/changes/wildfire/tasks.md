# Tasks

Phase 4, with `survival-mode` (owner, 2026-11-26). The write-up and the
mockup first; game code waits on their approval and a request to build.

## 1. Mockup for the owner

- [ ] 1.1 A forest fire spreading at night in the dry season through trees and grass, lit by its own flames, wind and rain on sliders, the rain putting it out, a dug break holding, and its scar by day (`docs/mockups/survival.html#fire`; drafted 2026-11-26, screenshots `docs/screenshots/survival-mode/mockup/`). Verify: the owner approves the look and the pace.

## 2. The weather

- [ ] 2.1 `ground_wet` in `pbd_core::atmosphere` (design 2), stepped with it, additive in `to_bytes`. Verify: a test that soaked ground dries in about a world day of summer sun, and an atmosphere saved before restores.
- [ ] 2.2 The fire danger (design 2). Verify: tests of each term and of a desert summer, a swamp and a tundra.

## 3. The rule in the core

- [ ] 3.1 `pbd_core::fire`: the fuel table from the generator (design 3), the `blaze` record and its lines, the one-second spread in cell-key order with seeded hashes (design 5), the limits. Verify: the design's Verification tests, a pentagon's five neighbours, and two runs and a reload mid-fire agreeing cell for cell.
- [ ] 3.2 Stopping (design 7): rain, water, breaks, the bucket, ember throws, the yield set. Verify: a test each; a player's field and fence untouched.
- [ ] 3.3 The scar and regrowth (design 8), charcoal from a snag. Verify: tests by day.
- [ ] 3.4 `assets/config/fire.ron` (design 11), validated, the one source of defaults.

## 4. In the game

- [ ] 4.1 The server ticks blazes in Survival worlds only; ignition by a lit torch (design 6). Verify: an app test that a torch on dry grass starts a blaze and on wet grass does not; an Explorer world never makes one.
- [ ] 4.2 Flames, smoke, embers and the fire's light in the cell light field; the burnt-cell table for the ground and tree draws. Verify: `--capture` shots at night at the front, the scar the next day and at day 30, in `docs/screenshots/wildfire/`; on the owner's hardware, `perf_suite.py` before and after with a forced blaze (not in a cloud session).
- [ ] 4.3 Torches in the rain (design 9): `lamp_out`, the held torch, relighting. Verify: an app test with forced rain, a roof keeping one lit, a reload keeping one out; Explorer's never going out.
- [ ] 4.4 Danger to the living (design 10). Verify: an app test that standing in fire hurts and animals flee.
- [ ] 4.5 The dry strike, once `lightning-fire` is built (design 6). Verify: a `--strike-at` test in a dry place and a wet one.
- [ ] 4.6 Multiplayer: two clients, one fire. Verify: the two-machine procedure (`multiplayer` design 20) shows the same scar.

## 5. Close

- [ ] 5.1 Screenshots of each state for the owner; the video task waits for the batch (CLAUDE.md).
- [ ] 5.2 Move `world/fire` into `openspec/specs/` requirement by requirement with the test that proves each; `openspec validate --all`.
