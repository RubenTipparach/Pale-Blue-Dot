# Tasks

The mockup comes first and is the whole of this change until the owner
approves it. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 8 are the build, listed so the mockup is judged
against what it would cost.

## 1. The mockup

- [x] 1.1 `docs/mockups/ocean.html`, "Seas, Rivers and Marshes": ten places as hex-cell scenes by day and at night with their life moving, the light tab, the bestiary, the chart and the engine tab; `tools/art/ocean_art.py`; the game's art in `docs/mockups/ocean/game/`, M7's and M9's in `ocean/m7/`, `ocean/m9/`. Verify: headless Chromium (SwiftShader, no GPU) shows no console errors; every tab opens and draws; one WebGL context for the whole page; the camera stays put when idle; no horizontal scroll at 390 px.
- [x] 1.2 Screenshots of each place by day, at night and close on its life, the tide, the storm, the dam's days, the light at 10, 40, 90 and 140 m today and proposed, in `docs/screenshots/mockup-ocean/`. Verify: they are there, and say they were rendered on SwiftShader.
- [x] 1.5 The owner's review of 2026-11-15 (design 11): every live animal a low-poly 3D body (`ocean/creatures3d.js`), a dropped catch a 2D item sprite ("Drop a catch" on the river tab), the reef packed with instanced corals, plants, rubble and fish, the waterfall's z-fighting fixed, descriptive text in modals. Verify: before and after screenshots in `docs/screenshots/mockup-ocean/`; headless checks as 1.1.
- [x] 1.6 A "Water sim" tab: a hex patch where a click pours a source, digs or places a block, and the water moves by Luanti's liquid rule from `docs/mockups/water-flow/liquid.js` (M28's port, now shared by both pages); play, pause, step, tick speed, reset, renewal toggle, and pond, waterfall and dam presets with a breach. Verify: a poured source spreads level by level over steps (`sim-*.jpg`); M28's presets settle as before and its from-scratch check stays at 0.
- [x] 1.7 Fresh water on cells by the Water sim's rule on every tab, and M28 folded into the Water sim tab (design 12). Verify: as `water-flow` task 1.9.
- [ ] 1.3 (Published 2026-11-14 at https://claude.ai/artifact/W2RzYUDG1pAXy6nZxPHh2z and on the tracker, per `mockup-work.md`; the survey questions, 1.4, are not yet put.) Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
- [ ] 1.4 Put Q1 to Q3 in the survey with their screenshots. Verify: they are in `docs/design/survey.html`.

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` and the survey answers, and fold them into this design. Verify: the stage is in `mockup-work.md`, and the quotes are in `proposal.md`.
  - 2026-11-15: the owner asked to build with "continue with next phase while i rewview" (phase 2). Q1 to Q3 unanswered; their recommendations are taken as provisional (recommendation taken, ask only with screenshots), design "Building it". The review itself stays open.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video of the page when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. Light under water (requested 2026-11-15; branch `claude/underwater-v2`)

- [x] 4.1 (built 2026-11-15; the owner judges `docs/screenshots/underwater-v2/compare-sheet.jpg`) `downwell_per_m` split from `absorption_per_m`, and `turbidity` per environment. Verify: a capture at 10, 40, 90 and 123 m (the seed's deepest floor is 125 m, design B1) beside the mockup's light tab; a core test that the zones fall at 40 and 90 m for ocean water.
- [ ] 4.2 Caustics, shafts and marine snow. Verify: captures; `tools/perf_suite.py` before and after on the owner's hardware. (Built 2026-11-15 and captured, `after-*-sun.jpg`; frame cost NOT measured, the perf suite waits for the owner's hardware, so this stays open.)
- [ ] 4.3 Spray and streaked foam from a gale up. Verify: a storm capture beside `sea-storm-day.jpg`. (Built 2026-11-15; no gale was found to film, so only a test capture with the threshold lowered exists, `spray-test-from-1mps.jpg`; the storm capture is still owed.)

## 5. Tides (requested 2026-11-15, after group 7, on the 1 mm condition)

- [x] 5.1 (Built 2026-11-15 on `claude/env-water`: `pbd_core::sea::tide_m`, the moon's second Legendre term scaled to `tide_range_m` (3 m, `water.ron`), in `LocalSea` and `sea.wgsl` alike, so hulls, the line and the camera float on it; the sheet listed over every cell under high water so no lip stands at a shore, the ground's tint following the tide at the camera. The sea parity test now runs on Metal too, the tide on, 360 points within 1 mm; `the_tide_ranges_twice_a_turn_and_keeps_the_seas_volume`; `docs/screenshots/water-flow/tide-high-beach.jpg`, `tide-low-beach.jpg`. Not done: the walker's swim still reads the column's static sea level, so a walker on a beach under high water is not swimming.) (Was: not built 2026-11-15, design B7.) The tide as an offset of the sea function, from the body's moon. Verify: the shader-against-core test still agrees within 1 mm with the tide on; a core test of the range and period.

## 6. Places in the generator (requested 2026-11-15, if time remains)

- [x] 6.0 (built 2026-11-15, `claude/env-life`; design B10, B12) The sea's yearly normals (`examples/sea_normals.rs`, `assets/climate/sea-normals-g7.bin`) and the plate field (`pbd_core::ocean::plates`). Verify: core tests that the normals load and are finite and that both kinds of plate boundary lie under the sea.
- [x] 6.1 (Built 2026-11-15 on `claude/env-life`: the function, read by the generator (the trench, the pieces), the spawner and the map's "Sea places" layer, B13.) `ocean::environment` in `pbd-core`, read by the generator, the spawner and the map. Verify: a test that the three agree on a sample of directions; the map's chart layer.
- [ ] 6.2 (Built 2026-11-15 as generator version 7: the trench carved, reef, kelp, vents and floes as PIECES; versions 6 and 7 pinned by digest (taken on the Mac). Open: reef, ice, basalt and mussel-bed CELLS wait for the column's material word to widen (B10); the pieces have no dither fade yet (B12).) Reef, kelp, vent, trench and sea-ice cells and pieces in a new generator version. Verify: the old version's digest test unchanged; a new digest test for the new version; an old save opens on its own version.

## 7. Water life (requested 2026-11-15)

- [ ] 7.1 (First cut built 2026-11-15: seven appended, `shore_crab`, `lobster`, `giant_clam`, `reef_turtle`, `sea_otter`, `reed_dabbler`, `pond_diver`, with `env` tags, the crawler and the floater, their own spawn cap `max_life`; the append-only and mover tests pass. Spawning by environment built 2026-11-15 with 6.1, and three more species (7.5). Open: the other 28 species.) The new species appended to `fauna.ron` with `env:` tags; the bed crawler and the surface floater. Verify: the append-only test; spawn tests per environment.
- [x] 7.2 (Clams and ducks built 2026-11-15; the octopus, plume worm and seal with their species, 7.5.) Behaviours: clams, the octopus, tube worms, seals, ducks. Verify: core tests of each state machine against the eye's distance.
- [x] 7.4 (built 2026-11-15; captures in `docs/screenshots/ocean-life/`; design B8) The mockup's 3D bodies for every water creature, fish included, posed and animated on the CPU as the herd is; a caught fish stays its item sprite. Verify: a `--life <id>` capture of each in `docs/screenshots/ocean-life/`, judged beside the mockup's; an app test that every species in the roster has a body with faces; READY with no `LATE_PIPELINE`.
- [x] 7.5 (built 2026-11-15; design B9) The octopus, tube worms and seals appended, with `Hides` and `HaulsOut`. Verify: the append-only test; a core test of each state machine against the eye's distance.
- [x] 7.3 Protected species refuse capture. Verify: a test per species. (Built 2026-11-15: `catch_refusal`, `can_bite`; a float over one says why, once a cast.)

## 9. Textures and M9's fish (requested 2026-11-15: "reef needs some textures", "use the fish species mockup to create proper fish"; design B14)

- [x] 9.1 (Built 2026-11-15 on `claude/env-reef`; the skins checked pixel for pixel against M9's JavaScript.) The water-life atlas: reef tiles and M9's skins and fin tiles, drawn by `tools/water_life_atlas.py` with a manifest and provenance. Verify: the generator's `--check` holds the committed atlas to its source.
- [x] 9.2 (Built 2026-11-15: `every_piece_is_laid_into_a_reef_tile`; `docs/screenshots/ocean-life/*-textured.jpg` beside `*-untextured.jpg`, at a version-8 reef since version 8 lost the old one, B14.) UVs on the reef's pieces into their kinds' tiles, the texture on `LifeMaterial`, nearest. Verify: a test that every piece vertex has a UV inside its tile; READY with no `LATE_PIPELINE`; `*-textured.jpg` captures beside the untextured ones.
- [x] 9.3 (Built 2026-11-15: `every_fish_variant_wears_its_own_skin`; `lineup-seabed-textured.jpg`, `fish-close-textured.jpg`. M9 gains the darter and anthias and a "Reef & sea floor" tab of the game's own models, `docs/screenshots/mockup-fish/`.) The fish from M9: profile, tail, fins, eye, skin and seed variants; the ray's flap. Verify: a test that every fish model and variant builds with faces and UVs in its skin; the lineup and three close shots beside M9's sheet.
- [x] 9.5 (Built 2026-11-15, design B15: `version_9_has_version_7s_reefs_again`, `version_9_makes_the_ground_its_worlds_are_made_on`; `reef-v9.jpg`.) Generator version 9: version 8 with the reefs version 8 lost. Verify: version 9's reef share within a tenth of version 7's on the globe scan; 8's and 9's digests pinned.
- [x] 9.6 (Built 2026-11-15, design B15: the budget-and-skin test; `creatures-before-after.jpg`, `lineup-creatures-modelled.jpg`, M9's `sea-creatures.jpg`.) The other water creatures modelled in M9's manner with atlas skins. Verify: each under 500 triangles, every UV in a tile; captures beside the old bodies.
- [ ] 9.4 Video (batch).

## 8. Fresh water (after volumetric water phases 2 and 3)

- [x] 8.1 (Built 2026-11-15, generator version 8 on 7, design 13: `every_river_cell_drains_to_the_sea`, `a_rivers_water_sits_on_its_own_bed`, version 8's digest pinned and 7's unchanged; `docs/screenshots/water-flow/river-*.jpg`.) A drainage river pass that steps down from springs, in a new generator version. Verify: a test that every river cell drains to the sea; captures beside the river tab.
- [ ] 8.2 The beaver colony record and the `beaver` world process through the journal. Verify: a catch-up test to day 40; a test that a broken dam is not rebuilt on the player's cells and the colony moves on.
- [x] 8.3 (Marsh pools built 2026-11-15 in version 8: version 5's swamp water only where a rim holds it; `marsh-pools-day.jpg`. The estuary is the river's mouth into the sea; no estuary of its own.) Marsh pools and the estuary. Verify: captures beside the marsh tab.

## 11. Kelp that reads as kelp (design B16; owner, 2026-11-15: "kelp needs fixing")

- [x] 11.1 (Built 2026-11-15, cloud session, lavapipe: `a_kelp_stand_is_slender_and_in_the_frond_s_colours`; `docs/screenshots/ocean-environments/kelp-before-after.jpg` at the owner's spot, with `kelp-after-first.jpg` the take before the green was deepened. The rocks read pale cyan: grey rock under the water's tint. Frame cost not measured.) The kelp stand redrawn after M25's frond: slender crossed stipes, narrow spiralling blades with gas floats, a surface canopy, one colour a part in the frond's palette; the rocks at its foot in the rock's palette. Placement unchanged. Verify: a test of the stand's widths and colours; captures at `--walk --at 71.308 -8.196 --time 12` before and after.


## 12. Under a river's water (owner, 2026-11-16: "problems with submerging in rivers"; design B17)

- [x] 12.1 (Built 2026-11-16 on `claude/river-fixes`: `inland_surface`, `eye_water_surface` in `water.wgsl`; `an_eye_in_a_river_reads_as_under_the_river`; `docs/screenshots/river-fixes/{before,after}-{under,up,walk}.jpg`.) The water view carries the eye's own water surface (`sea_frame.y`, nought the sea); `compose`, `under_water_light` and the shafts measure depth, the waterline and the sky's exit against it; the swell only on the sea's. Verify: an app test that an eye in a river's column reads as under water against the river's surface; captures under a river before and after at one spot in `docs/screenshots/river-fixes/`.
- [x] 12.2 (Built 2026-11-16: `rebuild_shower` asks `inland_surface`. Not shown in a capture: the shots turn the shower off.) The near shower stops for an eye under any water. Verify: the same capture's rain-on test.
- [x] 12.3 (Already true: the walker reads the column, `water-flow` E10; pinned by `a_walker_swims_in_a_river_and_climbs_out`.) A walker in a river swims and climbs out. Verify: an app test of `swim_surface` and the walker's water state in a river column.
- [x] 12.4 (Built 2026-11-16, design B17 second pass: `WATERLINE_EDGE_M`; `WATER_SHORT_SHIFT`, `a_flowing_tops_surface_rides_its_record_to_the_eighth`; `before-skim.jpg`, `after-skim.jpg`, `after-bank.jpg`.) The waterline: the composite's mask and wobble only under the surface; the terrain's tint, darkening and caustics against the column's own water surface, a flowing level included. Verify: the packing test; captures at a river's surface and from the bank.
