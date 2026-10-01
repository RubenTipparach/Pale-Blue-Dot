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
- [x] 0.10 (`settlement::pieces` `Surface`, `DoorLeaf`; `settlement::record` door records; `walking.rs` `footprint_in`, `sweep`, `resolve_ground`; `towns.rs` `use_doors`, `swing_doors`, `door_states`; `--open-doors`, `--up`. The tests named in the design's 2b "As built"; `docs/screenshots/cities-in-the-world/game-2b-*.png` beside `mockup-2b-*.png`.) Slice 2b, the rest of the walker's rules (design decision 9): floors and stairs as surfaces, the newel stairs and straight flights cut from each building's stair cells, the pitch line, the 0.35 m hold, sliding, and doors that E opens and shuts and the save keeps (`door` records). Verify: core tests of each stair's pitch line and underside, the stair wells and the door leaf; app tests on Holbrook's houses up and down a newel and a flight (no eye jump over 0.1 m, no tick in the air coming down), a slide along a wall, a door that stops and then lets the walker in, and a door opened, saved and found open again; captures beside the mockup's views.

### Slice 4: every kind (design, "Slice 4")

The owner, 2026-09-30: "alright once your tuning is done, begin working on other towns". Each sub-slice ends in shots of the town in the game beside the mockup's same views. The order is a recommendation taken (ask only with screenshots).

- [ ] 0.11 Slice 4a, every village stands: every village site is laid and stored on the world's first open and stands within 1.2 km, faded in and out, turned by its seed. It takes the first parts of tasks 2.2, 4.1, 4.3, 4.4 and 4.5, which stay open until each is whole. Verify:
  - core tests that every village site of the shipped seed lays and cuts at each of its six turns, and that the home village's layout is unchanged (`holbrooks_ground_is_pinned`);
  - a core test that a site with an edit in its footprint or margin is stored unsettled, and is not laid again;
  - app tests that a new world stores every village once and a second open writes nothing, that an old save gains its villages once, and that a town stands within 1.2 km and is dropped past 1.5 km;
  - the time of `column::surface_m` with no towns and with twenty, and a village's lay time in release, recorded in the design;
  - shots of two other villages at 11:00 and 22:30, and of one fading in on the walk between them.
  - **Built (2026-10-01); open for three checks.** Done: `the_village_lays_and_cuts_at_each_of_its_six_turns`, `a_sites_turn_is_its_own_and_the_six_are_dealt_evenly`, `holbrooks_ground_is_pinned`, `an_edit_in_a_towns_ground_is_found_and_one_outside_is_not`, `an_unsettled_site_stays_unsettled`, `a_world_stores_every_village_once` (a second open writes nothing, byte for byte), `a_village_on_the_players_work_is_left_unsettled`, `a_town_stands_in_range_goes_past_it_and_fades_each_way`, `a_village_stands_as_the_walker_comes_and_is_taken_down_as_it_leaves`; the height's cost with twenty villages measured and brought to noise by the grid (design, "4a as built"); Holmouth and Dunmouth at 11:00 and 22:30 (`docs/screenshots/cities-in-the-world`, slice 4a to 4c). Open: an old save's own test (it takes the same path as a new world's), the release lay time (a cloud session has no fair release timing; debug is 0.30 s), and a shot of a village fading in.
- [ ] 0.12 Slice 4b, the walled town's streets and houses: `town.json` exported; heights in metres and footprints from areas; each footprint cell at its own level (record schema 2); street steps; its 29 buildings. Verify:
  - core tests that the town template loads, that the village's footprint is unchanged by reading areas, that a schema 1 record reads as level 0, that each cell stands at its template level, and that every level change of a layer has its step;
  - the town's layout checks;
  - shots of its lane, square and market, and from 60 m.
  - **Built (2026-09-30); open for steps and two shots.** Done: `town.json` exported with `terraced`; levels stored in schema 2 and a flat town in schema 1 (`a_terraced_town_is_stored_in_schema_2_and_a_flat_one_in_schema_1`), each built cell at its level (`the_walled_town_lays_on_its_levels`), every walled site laid and cut (`every_walled_town_lays_and_cuts_on_its_levels`); Linenleigh over the roofs and from 60 m. Left from the plan, as the design's "4b as built" says: heights stay whole metres (the town's are), the footprint is read as before (areas were not needed), and there are no step pieces yet (a 1 m change is the terrain's own step). Open: the step pieces and their test, and shots of the square and market from the street.
- [ ] 0.13 Slice 4c, the walled town's masonry (design, "4c in detail"): the curtain wall as masonry prisms from the template with its wall walk and merlons, and the gate passages; then the two stair towers and the keep. Verify:
  - core tests that every wall cell is cut, that its top is a surface at `WALL_TOP` and its sides are solid, that a gate's passage is open to 4 m, and that merlons stand only on outward edges;
  - a walker test through a gate and, with the towers, up a newel onto the wall walk;
  - shots of the wall from 60 m, the north gate from the road, and the walk.
  - **Built (2026-10-01); open for the walker test and two shots.** Done: `the_walled_towns_wall_stands_and_its_gates_open` (every wall cell cut, its top a floor at the walk, its sides solid, the gates open to 4 m, merlons only outward), `a_stair_tower_climbs_to_the_walk_and_the_keep_to_its_roof`; the wall, towers and keep from 60 m by day, at dusk and at night. `a_walker_goes_through_a_gate_and_up_a_tower_onto_the_walk` (2026-10-01) walks the walker's own queries (`holds`, `stand`, a body 1.8 m by 0.3 m) across every wall cell (the 4 gate cells let it by, the rest stop it), from every wall cell to each beside it on the walk, and up each tower's newel and out of its doorway onto the walk. It found the seam between cells' frames (design, "The seam on the walk"), now bridged. Open: shots of a gate from the road and of the walk.
- [ ] 0.14 Slice 4d, the harbour (design, "4d in detail"): `coast.json` exported; the whitewash and driftwood kits; the harbour standing on the sea (terrace at sea level, its levels the template's, no cell under its sea laid); its turn and shift chosen from the ground; open-sided boathouses, piers on piles, stilted fish huts with decks and porch stairs, the mole's light, lanterns over the water. Verify:
  - core tests that the kits load and a saved town can name them, that no cell under the template's sea is in the footprint and none over the water is laid, that the quay is 1 m over the water, and that on the shipped seed every harbour's placement puts three quarters of its sea over the planet's;
  - core walker tests along the main pier to its head, into a boathouse from the sea, and up a fish hut's porch stair onto its deck and in at its door;
  - an app test that every harbour lays and cuts and is stored once, and the placement scan's time recorded in the design;
  - shots beside the mockup's: the harbour from the pier head, from 60 m by day and at night, a fish hut and a boathouse.
- [ ] 0.15 Slice 4e, the desert: the sandstone and adobe kits, walkable flat roofs with parapets and their gaps, domes, outdoor stairs, the oasis. Verify: the walker up an outdoor stair onto a roof, and shots.
- [ ] 0.16 Slice 4f, the mountain: the alpine kit, its terraces, switchback stairs, and the rock-cut rooms as the first hollows (task 3.1a). Verify: 3.1a's tests on the cliff's rooms, and shots.
- [ ] 0.17 Slice 4g, the tundra: the granite longhouse, igloos with their tunnels, the ice keep and wall. Verify: the walker into an igloo through its tunnel, and shots.
- [ ] 0.18 Slice 4h, the swamp: the alder kit, stilt floors on piles, decks, porch stairs, boardwalks and the jetty over the bayou. Verify: a walk along the boardwalk into a stilt house, and shots.
- [ ] 0.19 Slice 4i, the jungle: kapok trunks, platforms 9 m up, rope bridges and the pole tower. Verify: a walk up the tower and across a bridge, and shots.
- [ ] 0.20 Slice 4j, the caves: the chamber, tunnel, carved rooms and shaft (the rest of task 3.1a), with lights that burn all day. Verify: 3.1a's tests on the chamber, and shots from the portal and the ledge.
- [ ] 0.21 Slice 4k, the mounds: turf domes, round doors, the vaulted back room, and a share of village sites taking the mound template by seed. Verify: a stored village keeps its template, and shots.

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

- [x] 5.0 Rooms lit by their own fires (design 7a): a hearth in the chimney cell, sconces on the stairs, and candles behind 55% of the windows. Each building's lights light its own rooms, storey by storey, the mockup's way. Verify:
  - core tests that every village house with a chimney has its hearth, every stair its sconces, and about half the windows a candle, all inside their building;
  - an app test that a building's rooms carry its lights;
  - captures at 11:00 and 22:30 beside the mockup's.
  - As built: `settlement::tests::every_house_has_its_hearth_its_sconces_and_its_candles`; `towns::tests::holbrooks_rooms_take_their_share_of_the_sky_and_the_town_casts` (the lights packed into a room's material, fires first); `docs/screenshots/sun-shadows/newel.jpg` and `flight.jpg` at five hours beside the mockup's.
- [ ] 5.1 (The candles are built in 5.0, as a room's light rather than a material; what is left here is the glow into the street.) `WindowCandle`, a dusk-lit material, placed behind about half the windows by the site's seed. Verify: core tests of its emission and dusk flag, and a test that a lit window's room is lit and its outer wall face is not.
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
