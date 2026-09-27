# Tasks

It goes with step 2a. The owner's choice is made on the `world-map` mockup
(its task 1.5), and nothing below group 1 starts before it.

## 1. Measure the candidates (instrument only)

- [ ] 1.1 In the `world-map` instrument, a mode that sweeps `moisture_m` over 188, 375 and 750 m. For each scale it reports the moisture's quantiles over the temperate land, and the `desert_below` and `wet_above` that give a third each. Verify: the table is printed and copied into this design.
- [ ] 1.2 Biome rasters for each candidate at its tuned thresholds, into the map mockup's biome layer. Verify: the mockup switches between them and shows each one's shares in its legend.
- [ ] 1.3 **Gate:** the owner picks a scale and confirms the shares on the mockup. Record the words in `proposal.md`, and put the chosen numbers in this design. Verify: the quote and the numbers are present.

## 2. The version is part of the world

- [ ] 2.1 `TerrainConfig::TENEBRIS_V4` (today's values, verbatim) and `TerrainConfig::for_version`. Verify: core tests that version 4's config generates the same altitude and biome as today's on 10,000 seeded directions, and that an unknown version answers `None`.
- [ ] 2.2 `WorldFile.generator: Option<u32>`, where absent reads as 4 and a new world writes `GENERATOR_VERSION`. Verify: format tests for an old save's bytes, a new world's round trip, and an unknown version being refused on load.
- [ ] 2.3 `planet_terrain.rs` builds the terrain from the save's version's config. Verify: an app test that an old save's spawn column has the same biome and top block before and after this change.

## 3. Version 5

- [ ] 3.1 Version 5's `moisture_m`, `desert_below` and `wet_above` set to the owner's choice, and `GENERATOR_VERSION` set to 5. Verify: `distribution_report` is re-run, and its output replaces the shares in `docs/tenebris-comparison.md`.
- [ ] 3.2 The walk tests become 4 km (three in four cross a biome) and 1 km (a third stay in one). The shares test runs the shipped seed and four others. Verify: all three pass on version 5, and the 1 km-stays test fails on version 4, which proves it tests the change.
- [ ] 3.3 Sync the two modified requirements into `openspec/specs/world/terrain` and `openspec/specs/planet/terrain`, naming each test. Verify: `openspec validate --all`.

## 4. Look and cost

- [ ] 4.1 `--capture` shots from orbit and at the spawn, version 4 beside version 5, in `docs/screenshots/bigger-biomes/`. The `surface`, `seam` and `coast` captures are re-framed if the spawn's view changed. Verify: the pages are committed and linked from the PR.
- [ ] 4.2 Record that frame cost was not measured in the cloud session, and ask the owner to run `tools/perf_suite.py` (version 4's build against version 5's) because of the extra jungle trees. Verify: the note is in the PR. The owner's report goes into `docs/benchmarks/`.
- [ ] 4.3 The owner accepts. Verify: the quote is in `proposal.md`. Archive.
