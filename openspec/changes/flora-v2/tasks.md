# Tasks

Built on `claude/flora-v2` (2026-11-15): 1, 2 and 4; the leaf cards (3.1)
were built and removed at the owner's word, and the baked impostors (3.2)
are not built; captures and their notes in
`docs/screenshots/flora-v2/README.md`. Frame cost not measured (the owner's
perf sitting); the owner's video waits for the batch.

Ordered by value. Captures go in
`docs/screenshots/flora-v2/`, before and after, from fresh worlds.

## 1. Pixel tuft grass and its toggle

- [x] 1.1 `pbd_core::flora`: the tuft ramps per biome and the per-biome table. Verify: core tests that every biome has a ramp, dark to light by luminance, and that the table's weights are non-negative and the desert grows no fern.
- [x] 1.2 `PlayerSettings::pixel_grass`, file version 5, the GRAPHICS check row, `--grass tufts|blades`, and `FloraLook` to the render world. Verify: settings round-trip and old-file tests.
- [x] 1.3 The uniform's flora lanes (both shaders), the `scatter.ron` knobs and their validation. Verify: the Params struct-size test; config validation tests.
- [x] 1.4 Tufts in the grass slots: crossed cards, the fragment's pattern and ramp, the cut-out in `dropped`, the sway. Verify: shader compiles; captures of tufts in the fields, jungle and swamp, and the blades unchanged with it off.

## 2. Fading and richer clutter

- [x] 2.1 Every clutter piece dithers out over `clutter_fade_m`. Verify: shader test that `dropped` gates kind 3 by the clutter reach.
- [x] 2.2 Flower colours from the biome palette; mushrooms and ferns, with the visibility pass listing them on the same rolls; vertex budget 564. Verify: the visibility GPU test's draw arguments; captures.

## 3. ~~Leaf cards~~ (dropped by the owner, 2026-11-15)

- [x] ~~3.1 Eighteen cards per tree in the foliage draw (306 vertices), the leaf mask, the reach dither, shadows through the cut-out.~~ Built, then removed: the owner, "Hmm trees have weird leave structures on them I don't really like". Q1 is A, trees as today (the owner's own decision); the foliage draw is back to 198 vertices.
- [ ] 3.2 Baked mid and far impostors (design 4, Q2). Deferred: not in this change.

## 4. Seasons

- [x] 4.1 `pbd_core::flora::season_at`, mirrored in WGSL, a test holding the constants together. Verify: core tests (hemispheres opposite, no seasons at the equator, peaks where the design puts them).
- [x] 4.2 The tints and the winter thinning in the shader; the year in the uniform. Verify: captures at `--day` for autumn and winter.

## 5. Checks

- [x] 5.1 `cargo fmt --all -- --check`, clippy, the relevant tests, `openspec validate --all`.
- [x] 5.2 A fresh world launched (not a capture) to READY with no ERROR and no `LATE_PIPELINE`.
- [ ] 5.3 The fungal forest (Q7). Not in this change: a new biome is a new generator version, its own change.
- [ ] 5.4 The owner's video waits for the batch (CLAUDE.md, 2026-09-27); frame cost is the owner's perf sitting.

## 6. The approved mockup into the engine (M13 approved 2026-11-15; design 9)

Built on `claude/env-flora` (2026-11-15). Captures and an honest read of each
in `docs/screenshots/flora-v2/README.md` (the `m13-*` set). Frame cost not
measured. The dry olive beach crowns (6.3) are built but were not caught in a
capture: no beach tree was in view.

- [x] 6.1 Painted flowers off the ground tiles: `no_flowers.py`'s rules as a step of `tools/build_tileset_atlas.py`; atlas rebuilt. Verify: `--check` passes; the atlas equals the mockup's copy.
- [x] 6.2 Tuft ramps from each biome's ground tile, and the plants' tile means, in `pbd_core::flora`. Verify: a test recomputes both off `atlas.png`.
- [x] 6.3 Plant contrast pulled to the tile's mean (crowns and bushes 0.45; trunks, twigs, pebbles 0.7); beach crowns and bushes in the dry olive gradient map. Verify: shader compiles; captures.
- [x] 6.4 Clutter reach doubled (120 m, fade 30 m). Verify: config validation; captures.
- [x] 6.5 Fallen leaves in the cap fragment: `Season::litter`, thicker under a tree, browned in winter, deciduous biomes only; `leaf_fall`, `leaf_under`, `leaf_size`. Verify: core tests on `litter`; the season constants test; captures at `--day` in autumn, winter and summer.
- [x] 6.6 Fruit and pine cones on the trees, and fallen fruit: foliage draw 258 vertices, offsets moved in both shaders; kinds per biome; season keep. Verify: the vertex-arithmetic tests; captures close on fruit trees.
- [x] 6.7 Checks: fmt, clippy, pbd-core and pbd-app tests, `openspec validate --all`; a normal launch to READY with no ERROR and no `LATE_PIPELINE`. Failing before this group and untouched by it: `planet_gen` version 4 and 5 digests, `husbandry` (shore crab class), and two `settings` file-version tests (they expect version 5; the file is 6 since the env-v2 merge).
- [ ] 6.8 Not built (design 9.9): the winter sod lip (no seasonal snow in the game), unbaked far trees (no impostors), the fungal forest (a new generator version), crops and forage plants (phase 1's are reused), picking fruit.

## 7. The detailed trees (M13's default; design 10)

The owner, 2026-11-15: "why didnt you implement new trees for env 2". Built on `claude/env-trees` (2026-11-15); captures and their read in `docs/screenshots/flora-v2/README.md` (`trees-*`). Frame cost not measured (the owner's perf sitting); the owner's video waits for the batch. Failing before this group and untouched by it: the `planet_gen` version 4 and 5 digests.

- [x] 7.1 Correct the record: design 4, the Q1 row and the proposal say the owner disliked cards on prisms and approved M13's detailed trees.
- [x] 7.2 `pbd_core::tree_kit`: the mockup's builders per species, four seeded variants, the anchors, the biome species mix and the draw-time pick. Verify: core tests (determinism, every variant inside its bounds, every leaf card rooted on a branch, the mix sums to one, vertex counts reported).
- [x] 7.3 The CPU impostor bake: 8 views of 48 x 72, Sweaft-64 in OKLab, block fold, leafy and bare. Verify: core tests (alpha 0 or 255 only, every pixel a palette colour, a view is not empty, deterministic).
- [x] 7.4 The kit buffer (draw binding 11), the near list (cull binding 8), the eight near draws, the impostor draw at 30 vertices, offsets moved in both shaders and the test arithmetic; the main pass and the sun cascades. Verify: shader compiles; the vertex-arithmetic tests; a launch to READY with no ERROR and no `LATE_PIPELINE`.
- [x] 7.5 The near tree in the shader: bark and leaf textures, masks, the mean pull, wind, fruit on anchors, autumn and winter; the mid cards and far billboard; the dithered cross-fade. Verify: captures of every species, autumn and winter, and a walk-away sequence beside the mockup's.
- [x] 7.6 Cacti as desert scenery (not choppable). Verify: `grows_tree` unchanged (its test); capture.
- [x] 7.7 Checks: fmt, clippy, pbd-core and pbd-app tests, `openspec validate --all`.
- [ ] 7.8 Not built (design 10.8): vines, the alien species, the desert palm, snow on branches. Not seen: the acacia (no beach cell grows a tree) and the swamp's willows (not caught in a capture).
