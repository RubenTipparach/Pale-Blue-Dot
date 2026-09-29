# Tasks

Nothing below group 1 starts before the owner answers B6.

## 1. Measure and choose

- [x] 1.1 (2026-09-29, the design's table and the survey's four maps.) The moisture field's quantiles over version 6's temperate land, the thresholds for a desert of a third, a quarter, a sixth and a tenth of it with the rest split evenly, each biome's share of the land, and the four maps side by side. Verify: the table is in the design and the maps are in the survey.
- [x] 1.2 (2026-09-29: "10.5 % desert", after "its good now" was asked back; `desert_below` 0.422, `wet_above` 0.524.) **Gate:** the owner answers B6 (how much desert). Record the words in `proposal.md` and the thresholds in the design.

## 2. Version 6

- [x] 2.1 (0.422 and 0.524; `grass_is_not_the_majority_on_five_seeds` holds version 6 to a sixth within a few points, fields and jungle over a third, and version 5 to its thirds; versions 4 and 5's digests unchanged. The map mockup's base, tiles and biome layer redrawn.) `TENEBRIS_V6`'s `desert_below` and `wet_above` at the chosen share, with versions 4 and 5 unchanged (their digests). Verify: `grass_is_not_the_majority_on_five_seeds` holds each temperate biome to its new band on five seeds, and fails at version 5's thresholds.
- [ ] 2.2 (Level 3 remade in the cloud on the halved desert: its first true year landed at 15.05 °C, and a new world's 60 days read 14.79 to 15.08 °C; level 5 on the owner's desktop, with `tropical-upper-wind` 3.1b, or after `tropical-weather-aloft` so it is made once.) Version 6's settled climates made again, levels 3 and 5. Verify: a new world's days hold 15 ± 0.5 °C at each level.
- [ ] 2.3 Captures of version 6's biomes beside version 5's: the whole planet's biome map with its key, and the spawn's continent. Verify: in `docs/screenshots/fewer-deserts/`.
- [ ] 2.4 Sync the modified requirement into `openspec/specs/world/terrain`, naming the test. Verify: `openspec validate --all`.
- [ ] 2.5 Frame cost not measured in the cloud: less desert is more jungle, so more trees. The owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
