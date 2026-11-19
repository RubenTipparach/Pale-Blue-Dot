# Tasks

Nothing here is started. The write-up and the mockup are this change. Code
waits for the owner's request.

## 0. Mockup and write-up (this commit)

- [x] 0.1 The write-up: proposal, design, spec deltas, tasks.
- [x] 0.2 Mockup: the Glow and carried light tab on the Foliage Detail
  Playground (M13, `docs/mockups/foliage.html`, version 2), with screenshots in
  `docs/screenshots/glowing-mushrooms/`.
- [ ] 0.3 Survey: the rounder cap (design 1), with `cap-today.png` and
  `cap-proposed.png`. The coordinator publishes it.
- [ ] 0.4 The owner approves the tab for build, then for the game.

## 1. Speed first: the carried light (part B)

- [ ] 1.1 `pbd_core::light::{CarriedLight, carried_light}` from
  `Material::emission`, `dusk_lit` and `GLOW_FLOWER_LEVEL`; `GLOW_HEAD_RGB`
  moved to the core. Verify: tests that every emitting material maps to its
  emission, and that a pickaxe and a pack-only torch give none.
- [ ] 1.2 `carried(p, n)` and the column walk in `planet_surface.wgsl`, with
  the falloff shared with `lamp_bake::falloff` by the constants test, and the
  early outs. Verify: Naga validation; a capture at midnight with a torch,
  beside the wall and the doorway.
- [ ] 1.3 `CarriedLights` uniform in `field_lit.wgsl` with the per-corner
  occlusion from the CPU; the hand and held item unoccluded at
  `carried_hand_min_m`. Verify: a capture of a drop and the hand at night.
- [ ] 1.4 `water.wgsl` takes the same term (check whether the column records
  are bound there; if not, the falloff alone). Verify: a capture at a pond's
  edge at night.
- [ ] 1.5 Merging the body glow with a held light, the rise and the fade, and
  `carried_max_lights` with other players' lights. Verify: unit tests of the
  merge and the fade curve.
- [ ] 1.6 The owner's measurement (design 13): `night-carry` in
  `perf_suite.py`, A/B with `PBD_CARRIED=0` and `PBD_CARRIED_WALK=0` on the
  RTX 3070 and the M4 Max. Take the fallback if it is over 0.5 ms at p50.
  Not run in a cloud session.

## 2. The glowing mushroom (part A)

- [ ] 2.1 `glow_plant_point` in `pbd_core::flora`, with the shader-lines test
  (as `hanging_fruit_points`).
- [ ] 2.2 The `glow-plant` record (schema 1), `GLOW_PLANT_REGROW_DAYS`, and
  the digest test pinning `glow_flower_chance` and the salt.
- [ ] 2.3 The record build: the pick clears `GLOW_FLOWER_BIT` and sets
  `GLOW_PICKED_BIT`; the shader's flower slot draws nothing on a picked cell;
  regrowth only beyond `clutter_radius_m`. Verify: a capture before and after
  a pick at night (`--pick-glow`, a capture instrument like `--pick-trees`).
- [ ] 2.4 `desktop/glow_plants.rs`: aim, the `F  Pick` offer, the pick in one
  durable write, the re-light. The good `glowing-mushroom` and its 16 px icon.
  Verify: a save and reload test; the toast in a capture.
- [ ] 2.5 `Effect::Glow`, the dish row in `cooking.ron`, the buff icon and its
  timer. Verify: the cooking tests extended (refresh, three effects, reload).
- [ ] 2.6 Move the built requirements into `openspec/specs/` with their tests,
  in the commits that make them true.
