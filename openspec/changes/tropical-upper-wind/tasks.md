# Tasks

Nothing below group 1 starts before the owner answers W1.

## 1. Measure and choose

- [x] 1.1 (2026-09-29, the design's table and `docs/screenshots/tropical-upper-wind/`.) The wind at cloud height as built and under candidates A, B and C, on version 5's settled climate through a day: zonal means in 2.5-degree bands, the steepest change within 35 degrees, and the day-mean map of each. Verify: the table is in the design and the maps are checked in.
- [ ] 1.2 **Gate:** the owner answers W1 (which candidate). Record the words in `proposal.md`, and move the spec delta's numbers to the chosen candidate's.

## 2. The wind

- [ ] 2.1 `tropical_easterly_mps` in `AtmosphereSettings` and `assets/config/atmosphere.ron`, validated finite and within 0..`jet_max_mps`, with its unit, and in the code-defaults test. Verify: the settings tests; a negative or non-finite value is refused.
- [ ] 2.2 The `aloft` stage: the cap before the fade, the fade's range, and the easterly weighted by what the fade leaves. Verify: a core test on a settled climate at level 3 holds the three scenarios (the equator blows westward at no less than half the easterly; no step between 2.5-degree bands steeper than 3.5 m/s per degree within 35 degrees; with the easterly at zero the equator is the surface wind). It fails on the rule as built.
- [ ] 2.3 The climate report run before and after, and its numbers in the design. Verify: the `atmospheric-circulation` claims still hold, including that the cloud-level wind's zonal-mean maximum lies poleward of a subtropical minimum.

## 3. What new worlds start from

- [ ] 3.1 The settled climates made again for every carried generator, at levels 3 and 5. Verify: a new world's days hold 15 ± 0.5 °C at each, and the tests that restore them pass.
- [ ] 3.2 An old save opens and carries on, with its weather as saved. Verify: a save made before the change restores byte for byte, and its next step is finite.

## 4. Seen

- [ ] 4.1 The map mockup's weather frames made again, and the mockup republished. Verify: its Jet overlay has no calm band.
- [ ] 4.2 Before and after `--capture` shots of the globe's Jet overlay from orbit over the equator, and of the map screen's. Verify: in `docs/screenshots/tropical-upper-wind/`.
- [ ] 4.3 Sync the requirement into `openspec/specs/world/weather`, naming the test. Verify: `openspec validate --all`.
- [ ] 4.4 Frame cost not measured in the cloud. The owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
