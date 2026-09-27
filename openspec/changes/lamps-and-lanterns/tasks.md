# Tasks

Step 1 of the owner's plan, and the first approval gate: the owner approves
the lights (group 7) before `world-map` starts.

## 1. Close the two half-open lighting changes

- [ ] 1.1 Move the delta specs of `voxel-light` and `night-and-lamps` from `specs/planet/spec.md` to `specs/planet/light/spec.md`. Verify: `openspec validate voxel-light night-and-lamps --strict` passes.
- [ ] 1.2 Take `night-and-lamps`' unbuilt requirements out of its delta: the torch's icon clause, the flowers and the sampler. They are carried by this change's delta. Verify: its delta holds only "The sun moves, and there is one of it" and "A cell can emit light of its own", each with a passing test named in its scenarios.
- [ ] 1.3 Sync both into `openspec/specs/planet/light/spec.md` with `/opsx:sync`, and name each requirement's test in its scenarios. Verify: `openspec validate --all` passes, and every requirement in the new spec names a test that passes.
- [ ] 1.4 Ask the owner for `voxel-light`'s in-game check (caves dark enough, the crease reads, the mouth's falloff), then archive both changes, moving their held items into this change's non-goals. Verify: neither appears in `openspec list`.

## 2. One sun, one set of light constants

- [ ] 2.1 Frame the "midnight" and "nightshore" captures from the clock's sun, not `sky::SUN_DIRECTION`, and delete the constant once nothing reads it. Verify: `git grep SUN_DIRECTION` is empty, and the two captures still find the antisolar point at a pinned `--time`.
- [ ] 2.2 Move the torch's tint and gain into `pbd_core::light` and add them to the shader-constant test. Verify: the test fails when one of the two copies is changed.

## 3. Moving things take the field's light

- [ ] 3.1 `light_at(point)` in `pbd_core::light`, blending the hex neighbours and the two layers, and answering full sky and no block light outside the region. Verify: core tests that a point in a sealed cave reads dark, a point beside a lamp reads the lamp, and a point off the region reads the open sky.
- [ ] 3.2 Sample the eight corners of the player's body, the held tool and the ship once a frame, and hand them to their materials as a uniform. Verify: an app test that a sealed cave gives eight dark corners.
- [ ] 3.3 Blend the eight samples in the moving-thing shaders, and extend the shader-constant test to the new uniform's layout. Verify: `--capture` shots of the ship parked in a cave at night and of the player beside a torch, checked into `docs/screenshots/lamps-and-lanterns/`.

## 4. Lamps that come on at dusk

- [ ] 4.1 `daylight::is_dusk_lit(clock)` from the sun's elevation, with hysteresis. Verify: core tests that it is true at midnight and false at noon, and that it does not flip back and forth at dusk.
- [ ] 4.2 Split emitters into always lit and dusk-lit, and re-bake the tier on the worker when the answer changes. Verify: an app test that a dusk-lit emitter lights nothing at noon and lights its cells at midnight, and that a torch lights them at both.

## 5. The lantern family, the brazier and the candle

- [ ] 5.1 Add the materials `LanternPost`, `LanternWall`, `LanternHanging`, `Brazier` and `Candle`, with emission levels, dusk-lit flags, not solid and not opaque. Verify: core tests of each emission level, that none of them is solid, and that the candle's reach is shorter than the brazier's.
- [ ] 5.2 Their geometry in the shader. The wall lantern's facing comes from its solid neighbour, and the hanging lantern hangs from the cell above. Verify: `--capture` shots of each by day and at night.
- [ ] 5.3 Icons for the torch and for each new light, as committed PNG sources with a manifest entry. Verify: the hotbar shows each light's own icon in a capture, and the art checks (palette, transparency) pass.
- [ ] 5.4 A kit grant, version N+1, that deals a few of each new light once. Verify: the existing kit tests extended with an old save gaining them once and a new world's kit holding them.
- [ ] 5.5 A synthetic "city" of 300 lanterns, baked and timed in a test. Verify: its bake time is recorded in the design's risk note. If it is over 12 ms, open a follow-up before `cities-in-the-world`.

## 6. Glowing flowers

- [ ] 6.1 A share of flower cells chosen on the CPU, carried as a bit in the cell record and fed to the emitters as dusk-lit. Verify: a test that the bit and the emitter list agree for every cell of a spawn tier.
- [ ] 6.2 The shader draws the glowing head only where the bit is set. Verify: `--capture` of a meadow at night and the same meadow by day, in `docs/screenshots/lamps-and-lanterns/`.

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
- [ ] 7.3 The owner watches the video and approves the lights. Verify: the owner's words are quoted in `proposal.md` under Why. Then archive this change.
