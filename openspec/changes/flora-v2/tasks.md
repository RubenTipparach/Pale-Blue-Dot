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

## 8. The crops and wild plants in the world's style (design 11)

The owner, 2026-11-15: "I see some plants are still using the old sprites, so please get new sprites that have a more unified art style".

- [x] 8.1 Copy `docs/mockups/farming/world/<crop>_<s>.png` (M8's outline-free world stages, the same bytes as M13's `foliage/crops/`) over `assets/items/crops/`; PROVENANCE says so. Verify: `farm::tests::every_stage_has_its_sprite`; a byte comparison with the mockup's copies; `--farm` captures before and after, the field and a wild plant.

## 9. Generator version 10, snow on the branches, vines, fruit to pick (design 12)

The owner, 2026-11-15: "finish up fishing wiki, herding, and flora". Cloud session: lavapipe captures, frame cost not measured.

- [x] 9.1 (Built: `every_version_keeps_where_its_trees_stand`; the x86_64 ground digests of 6 to 9 pinned beside the Mac's per target; both pass on this container.) Pin where version 9's trees stand (a tree digest over 10,000 sample cells) and record the x86_64 ground digests of versions 6 to 9 beside the aarch64 ones. Verify: the digest tests pass on this machine before the table moves.
- [x] 9.2 (Built: `TreeTable::V4`/`V10`, `version_10_is_version_9_s_ground_with_m13_s_trees`, `version_10_grows_m13_s_forests`; GENERATOR_VERSION 10 with version 9's settled climate; captures `v10-temperate-walk.jpg`, `v10-taiga-winter-snow.jpg`, `v10-jungle.jpg`, `v10-beach-acacias.jpg`.) `TreeTable` in `TerrainConfig`: versions 4 to 9 carry today's table, version 10 M13's (design 12.2) with the beach's sand eligible; `grows_tree` takes the world's table at every caller. Verify: version 10's ground digest equals version 9's; version 9's tree digest is unchanged; version 10's tree counts per biome match 12.2 within sampling.
- [x] 9.3 (Built: the table packed in `plant_means[b].w`'s high half; `the_tree_rule_is_the_foliage_pass_s` mirrors both passes' lines against `grows_tree`; every launch reached READY with 50 pipelines; captures show `LATE_PIPELINE` lines as captures always have.) The foliage pass reads the table from `PlanetParams`; its literals go. Verify: a test that the uniform carries the open world's table, and that no density literal is left in the WGSL; shader validation; a launch to READY with no `LATE_PIPELINE`.
- [x] 9.4 (Built: `snow_lies_on_branches_only_in_a_cold_winter`; `v10-taiga-winter-snow.jpg`. The near pines read nearly white, every needle card facing up: a lighter hand is the owner's call. No fields oak in snow: the fields spawn is at 24 degrees, under the cold ramp.) Snow on the branches (12.3). Verify: `winter_cover` core test and its WGSL twin; a winter capture of a tundra pine and a fields oak beside summer.
- [x] 9.5 (Built: `vines_hang_from_anchors_and_never_reach_the_ground`; `v10-jungle.jpg` shows them only faintly from the ground; a canopy shot is still to take.) Vines (12.4). Verify: core tests (a vine hangs from an anchor, no longer than the trunk); a jungle capture.
- [ ] 9.6 (Built, partly verified. Picking: `hanging_fruit` against the shader's lines (`the_fruit_to_pick_is_the_fruit_the_draw_hangs`), nine fruit goods and icons, the `fruit` record, `fruit::pick` on E; not yet picked in a capture. Hiding: `trees_taken` lists felled and picked trees in the uniform and both passes leave felled ones out; `v10-trees-taken-compare.jpg` (`--take-trees`, the 11 felled trees 26 to 39 m out) shows background trunks gone. Open: the instrument finds no tree nearer than 26 m though trunks stand 3 m from the camera, so its distances or its records are off; and the picked fruit's absence is not yet shown.) Fruit to pick (12.5): goods and icons, `flora::fruit_of`, E on a tree, the `fruit` record, the picked and felled bits. Verify: a test that a picked tree gives nothing until it regrows and that the record survives a reload; captures of a tree before and after picking, and a felled tree gone.
- [x] 9.7 (Built: `the_ground_follows_the_seasons`; `ground-season-strip.jpg`, day 2, 27 and 52 at one fields spot.) The ground follows the seasons (12.6): `ground_season` in the core, the grass top, the sod lip and the tufts tinted dry and frosted by it in the surface shader. Verify: a test holding the WGSL constants to the core's; captures at one fields spot and forest edge at `--day 2`, `27` and `52`, side by side.
- [x] 9.8 (Built 2026-11-16 in the core; the three tests pass; no capture: ore is drawn as stone, 9.9.) Ore veins in version 10 (12.8): `OreVeins` in `TerrainConfig` (None to 9), stone turned to mixed ore in clustered veins in `column::generate_solid`. Verify: version 10's columns equal version 9's but for stone turned to ore (every target); a v10 column digest; ore within 14 layers of the surface in most sampled land columns; a capture of a dug hillside.
- [ ] 9.9 Ore drawn as ore: a fifth bit in the column's material word (six codes a word, Rust and WGSL), an ore code, and an ore tile on the stone sheet. Verify: a capture of a dug hillside showing the veins; the material word's packing tests.

