# Tasks

Built on 2026-11-19 on the owner's request ("Approved. Build rest of phase
2"), parts A to C. What stays open is the owner's: the survey's cap (A3), the
measurement on real hardware (1.6) and the placed torch's look (1.8). The
build's departures from the design are in design.md, "As built".

## 0. Mockup and write-up (this commit)

- [x] 0.1 The write-up: proposal, design, spec deltas, tasks.
- [x] 0.2 Mockup: the Glow and carried light tab on the Foliage Detail
  Playground (M13, `docs/mockups/foliage.html`, version 2), with screenshots in
  `docs/screenshots/glowing-mushrooms/`.
- [x] 0.5 The owner's notes on v2 (2026-11-19: "no need for shadow caster from
  my light source, please improve torch look"): the shadow dropped from the
  write-up and the mockup, the cost re-estimated, and the held torch,
  lantern and mushroom redrawn in the approved hand (design 7, 8, 14; M13
  version 3, `torch-v3-*.jpg`).
- [ ] 0.3 Survey: the rounder cap (design 1), with `cap-today.png` and
  `cap-proposed.png`. The coordinator publishes it.
- [ ] 0.4 The owner approves the tab for build, then for the game.

## 1. Speed first: the carried light (part B)

- [x] 1.1 `pbd_core::light::{CarriedLight, carried_light}` from
  `Material::emission`, `dusk_lit` and `GLOW_FLOWER_LEVEL`; `GLOW_HEAD_RGB`
  moved to the core. Verify: tests that every emitting material maps to its
  emission, and that a pickaxe and a pack-only torch give none.
- [x] 1.2 `carried(p, n)` in `planet_surface.wgsl`, with no shadow (design 7),
  the falloff shared with `lamp_bake::falloff` by the constants test, and the
  early outs. Verify: Naga validation; a capture at midnight with a torch
  beside a one-cell wall, the far side lit through it.
- [x] 1.3 `CarriedLights` uniform in `field_lit.wgsl`, no occlusion; the hand
  and held item at `carried_hand_min_m`. Verify: a capture of a drop and the
  hand at night.
- [x] 1.4 `water.wgsl` takes the same term. Verify: a capture at a pond's
  edge at night.
- [x] 1.5 Merging the body glow with a held light, the rise and the fade, and
  `carried_max_lights` with other players' lights. Verify: unit tests of the
  merge and the fade curve.
- [ ] 1.6 The owner's measurement (design 13): `night-carry` in
  `perf_suite.py`, A/B with `PBD_CARRIED=0` on the RTX 3070 and the M4 Max,
  against a budget of 0.1 ms at p50. Not run in a cloud session.
- [x] 1.7 The held look (design 14), only on the owner's approval of the v3
  tab: the torch's haft and head textures, the flame cards and their sheet,
  the embers, the glow, the shared flicker on the light's strength, the holds
  for the torch, lantern and mushroom in `held_tools.ron` with the approved
  hand, and the idle sway. Verify: captures at night beside the mockup's
  `torch-v3-*.jpg`.
- [ ] 1.8 The placed torch matching the held one (design 14): a separate look
  change, asked with `torch-v3-placed.jpg`. Not part of this change until
  the owner says so.

## 2. The glowing mushroom (part A)

- [x] 2.1 `glow_plant_point` in `pbd_core::flora`, with the shader-lines test
  (as `hanging_fruit_points`).
- [x] 2.2 The `glow-plant` record (schema 1), `GLOW_PLANT_REGROW_DAYS`, and
  the digest test pinning `glow_flower_chance` and the salt.
- [x] 2.3 The record build: the pick clears `GLOW_FLOWER_BIT` and sets
  `GLOW_PICKED_BIT`; the shader's flower slot draws nothing on a picked cell;
  regrowth only beyond `clutter_radius_m`. Verify: a capture before and after
  a pick at night (`--pick-glow`, a capture instrument like `--pick-trees`).
- [x] 2.4 `desktop/glow_plants.rs`: aim, the `F  Pick` offer, the pick in one
  durable write, the re-light. The good `glowing-mushroom` and its 16 px icon.
  Verify: a save and reload test; the toast in a capture.
- [x] 2.5 `Effect::Glow`, the dish row in `cooking.ron`, the buff icon and its
  timer. Verify: the cooking tests extended (refresh, three effects, reload).
- [x] 2.6 Move the built requirements into `openspec/specs/` with their tests,
  in the commits that make them true.
