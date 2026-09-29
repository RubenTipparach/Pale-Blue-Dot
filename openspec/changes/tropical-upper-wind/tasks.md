# Tasks

## 1. Measure and choose

- [x] 1.1 (2026-09-29, the design's table and `docs/screenshots/tropical-upper-wind/`.) The wind at cloud height as built and under candidates A, B and C, on version 5's settled climate through a day: zonal means in 2.5-degree bands, the steepest change within 35 degrees, and the day-mean map of each. Verify: the table is in the design and the maps are checked in.
- [x] 1.2 (2026-09-29: "are the jets fixed? just do recommended", candidate C.) **Gate:** the owner answers W1 (which candidate). Record the words in `proposal.md`, and move the spec delta's numbers to the chosen candidate's.

## 2. The wind

- [x] 2.1 (`tropical_easterly_mps`, 8 m/s; `a_tropical_easterly_out_of_range_is_refused`.) `tropical_easterly_mps` in `AtmosphereSettings` and `assets/config/atmosphere.ron`, validated finite and within 0..`jet_max_mps`, with its unit, and in the code-defaults test. Verify: the settings tests; a negative or non-finite value is refused.
- [x] 2.2 (On the shipped level-5 state: 8.0-8.3 m/s from the east at the equator, steepest 3.2 m/s a degree; the old rule measures 6.0 and fails both tests.) The `aloft` stage: the cap before the fade, the fade's range, and the easterly weighted by what the fade leaves. Verify: a core test on a settled climate at level 3 holds the three scenarios (the equator blows westward at no less than half the easterly; no step between 2.5-degree bands steeper than 3.5 m/s per degree within 35 degrees; with the easterly at zero the equator is the surface wind). It fails on the rule as built.
- [x] 2.3 (The design's "Built": 10 days at level 5, before and after.) The climate report run before and after, and its numbers in the design. Verify: the `atmospheric-circulation` claims still hold, including that the cloud-level wind's zonal-mean maximum lies poleward of a subtropical minimum.

## 3. What new worlds start from

- [ ] 3.1 The settled climates made again for every carried generator, at levels 3 and 5. Verify: a new world's days hold 15 ± 0.5 °C at each, and the tests that restore them pass.
  - [x] 3.1a Level 3, generators 4 and 5 (60 days at 14.71-15.09 °C).
  - [ ] 3.1b Level 5, generators 4 and 5: on the owner's desktop ("we can run this sim on my desktop later!"), `settle_climate -- 5 2 30 assets/climate <generator>`, about two hours each. Then the stricter settings-file check can come back (design, "Built").
  - [ ] 3.1c Generator 6, both levels, on the `taller-mountains` branch after survey B6 (they are remade then anyway).
- [x] 3.2 (Four pre-change states restored byte for byte and ran 600 steps finite, 14.9-15.1 °C.) An old save opens and carries on, with its weather as saved. Verify: a save made before the change restores byte for byte, and its next step is finite.

## 4. Seen

- [x] 4.1 (8.5 m/s at the equator in the frames' day mean.) The map mockup's weather frames made again, and the mockup republished. Verify: its Jet overlay has no calm band.
- [x] 4.2 (`game-globe-*` and `game-map-*`, described in the README.) Before and after `--capture` shots of the globe's Jet overlay from orbit over the equator, and of the map screen's. Verify: in `docs/screenshots/tropical-upper-wind/`.
- [x] 4.3 (In `openspec/specs/world/weather`, naming both tests.) Sync the requirement into `openspec/specs/world/weather`, naming the test. Verify: `openspec validate --all`.
- [ ] 4.4 Frame cost not measured in the cloud. The owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
