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
- [ ] 1.3 (Published 2026-11-14 at https://claude.ai/artifact/Ky4u9ybCSUm31t72NRMAA1 and on the tracker, per `mockup-work.md`; the survey questions, 1.4, are not yet put.) Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
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

## 13. Sea life that vanishes up close, floes on the beach (owner, 2026-11-16; design B18)

- [x] 13.1 (Built 2026-11-16 on `claude/sea-life-fixes`; `docs/screenshots/sea-life-fixes/`.) Floes only where they float at low water; none on beaches or terraces. Verify: an app test that no floe is laid over ground above its draft; captures at the polar shore before and after.
- [x] 13.2 (Built 2026-11-16 on `claude/sea-life-fixes`; `docs/screenshots/sea-life-fixes/`.) A floe seal slides off and swims away, and climbs back later; never skipped in one frame. Verify: an app test over the slide that it is drawn every frame.
- [x] 13.3 (Built 2026-11-16 on `claude/sea-life-fixes`; `docs/screenshots/sea-life-fixes/`.) A startled seal dives over time and swims away; a diving duck sinks and rises, never skipped. Verify: core tests of the `under` rate and the flee goal; the `--life seal` approach sequence 30 m to 2 m.
- [x] 13.4 (Built 2026-11-16 on `claude/sea-life-fixes`; `docs/screenshots/sea-life-fixes/`.) Ambient life fades with its tile (no 30 m cut); the sway eases out. Verify: captures; the tile-fade test.
- [x] 13.5 (Built 2026-11-16 on `claude/sea-life-fixes`; `docs/screenshots/sea-life-fixes/`.) Schools fade in when spawned and out before they are dropped. Verify: an app test of the fade; READY with no new `LATE_PIPELINE`.
- [ ] 12.5 (Built 2026-11-16 on `claude/river-fixes-2`, design B18. Validated by the waterline videos, before and after; `docs/screenshots/river-fixes/videos/`. Open: one riffle frame in the tributary video, listed in the README.) The river's water from inside, from the owner's spot. No straddle band, the eye measured against the drawn top, the surface overhead drawn from below, and the view under water seen through water. The sea's waterline: the eye over a tide-covered beach is in the sea, and the composite measures from the sea's tided surface. One surface at a river's mouth. Verify: `where_waters_meet_the_surfaces_join_and_the_eye_follows_them`, run on 5 mouths, 3 tributaries and 3 falls; the videos stepped frame by frame.

## 14. The waterline from just under, the river's ceiling, the speckle (owner, 2026-11-16; design B19)

- [x] 14.1 (Built 2026-11-16 on `claude/waterline-3`; `docs/screenshots/waterline-3/`.) The sheet seen from below mirrors the water outside Snell's window (the screen trace, the extra water on the way), the murk only where the trace finds nothing. Verify: the river and the sea seen from under, before and after, in the waterline videos.
- [x] 14.2 (Built 2026-11-16 on `claude/waterline-3`; `docs/screenshots/waterline-3/`.) The composite stands in for the sheet the near plane clips for an eye under the surface. Verify: the frames within 0.3 m of each crossing show no clear sky under the surface.
- [ ] 14.3 (Built 2026-11-16 on `claude/waterline-3`; not yet shown: the sea was calm on every shot, so no far crest stood over the eye. Open until a windy shore is filmed.) The sea's far sheet is seen from below only when the eye is under its own surface. Verify: the shore video's frames just over the surface show no dark band at the far waterline.
- [x] 14.4 (Built 2026-11-16 on `claude/waterline-3`; `docs/screenshots/waterline-3/`.) The shafts and the caustic web fade to their mean by their sampling footprint. Verify: the far end of the river channel without speckle in the after video.
- [x] 14.5 (Built 2026-11-16 on `claude/waterline-3`; `docs/screenshots/waterline-3/`.) Before and after videos at the owner's two spots, a sea shore and a river mouth, smooth crossings at 0.1 m/s, on Drive, with contact sheets and the stepped frame list in `docs/screenshots/waterline-3/`. Verify: the README's list.

## 15. Kelp that does not hammer the frame (owner, 2026-11-16: "kelp forest is HAMMERING PERF"; design B20)

- [x] 15.1 (Built 2026-11-16 on `claude/kelp-perf`.) Measurement instruments: `PBD_PASS_OFF=kelp` and the `SEA_PLACES` log line. Verify: the before exe prices kelp on and off in one sitting.
- [x] 15.2 (Built 2026-11-16 on `claude/kelp-perf`: `a_kelp_stand_is_the_mockup_s_few_cards`, `kelp_stands_at_the_mockup_s_density`; `docs/screenshots/kelp-perf/`.) The kelp stand is M25's: two crossed frond cards a 4 m of water and one flat on the surface, in M25's own card, cut out; at M25's density; its own mesh, frustum culled, its pipeline warmed at load. Verify: the tests; before and after stills from just above the surface and on the bed; the held-view sitting in `docs/benchmarks/2026-11-16-kelp/`.

## 16. No mirror on the water's underside; the split frame (owner, 2026-11-16; design B21)

- [x] 16.1 (Built 2026-11-16 on `claude/underwater-waterline`; `docs/screenshots/underwater-waterline/`.) The sheet seen from below is Snell's window and the murk, the waves and rings a gentle shading on the murk (at most 12% either way); no mirror and no screen-space trace, at any REFLECTIONS level; the composite's stand-in for the clipped sheet the same. Verify: `the_underside_of_the_water_never_mirrors`; the owner's two spots before and after in the videos.
- [x] 16.2 (Built 2026-11-16 on `claude/underwater-waterline`; the stepped frames in `docs/screenshots/underwater-waterline/README.md`.) The split frame at the owner's shallow-sea spot is gone: the frames within 0.3 m of every crossing show no mirrored sea floor and no streak band. Verify: the stepped frame list in `docs/screenshots/underwater-waterline/README.md`.
- [x] 16.4 (Built 2026-11-16 on `claude/underwater-waterline`; `jump-beach-*` in `docs/screenshots/underwater-waterline/`.) A jump on a dry beach at low tide tints and wobbles nothing: the composite's waterline is the sea's tided surface for an eye in air as for one over the sea's water. Verify: `a_jump_over_a_beach_keeps_the_composites_waterline_at_the_tide`; the `jump-beach` video before and after, stepped.
- [ ] 16.3 (Owed: the Mac was not quiet on 2026-11-16.) The frame cost of the change, under the water. Verify: a held under-water view, `--frame-log`, release build, before and after interleaved on a quiet machine.

## 17. Holes in the sea at caverns beside it (owner, 2026-11-16; design B22)

- [x] 17.1 (Built 2026-11-16 on `claude/sea-cavern-holes`; `docs/screenshots/sea-cavern-holes/`.) The sea's reach: a walk from the open sea through every open cell up to the high water's layer, at the build and after an edit, marks each reached finest record. Verify: an app test that a pit behind a beach and a cavern under a bank are reached and an inland pit is not.
- [x] 17.2 (Built 2026-11-16 on `claude/sea-cavern-holes`; `docs/screenshots/sea-cavern-holes/`.) The sheet is listed over every reached cell and the mesher leaves it the sea's top there; one surface in a sea cave. Verify: an app test over the owner's spot that no cell draws two surfaces and no reached cell draws none.
- [x] 17.3 (Built 2026-11-16 on `claude/sea-cavern-holes`; `docs/screenshots/sea-cavern-holes/`.) The sheet under a roof takes the cave's sky light from its record. Verify: captures in a sea cave before and after.
- [x] 17.4 (Built 2026-11-16 on `claude/sea-cavern-holes`; `docs/screenshots/sea-cavern-holes/`.) The eye and the swimmer read the reach. Verify: an app test of `eye_water_at` in a reached pit and an unreached one.
- [x] 17.5 (Built 2026-11-16 on `claude/sea-cavern-holes`; `docs/screenshots/sea-cavern-holes/`.) Before and after videos at the owner's spot and two other sea caves, on Drive, stepped frame by frame. Verify: `docs/screenshots/sea-cavern-holes/README.md`.

## 18. Fish under the water at full brightness (owner, 2026-11-17; design B23)

- [x] 18.1 (Built 2026-11-17 on `claude/underwater-fish-light`; `docs/screenshots/underwater-fish-light/`.) One rule for what water does to a colour: `pbd_core::ocean::under_water_albedo`, the terrain's `att * mix(colour, deep, (1 - mean(att)) * 0.5)`; every sea-life colour (schools, reef pieces, kelp, ambient anthias, clownfish and seals) goes through it. Verify: a core test of the rule and an app test that holds it to `planet_surface.wgsl`.
- [ ] 18.2 (Built 2026-11-17 on `claude/underwater-fish-light`; not yet shown: the perch frame is at a sea shore, and no school in a lake or river over sea level was filmed.) A school's depth is read from the surface of the water it is in (a lake's or river's top, the sea's tided surface), `PlanetWater::surface` at its anchor. Verify: before and after of a school under a lake's surface.
- [x] 18.3 (Built 2026-11-17 on `claude/underwater-fish-light`; `docs/screenshots/underwater-fish-light/`.) Sea life is lit as the terrain is (`LitLikeTerrain`): the terrain's fill, sun after the air, cascades and lamps. Verify: READY with no new `LATE_PIPELINE`; the night frame with a torch.
- [x] 18.4 (Built 2026-11-17 on `claude/underwater-fish-light`; `docs/screenshots/underwater-fish-light/`.) Before and after stills of the same frames (near the surface, about 9 m down, from above, at night with a torch) in `docs/screenshots/underwater-fish-light/`. Verify: the README.
