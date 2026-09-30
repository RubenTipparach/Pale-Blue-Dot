# Tasks

Step 2d, the last of step 2. It starts after the owner approves the lights,
the map, the sites, and the climate and fish layers, in that order.

## 0. Slice 1: Holbrook stands (design decision 9)

The owner, 2026-09-29: "I thought you were building cities, I wnat shots of
the cities". Built ahead of the groups below, which it takes the first parts
of; their own tasks stay open until each is whole.

- [x] 0.1 (`tools/export_town_textures.js`, `tools/export_town_templates.js`; `assets/textures/settlement/` with `manifest.ron`, `assets/settlements/v1/village.json`, `assets/config/kits.ron`. `settlement::tests::the_kits_and_the_village_load_and_every_building_has_its_kit`.) The mockup's textures, its village layout and its kits as data, exported from the approved mockup itself, not retyped.
- [x] 0.2 (`pbd_core::settlement::chart`. `settlement::tests::the_chart_keeps_neighbours_neighbours_and_uses_no_cell_twice`, `a_cells_sides_run_counter_clockwise_with_their_neighbours_across`.) The chart through the neighbour tables (decision 2), on a level-7 sphere at the gold standard's 2.833 m cells.
- [x] 0.3 (`pbd_core::settlement::ground`, asked by `column::surface_m` and `generate_solid`; the cleared bit, `planet_terrain::CLEARED_BIT`, read by `planet_visibility.wgsl`'s foliage pass; the map cache named by the ground's digest. `settlement::tests::the_ground_terraces_the_footprint_and_eases_the_margin`.) The terrace under the footprint, eased over its margin, with lanes of dirt and no trees.
- [x] 0.4 (`pbd_core::settlement::pieces`. `settlement::tests::every_village_building_cuts_into_its_pieces`, with every triangle facing out.) Walls on edges with their doors, shuttered windows and sills, corner posts, floors and beams, gable roofs with their gable ends, soffits and ridges, the huts' cones, and chimneys, cut from each cell's real corners.
- [x] 0.5 (`pbd_app::towns`, `--at <lat> <lon>`. `towns::tests::holbrook_is_laid_out_on_its_own_ground`; `docs/screenshots/cities-in-the-world/`.) Holbrook in the game: laid out when the world's sites are on disk, its ground installed and the planet rebuilt round the player, drawn with the mockup's textures and lit by the field.
- [ ] 0.6 The owner looks at the shots. Screenshots stand in for the video until the owner's batch.
- [ ] 0.7 Slice 2a, walls and doorways (design decision 9): walls, posts, chimneys and upper floors are solids; the walker stops at a wall, goes in at a door, and meets the floor above as a ceiling. Verify: `settlement::tests::every_village_building_cuts_into_its_pieces` (wall, doorway, air, yard, ceilings), `walking::tests::a_town_wall_stops_the_walker_and_its_doorway_lets_it_in`, and a capture from inside a house looking out of its door.
- [ ] 0.8 The owner's notes on the towns mockup (2026-09-29; screenshots in `docs/handoff/2026-09-29-owner-notes/`): water shows inside boat hulls; the stair block has inverted outside faces; windows should be double-sided, transparent and not glow; chimney bottoms z-fight; big doors with furniture behind them (the mound houses') should open outward. In the jungle town, the rail posts at a rope bridge's ends should stand on the platform hex's edge vertices; stairwells need more light. The igloo's geometry needs work: the tunnel's slab walls show bare faces, and its vault neither closes nor meets the dome (`igloo.png`). Write each up before code, in the mockup and the game alike. Verify: before-and-after shots of each.
- [x] 0.9 (`pbd_core::settlement::record`; `pbd_app::towns` `ensure`, `stand`. `settlement::tests::a_town_round_trips_through_its_records`, `a_town_built_from_its_record_is_the_town_its_template_lays`, `a_revised_template_leaves_a_made_town_as_it_was`, `a_damaged_settlement_record_is_named_not_remade`, `a_towns_records_are_small`, `every_kit_a_saved_town_can_name_is_shipped`; `towns::tests::a_world_stores_its_town_once_and_keeps_it_when_the_template_changes`, `a_damaged_settlement_is_neither_built_nor_written_over`, `holbrooks_ground_is_pinned`; `docs/screenshots/cities-in-the-world/retakes-2026-09-30.jpg`.) Slice 3a, the town is a stored record (design decision 9; task 4.5's storing half, ahead of 2b). `pbd_core::settlement::record`: `settlement` and `building` records, schema 1. The settlement holds its template's name, the terrace layer, every footprint cell's layout cell, key, side and top, and its buildings' ids. Each building holds its definition in layout cells, its floor over the terrace, and its state. The home village is stored once the sites are on disk, buildings first and the settlement last. It is built only once the writer's mark has passed, and always from the records. A damaged record is neither built nor overwritten. Verify: core tests that Holbrook's records round-trip; that the town built from them is the one laid from the template; that a changed template leaves a made town unchanged; the records' size; and Holbrook's ground digest pinned. App tests that a new world stores its town once, that a second open writes nothing, and that a damaged record builds no town. A capture of the lane from the records, beside the one from before.

## 1. The pieces: `tenebris-towns`, built

- [ ] 1.1 The owner answers `tenebris-towns`' open questions. Record the answers in its proposal. Verify: each question has an answer quoted.
- [ ] 1.2 `tenebris-towns` tasks 1 and 2: thin solids and surfaces in `stand`, and the walker's three rules. Verify: that change's tests, on synthetic stairs, with no eye jump over 0.1 m and no airborne tick.
- [ ] 1.3 `tenebris-towns` tasks 3, 3b, 3c and 3d: pieces, kits, settlement templates and the harbour, as data under `assets/settlements/v1/`. Verify: that change's layout checks pass on every template.
- [ ] 1.4 `tenebris-towns` tasks 4 and 5: drawing, light, furniture, trim and doors. Verify: that change's tests. Then sync its `world/settlements` and `player/walking` deltas and archive it.

## 2. Onto the sphere

- [ ] 2.1 The chart: axial to cells through the neighbour tables, with rotation and mirror. Verify: core tests on every site of the shipped seed that layout neighbours are sphere neighbours and no cell repeats.
- [ ] 2.2 Seeded variation with the layout checks and the fixed fallback order. Verify: tests that two village sites differ, that every variation built passes every check, and that the result is identical on one thread and many.

## 3. The ground

- [ ] 3.1 The settlement ground function (footprint, margin and cleared bit) and the column generator reading it. Verify: tests that terraces come out equal from inside-out and outside-in generation; that no step in the margin is over one layer; and that no tree or clutter is in a street or plot.
- [ ] 3.1a Hollows (survey T13): the ground function answers a hollow's floor and roof from the settlement record, and the column generator gives such a column its two solid runs. A cave town's chamber, tunnel, rooms and shaft and a cliff village's rock-cut rooms are hollows. Verify: tests that a cave town's chamber columns come out equal from inside-out and outside-in generation; that the chamber is empty between floor and roof with the mountain's rock above; that the shaft cell is open to the sky; and that reloading the world finds the same hollow with no journal entry written for it.
- [ ] 3.2 Sync the modified `planet/terrain` requirement with its new test. Verify: `openspec validate --all`.

## 4. In the world

- [ ] 4.0 The tier's relight with a city in it (follow-up from `lamps-and-lanterns` task 5.5). A synthetic 300-lantern city baked at 19.6 to 38.5 ms (median about 21 ms) in the cloud container, over the 12 ms that task set; the lanterns are 1 to 3 ms of it and the sky flood the rest. The owner measures the same test on real hardware, where the tier bake was 6 ms. If a city's bake is still over 12 ms there, the relight moves off the main thread or becomes incremental before a town is placed. Verify: the timing on the owner's machine, recorded here, and the decision it led to.
- [ ] 4.1 A settlement becomes an active chunk entity in range, with its pieces, contact index and meshes built on the pool and published whole. Verify: an app test walking from the spawn to a village site and in through a door, and `--capture` shots in `docs/screenshots/cities-in-the-world/`.
- [ ] 4.2 The far form and the night points, drawn by the planet pass. Verify: captures of a town from a kilometre by day, and of the night side from orbit.
- [ ] 4.2a The night lights light the coarse LOD hexes under each settlement (survey C3). Verify: a capture of the night side from orbit before and after, and a test that a hex under a lit town reads a glow and one outside its margin reads none.
- [ ] 4.2b Every harbour boat is usable (survey T7, "you can use any boat you find"). The small boats are the game's existing sailing and paddle craft, parked as vehicle records at their moorings. Verify: an app test boards a moored boat from the pier and paddles it away, and a reload finds it where it was left.
- [ ] 4.3 The fade between the three forms. Verify: the frame-by-frame no-pop check on a scripted fly-in at cruise speed.
- [ ] 4.4 Unsettled sites: a site with earlier edits is skipped and marked on the map. Verify: an app test with an old save holding an edit in a footprint.
- [ ] 4.5 Settlements and their building definitions as records in `world-persistence`'s store, generated for the whole planet at creation or on an old world's first open. Pieces are derived from the definitions, and abandoned buildings stay dark. Verify: tests that a revised template leaves a made world's towns unchanged, that the test process's abandonment darkens a house and survives a reload, and that an old save gains its records once.

## 5. Light

- [ ] 5.1 `WindowCandle`, a dusk-lit material, placed behind about half the windows by the site's seed. Verify: core tests of its emission and dusk flag, and a test that a lit window's room is lit and its outer wall face is not.
- [ ] 5.2 Street and wall lanterns placed by the template's spacing. The tier bakes the whole town. Verify: the bake time is recorded against `lamps-and-lanterns`' 300-lantern figure, and captures of a walled town at 22:30 from the street and from above.

## 6. The owner's check

- [ ] 6.1 A `town` scenario in `tools/perf_suite.py`, and a note that frame cost was not measured in the cloud session. Verify: the scenario runs in a smoke test, and the note is in the PR.
- [ ] 6.2 The gate video (`step-videos`, showcase `towns`). Its shots are:
  - a walk from the fields into each kind of settlement;
  - up an inn's stair and a keep's newel;
  - a door opened;
  - dusk falling on the walled town, with its lanterns and windows coming on;
  - a flight in from a kilometre, showing the fade;
  - the night side from orbit;
  - a house abandoned by the test process, going dark.

  It is published with stills beside the mockup's views. Verify: the gate page is linked from the PR.
- [ ] 6.2a Ask the owner to record the fly-in and the dusk shots in real time with `obs-record`, since a fixed-step video cannot show pop-in or hitches. Verify: the request is in the PR.
- [ ] 6.2b Walk `docs/plans/towns-mockup-parity.md`: every row is built, or agreed mockup-only in the survey. Each built row has a shot in a gate video beside the same shot in the prototype. Verify: no row reads planned or ask.
- [ ] 6.3 The owner watches both videos and accepts. Verify: the quote is in `proposal.md`. Sync `world/settlements`, and archive.
