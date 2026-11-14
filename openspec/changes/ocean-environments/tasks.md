# Tasks

The mockup comes first and is the whole of this change until the owner
approves it. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 8 are the build, listed so the mockup is judged
against what it would cost.

## 1. The mockup

- [x] 1.1 `docs/mockups/ocean.html`, "Seas, Rivers and Marshes": ten places as hex-cell scenes by day and at night with their life moving, the light tab, the bestiary, the chart and the engine tab; `tools/art/ocean_art.py`; the game's art in `docs/mockups/ocean/game/`, M7's and M9's in `ocean/m7/`, `ocean/m9/`. Verify: headless Chromium (SwiftShader, no GPU) shows no console errors; every tab opens and draws; one WebGL context for the whole page; the camera stays put when idle; no horizontal scroll at 390 px.
- [x] 1.2 Screenshots of each place by day, at night and close on its life, the tide, the storm, the dam's days, the light at 10, 40, 90 and 140 m today and proposed, in `docs/screenshots/mockup-ocean/`. Verify: they are there, and say they were rendered on SwiftShader.
- [ ] 1.3 Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
- [ ] 1.4 Put Q1 to Q3 in the survey with their screenshots. Verify: they are in `docs/design/survey.html`.

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` and the survey answers, and fold them into this design. Verify: the stage is in `mockup-work.md`, and the quotes are in `proposal.md`.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video of the page when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. Light under water (on request, after approval)

- [ ] 4.1 `downwell_per_m` split from `absorption_per_m`, and `turbidity` per environment. Verify: a capture at 10, 40, 90 and 140 m on the owner's desktop beside the mockup's light tab; a core test that the zones fall at 40 and 90 m for ocean water.
- [ ] 4.2 Caustics, shafts and marine snow. Verify: captures; `tools/perf_suite.py` before and after on the owner's hardware.
- [ ] 4.3 Spray and streaked foam from a gale up. Verify: a storm capture beside `sea-storm-day.jpg`.

## 5. Tides (on request, after approval)

- [ ] 5.1 The tide as an offset of the sea function, from the body's moon. Verify: the shader-against-core test still agrees within 1 mm with the tide on; a core test of the range and period.

## 6. Places in the generator (on request, after approval)

- [ ] 6.1 `ocean::environment` in `pbd-core`, read by the generator, the spawner and the map. Verify: a test that the three agree on a sample of directions; the map's chart layer.
- [ ] 6.2 Reef, kelp, vent, trench and sea-ice cells and pieces in a new generator version. Verify: the old version's digest test unchanged; a new digest test for the new version; an old save opens on its own version.

## 7. Water life (on request, after approval)

- [ ] 7.1 The new species appended to `fauna.ron` with `env:` tags; the bed crawler and the surface floater. Verify: the append-only test; spawn tests per environment.
- [ ] 7.2 Behaviours: clams, the octopus, tube worms, seals, ducks. Verify: core tests of each state machine against the eye's distance.
- [ ] 7.3 Protected species refuse capture. Verify: a test per species.

## 8. Fresh water (after volumetric water phases 2 and 3)

- [ ] 8.1 A drainage river pass that steps down from springs, in a new generator version. Verify: a test that every river cell drains to the sea; captures beside the river tab.
- [ ] 8.2 The beaver colony record and the `beaver` world process through the journal. Verify: a catch-up test to day 40; a test that a broken dam is not rebuilt on the player's cells and the colony moves on.
- [ ] 8.3 Marsh pools and the estuary. Verify: captures beside the marsh tab.
