# Tasks

It goes with step 2a. The owner's choice is made on the `world-map` mockup
(its task 1.5), and nothing below group 1 starts before it.

## 1. Measure the candidates (instrument only)

- [x] 1.1 In the `world-map` instrument, a mode that sweeps `moisture_m` over 188, 375 and 750 m. For each scale it reports the moisture's quantiles over the temperate land, and the `desert_below` and `wet_above` that give a third each. Verify: the table is printed and copied into this design.
- [x] 1.2 (The map mockup's "Biome size" buttons switch between today, 188, 375 and 750 m, with each one's shares in the legend: `docs/mockups/world-map/biomes-{today,188,375,750}.png`, `docs/screenshots/world-map/biomes-750.jpg`.) Biome rasters for each candidate at its tuned thresholds, into the map mockup's biome layer. Verify: the mockup switches between them and shows each one's shares in its legend.
- [x] 1.3 (Survey B1 and B2, recommendations accepted, 2026-09-27: 750 m, about a third each. The owner, 2026-09-28, after the map's 750 m preview: "did you do large biomes too? I see you had preview with map stuff". The numbers are in the design's measured table: `moisture_m` 750, `desert_below` 0.468, `wet_above` 0.544.) **Gate:** the owner picks a scale and confirms the shares on the mockup. Record the words in `proposal.md`, and put the chosen numbers in this design. Verify: the quote and the numbers are present.

## 2. The version is part of the world

- [x] 2.1 (`TENEBRIS_V4` verbatim, `TENEBRIS` its alias until version 5 lands, and `for_version`. The digest test pins version 4's altitude and biome on 10,000 directions. The modified "The generator is versioned" is synced into `openspec/specs/planet/terrain`.) `TerrainConfig::TENEBRIS_V4` (today's values, verbatim) and `TerrainConfig::for_version`. Verify: core tests that version 4's config generates the same altitude and biome as today's on 10,000 seeded directions, and that an unknown version answers `None`.
- [ ] 2.2 Version 5 added to the versions the world identity (`world-persistence` group 2, which lands first) accepts; an old save's identity still reads as 4. Verify: tests that a new world's identity records 5, an old one's 4, and an unknown one is refused.
- [x] 2.2a (`PLANAR_SALT`; `terrain::tests::the_planar_sampler_is_fixed_whatever_the_generator_version` pins its voxels, and fails when the salt is set to 5.) (Decision 6.) The planar sampler's hash salt is its own constant, fixed at 4. Verify: `headless`'s sampler gives the same voxels before and after `GENERATOR_VERSION` moves.
- [ ] 2.2b (Decision 6.) The 47 reads of `TERRAIN` go through one accessor, set once at launch from the world being opened; a world of the other version chosen on the saves screen restarts the game into it (provisional, until the owner answers). Verify: an app test that the accessor answers version 4's config for an old slot and version 5's for a new one.
- [ ] 2.3 `planet_terrain.rs` builds the terrain from the save's version's config. Verify: an app test that an old save's spawn column has the same biome and top block before and after this change.

## 3. Version 5

- [ ] 3.1 Version 5's `moisture_m`, `desert_below` and `wet_above` set to the owner's choice, and `GENERATOR_VERSION` set to 5. Verify: `distribution_report` is re-run, and its output replaces the shares in `docs/tenebris-comparison.md`.
- [ ] 3.1a (Decision 6.) The settled climates re-made on version 5's terrain, levels 3 and 5, and the shipped `.ron` records the generator it was made on; `shipped_settled` falls back where the world's differs. Verify: the settle's new-world days hold 15 ± 0.5 °C on version 5, and a test that a version-4 world does not open on a version-5 state.
- [ ] 3.2 The walk tests become 4 km (three in four cross a biome) and 1 km (a third stay in one). The shares test runs the shipped seed and four others. Verify: all three pass on version 5, and the 1 km-stays test fails on version 4, which proves it tests the change.
- [ ] 3.3 Sync the two modified requirements into `openspec/specs/world/terrain` and `openspec/specs/planet/terrain`, naming each test. Verify: `openspec validate --all`.

## 4. Look and cost

- [ ] 4.1 `--capture` shots from orbit and at the spawn, version 4 beside version 5, in `docs/screenshots/bigger-biomes/`. The `surface`, `seam` and `coast` captures are re-framed if the spawn's view changed. Verify: the pages are committed and linked from the PR.
- [ ] 4.1a The gate video (`step-videos`, showcase `biomes`): an orbit pass over version 4 then version 5, and a low flight and a walk across a desert, a field and a jungle on version 5, each shot captioned with its biome shares. Verify: the gate page is published and linked from the PR.
- [ ] 4.2 Record that frame cost was not measured in the cloud session, and ask the owner to run `tools/perf_suite.py` (version 4's build against version 5's) because of the extra jungle trees. Verify: the note is in the PR. The owner's report goes into `docs/benchmarks/`.
- [ ] 4.3 The owner watches the video and accepts. Verify: the quote is in `proposal.md`. Archive.
