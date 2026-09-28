# Tasks

Nothing below group 1 starts before the owner answers H1 to H3.

## 1. Measure and choose

- [x] 1.1 The relief today and nine candidates, measured by a scratch probe: summit, percentiles, land over 200 m, the Mountains biome's share, and the steps between neighbouring high cells. Verify: the tables are in the design.
- [x] 1.2 (Surveyed 2026-09-28: the column tier's 175 m top and its 9-bit fields, and the 300 m cloud base; the rest scales, and the soft effects are listed. The design's "Surveyed" section, decisions 5 to 7.) Every place in the engine that bounds or assumes terrain height, with what each needs at the recommended summit. Verify: the list is in the design, each item a task below or marked as holding.
- [x] 1.3 (2026-09-28: https://claude.ai/artifact/BXtLUoFSif3oVt1eutiuEr, linked from survey H1 and H3. Each candidate at its own tallest summit in daylight, found by sampling 1.5 million points of the scratch build's terrain (wide ranges 249 m at 34.8 S 116.6 W; peaks 310 m at 19.1 N 13.1 W, today's tallest ridge, 157 m), from 700 m east at 60 m and 200 m, and on the map, beside today's ground from the same camera. Cloud and rain are switched off in the scratch config, since the new settled climate rains over both summits at their noons, and the snow line is 200 m, the owner having asked for snowy tops.) In-game screenshots of today against the recommended ranges and the peaks, from orbit, from 40 m over a range and at eye level on a slope, built from a scratch config (with the tier raised, decision 5) and not committed. Verify: they are on a page the owner can compare, linked from the survey.
- [x] 1.4 (2026-09-28: "Accept everything as recommended", in `proposal.md`: H1 about 290 m, H2 version 6 after PR #18, H3 walkable wide ranges; and saved games are left alone.) **Gate:** the owner answers H1 (how tall), H2 (version 6 or folded into 5) and H3 (walkable or jagged). Record the words in `proposal.md` and the numbers in the design.

## 2. Version 6

- [ ] 2.1 `TENEBRIS_V6` and the range term in `surface_altitude`, with version 4 and 5's digests unchanged. Verify: core tests that 4 and 5 are unchanged, and the budget, lowland and walkable scenarios on version 6.
- [ ] 2.2 The Mountains biome, snowcap, snow and stone lines moved with the heights. Verify: the biome shares measured and recorded; `grass_is_not_the_majority_on_five_seeds` still holds.
- [ ] 2.3 (Decision 7, first.) A test that reads `COLUMN_BASE_M` and `COLUMN_TOP_M` from `planet_surface.wgsl` and `planet_visibility.wgsl` and holds them to `column::BASE_M` and the top of `column::LAYERS`. Verify: it fails when either shader's copy is changed.
- [ ] 2.4 (Decision 5.) `column::LAYERS` 504, and every copy the test in 2.3 and `the_shader_carries_the_reference_light_constants` hold (`LIGHT_WORDS`, `MATERIAL_WORDS`, `LIGHT_LAYERS`, the span). Verify: a save made before opens with its edits where they were; digging and placing work at 300 m; `the_span_covers_the_measured_relief` pins the new summit with 50 m above it.
- [ ] 2.5 (Decision 6.) A test that version 6's summit is under the cloud base. Verify: it fails with the range weight raised past the base.
- [ ] 2.6 `GENERATOR_VERSION` 6, the pinned summit bands in `planet_gen` and `planet::terrain` moved to the chosen band. Verify: the core and app suites.

## 3. Climate, look and cost

- [ ] 3.1 The settled climates made on version 6, levels 3 and 5. Verify: the new world's days hold 15 ± 0.5 °C.
- [ ] 3.2 Captures of version 5 beside version 6: orbit, the map, a range from 40 m, at eye level. Verify: in `docs/screenshots/taller-mountains/`.
- [ ] 3.3 Frame cost not measured in the cloud; the owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
- [ ] 3.4 Sync the modified requirement into `openspec/specs/world/terrain`, naming the tests. Verify: `openspec validate --all`.
