# Tasks

Step 1 of the owner's plan, and the first approval gate: the owner approves
the lights (group 7) before `world-map` starts.

## 1. Close the two half-open lighting changes

- [x] 1.1 Move the delta specs of `voxel-light` and `night-and-lamps` from `specs/planet/spec.md` to `specs/planet/light/spec.md`. Verify: `openspec validate voxel-light night-and-lamps --strict` passes.
- [x] 1.2 Take `night-and-lamps`' unbuilt requirements out of its delta: the torch's icon clause, the flowers and the sampler. They are carried by this change's delta. Verify: its delta holds only "The sun moves, and there is one of it" and "A cell can emit light of its own", each with a passing test named in its scenarios.
- [x] 1.3 Sync both into `openspec/specs/planet/light/spec.md` with `/opsx:sync`, and name each requirement's test in its scenarios. Verify: `openspec validate --all` passes, and every requirement in the new spec names a test that passes.
- [x] 1.4 (Survey L3, the owner 2026-09-27: "its good for now".) Ask the owner for `voxel-light`'s in-game check (caves dark enough, the crease reads, the mouth's falloff), then archive both changes, moving their held items into this change's non-goals. Verify: neither appears in `openspec list`.

## 2. One sun, one set of light constants

- [x] 2.1 Frame the "midnight" and "nightshore" captures from the clock's sun, not `sky::SUN_DIRECTION`, and delete the constant once nothing reads it. Verify: `git grep SUN_DIRECTION` is empty, and the two captures still find the antisolar point at a pinned `--time`.
- [x] 2.2 Move the torch's tint and gain into `pbd_core::light` and add them to the shader-constant test. Verify: the test fails when one of the two copies is changed.

## 3. Moving things take the field's light

- [x] 3.1 `light::sample` in `pbd_core` (decision 10), blending the hex neighbours and the two layers, and answering full sky and no block light outside the region. Verify: core tests that a point in a sealed cave reads dark, a point beside a lamp reads the lamp, and a point off the region reads the open sky.
- [x] 3.2 Sample the eye for the held tool and hand, and the eight corners of the ship's, each fish school's and the float's bounds, once a frame, and hand them to their materials (decision 10: there is no player body). Verify: an app test that a sealed cave gives eight dark corners.
- [ ] 3.3 (Built; the ship at midnight, the ship beside lamps and the hand beside lamps are captured. Open for the ship parked in a cave, which needs a rig that parks it in one.) Blend the eight samples in the moving-thing shaders, and extend the shader-constant test to the new uniform's layout. Verify: `--capture` shots of the ship parked in a cave at night and of the player beside a torch, checked into `docs/screenshots/lamps-and-lanterns/`.

## 4. Lamps that come on at dusk

- [x] 4.1 `daylight::is_dusk_lit(clock)` from the sun's elevation, with hysteresis. Verify: core tests that it is true at midnight and false at noon, and that it does not flip back and forth at dusk.
- [x] 4.2 Split emitters into always lit and dusk-lit, and re-bake the tier on the worker when the answer changes. (Built: the switch re-bakes on the main thread as an edit does, and a tier built on the streaming worker is baked there at its dusk state; the design's risk note says why.) Verify: an app test that a dusk-lit emitter lights nothing at noon and lights its cells at midnight, and that a torch lights them at both.

- [x] 4.4 (`PlanetContact::relit`; `a_relit_set_is_still_served`; `docs/screenshots/inventory-grid/drops-midnight.jpg` digs where `midnight-dig-before-fix.jpg` could not.) The dusk re-bake re-points the contact at the relit set (decision 13). Verify: an app test that after `switch_dusk_lamps` relights the set, the contact serves the new set and answers the same cells; and a capture pinned at midnight digs.
- [x] 4.3 A time-of-day control on the pause panel, as the towns mockup has (a slider, Day and Night buttons, and L). It moves the world clock through the same path the clock uses, so the dusk re-bake runs. Verify: an app test that the control sets the clock and that the dusk-lit lanterns come on when it is moved past dusk. It is in the controls list.

## 5. The lantern family, the brazier and the candle

- [x] 5.1 Add the materials `LanternPost`, `LanternWall`, `LanternHanging`, `Brazier` and `Candle`, with emission levels, dusk-lit flags, not solid and not opaque. Verify: core tests of each emission level, that none of them is solid, and that the candle's reach is shorter than the brazier's.
- [x] 5.2 Their geometry in the shader. The wall lantern's facing comes from its solid neighbour, and the hanging lantern hangs from the cell above. Verify: `--capture` shots of each by day and at night.
- [x] 5.2b A level is a metre (decision 7): the block channel's sideways step costs three levels, and the sky's stays one. Verify: core tests that a brazier lights four cells across and fifteen layers up, a candle two across, and the sky's falloff is unchanged.
- [x] 5.2c The shader adds `(f (2 - f))^2` of the lamp colour, the mockup's curve. Verify: the same captures at midnight, beside the ones before the change.
- [x] 5.2d Trees and ground clutter take the field at their own cell (decision 9). Verify: a night capture beside the lamps in which the grass by a lamp is lit and the grass away from them is not.
- [x] 5.2e Daylight crosses a cell as a lamp does (decision 11, survey L1): one `light::ACROSS` of three levels for both channels, and "Light is what reached a cell" modified in the main spec in the same commit. Verify: core tests that a level tunnel reads 12, 9, 6, 3 from its mouth and is dark at the fifth cell, that a shaft still loses one level a layer, and that open ground is unchanged; a `--spawn mouth` capture at noon before and after.
- [x] 5.3 Icons for the torch and for each new light, as committed PNG sources with a manifest entry. Verify: the hotbar shows each light's own icon in a capture, and the art checks (palette, transparency) pass.
- [x] 5.4 Kit grant 2 (decision 12, survey L2): a new world's kit drops snow, rock and ore and holds the five lights; an old save is dealt them once into whatever room it has, and the log names what did not fit. Verify: the kit tests extended with a new world's ten slots, an old save gaining them once, and a full save gaining nothing and saying so.
- [x] 5.5 A synthetic "city" of 300 lanterns, baked and timed in a test. Verify: its bake time is recorded in the design's risk note. If it is over 12 ms, open a follow-up before `cities-in-the-world`.

## 6. Glowing flowers

- [x] 6.1 A share of flower cells chosen on the CPU, carried as a bit in the cell record and fed to the emitters as dusk-lit. Verify: a test that the bit and the emitter list agree for every cell of a spawn tier.
- [x] 6.2 The shader draws the glowing head only where the bit is set. Verify: `--capture` of a meadow at night and the same meadow by day, in `docs/screenshots/lamps-and-lanterns/`.

## 7. The owner's gate

- [ ] 7.1 The gate video (`step-videos`, showcase `lights`). Each shot is captioned with the requirement it shows:
  - a sealed cave;
  - a torch-lit tunnel with the player walking through it;
  - the ship parked in a cave;
  - each light placed and taken back;
  - a lantern row as dusk passes and on to midnight, then dawn;
  - a candle-lit room;
  - a brazier in a square;
  - the glowing meadow.

  It is published on the gate page with a before-and-after set of stills. Verify: the page exists and is linked from the PR.
- [ ] 7.2 Record that frame cost was not measured in the cloud session (CLAUDE.md), and ask the owner to run `tools/perf_suite.py` on real hardware. Verify: the note is in the PR.
- [x] 7.3 The owner approves the lights. (On the screenshots, 2026-09-27; the video is 7.1's, in the owner's batch.) Verify: the owner's words are quoted in `proposal.md` under Why. Archive this change once 7.1 is done.
