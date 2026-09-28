# Tasks

Nothing below group 1 starts before the owner answers H1 to H3.

## 1. Measure and choose

- [x] 1.1 The relief today and nine candidates, measured by a scratch probe: summit, percentiles, land over 200 m, the Mountains biome's share, and the steps between neighbouring high cells. Verify: the tables are in the design.
- [ ] 1.2 Every place in the engine that bounds or assumes terrain height, with what each needs at the recommended summit. Verify: the list is in the design, each item a task below or marked as holding.
- [ ] 1.3 In-game screenshots of today against the recommended ranges and the peaks, from orbit, from 40 m over a range and at eye level on a slope, built from a scratch config and not committed. Verify: they are on a page the owner can compare, linked from the survey.
- [ ] 1.4 **Gate:** the owner answers H1 (how tall), H2 (version 6 or folded into 5) and H3 (walkable or jagged). Record the words in `proposal.md` and the numbers in the design.

## 2. Version 6

- [ ] 2.1 `TENEBRIS_V6` and the range term in `surface_altitude`, with version 4 and 5's digests unchanged. Verify: core tests that 4 and 5 are unchanged, and the budget, lowland and walkable scenarios on version 6.
- [ ] 2.2 The Mountains biome, snowcap, snow and stone lines moved with the heights. Verify: the biome shares measured and recorded; `grass_is_not_the_majority_on_five_seeds` still holds.
- [ ] 2.3 `GENERATOR_VERSION` 6 and whatever task 1.2 finds. Verify: the app suite.

## 3. Climate, look and cost

- [ ] 3.1 The settled climates made on version 6, levels 3 and 5. Verify: the new world's days hold 15 ± 0.5 °C.
- [ ] 3.2 Captures of version 5 beside version 6: orbit, the map, a range from 40 m, at eye level. Verify: in `docs/screenshots/taller-mountains/`.
- [ ] 3.3 Frame cost not measured in the cloud; the owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
- [ ] 3.4 Sync the modified requirement into `openspec/specs/world/terrain`, naming the tests. Verify: `openspec validate --all`.
