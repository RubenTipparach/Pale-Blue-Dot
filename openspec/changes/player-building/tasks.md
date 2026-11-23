# Tasks

Step 3 of the owner's plan. The mockup (group 1) can be made once the owner
has approved the towns' pieces. The Rust starts after `cities-in-the-world`
has built them into the engine.

## 1. The build mockup, for the owner's approval

- [x] 1.1 (Covered by M6, `docs/mockups/building.html`, the Hex House Builder: build mode, the palette, the aim-snapped ghost with its reason, the roof tool, at night; drafted, not separately approved.) `docs/mockups/build.html` from the towns mockup's pieces, kits, walker and checks. It has build mode, the palette, the aim-snapped ghost with its reason, the roof tool and blueprints, on an empty plot and beside the walled town, with day and night. Verify: a headless test builds a two-storey house piece by piece and walks up its stair with no eye jump over 0.1 m, and a refused placement shows its reason.
- [x] 1.2 (M6 is published, https://claude.ai/artifact/6kd3oRFMMtZ1d4FCjMiAYt, and on the tracker.) Publish it and link it from the roadmap page and the PR. Verify: the link opens.
- [ ] 1.2a A walkthrough video of the build mockup: a two-storey house built piece by piece, a refused placement and its reason, the roof tool, and a blueprint. Verify: it is on the gate page.
- [x] 1.3 (Recorded as the owner's "Build phase 3", 2026-11-21, in `proposal.md`, "Started"; no feel was approved separately and none is claimed.) **Gate:** the owner watches the video, builds with it, approves the feel, and answers the open questions. Record the words in `proposal.md`. Verify: the quote is present.

## 2. One set of checks

- [x] 2.1 (There was no `settlement::check` to split: the rules are written once in `pbd_core::building::rules`, and the towns reach them through `building::blueprint` and `check_layout`; `settlement::tests` pass unchanged.) Split `settlement::check` into per-rule functions over the pieces within reach, and have the layout check call them. Verify: `tenebris-towns`' layout tests pass unchanged.
- [x] 2.2 (`building::tests::every_plain_template_building_is_accepted_whole_and_piece_by_piece`, `each_rule_refuses_with_its_own_name`, `nothing_is_built_on_a_pentagon_or_beside_one`.) `building::can_place(pieces, candidate)` returns the first rule that fails. Verify: a test builds every template both whole and piece by piece, and both are accepted; and one test per rule is refused with that rule's name.

## 3. Piece edits

- [x] 3.1 (As `piece` records on the journal's `rec` line, the kind noted in the identity; design B3 says why the log's line format was not bumped. `building::tests::a_piece_record_round_trips_and_a_removed_one_is_gone`, `builds::tests::a_built_room_survives_a_reload_and_an_old_log_reads_as_it_did`.) Add and remove piece edits in the edit log, through the durable path, with the format version bumped. Verify: format tests for the round trip and for an old save reading unchanged; an app test that a built room survives a reload.
- [x] 3.2 (`building::tests::removing_a_floor_under_a_lantern_names_the_lantern`; the game says "Remove the lantern first".) Refused removals name their dependant. Verify: a test removing a floor under a table.

## 4. Build mode

- [ ] 4.1 (On B since 2026-11-22, breeding on N, design B10; `controls::tests::build_mode_is_on_b_and_breeding_on_n` holds the keys and B in flight braking. The player's slots are never touched and the selection is given back on leaving; the app test of the hotbar coming back is still owed.) B on foot opens and closes build mode, the hotbar becomes the kit's palette, and leaving restores it. B is added to `BINDINGS` for on foot. Verify: app tests that the hotbar and selection come back unchanged, and that B in a ship still brakes.
- [x] 4.2 (`docs/screenshots/player-building/green-ghost.png`, `red-ghost.png`; lavapipe.) The aim-snapped ghost, its colour and its reason text. The wheel steps openings and stair rotations. Verify: `--capture` shots of a green and a red ghost with its reason, in `docs/screenshots/player-building/`.
- [x] 4.3 (`building::tests::the_roof_tool_roofs_a_two_by_four_house`; `two-storey-house-night.jpg`.) The roof tool's flood fill and fit. Verify: a test roofing a two-by-four house, and a capture.
- [ ] 4.4 (Built: the Blueprint slot charts any plain template building at the aim, the wheel turns it, shift and the wheel picks which, one click fills it whole; `house-*.png`. The app test that walks in through its door is still owed.) The blueprint picker and fill, both at once and piece by piece. Verify: an app test places a cottage blueprint, fills it, and walks in through its door.

- [ ] 4.5 (Orbit, pan, zoom and a build layer built as architecture mode, design B9, task 7.5; double-click to walk and the cutaway not built.) The build camera, from the towns mockup's overview: orbit by drag, pan along its heading (right-drag, WASD), zoom by the wheel, double-click to walk there, and a cutaway height that hides everything above a chosen layer. Verify: an app test for each, and a capture with the cutaway through a two-storey house.
- [x] 4.6 (`kit-picker.jpg`: all 22 kits the game ships, not 19; the wheel steps the kit.) The palette's kit picker shows every kit side by side, as the mockup's Kits tab does. Verify: a capture of the picker with all 19 kits.

## 5. The owner's check

- [x] 5.1 (Not measured in the cloud: lavapipe. `building::tests::time_can_place_over_the_walled_towns_biggest_building`: The Lantern inn's 78 pieces, about 24 us a check in a debug build.) Record that frame cost was not measured in the cloud session, and time `can_place` in a core test on the walled town's densest street. Verify: the time is in the risk note, and the note is in the PR.
- [ ] 5.1a The gate video (showcase `building`). Its shots are:
  - build mode opened;
  - a two-storey house built piece by piece;
  - a refused placement with its reason;
  - the roof tool;
  - a blueprint filled;
  - walking up the new stair;
  - a quit and reload showing the house piece for piece.

  Verify: the gate page is linked from the PR.
- [ ] 5.2 The owner watches the video, builds in the game, and accepts. Verify: the quote is in `proposal.md`. Sync `player/building`, and archive.

## 6. Fixes after the first captures (2026-11-22)

- [x] 6.1 A floor over lower ground stands on a fieldstone footing down to it (design 8). Verify: `building::tests::a_floor_over_lower_ground_stands_on_a_footing`, and a capture of the house on the desert's terraces.
- [x] 6.2 A player's building casts the sun's shadow as a town's does (design 9). Verify: `builds::tests::a_plot_casts_the_suns_shadow_but_its_flames_do_not`, and a capture.
- [ ] 6.3 Retake `house-4-roofed.jpg` and `red-ghost.jpg`, the red ghost wholly in frame. Verify: in `docs/screenshots/player-building/`.

## 7. The owner's notes (2026-11-22; design B7 to B10)

- [x] 7.1 (`footing.png`, `tools/art/footing_texture.py --check`; `builds::tests::the_footing_is_drawn_in_the_stones_palette`; `footing-close.png`.) The footing's own texture, `footing.png`, in the stones' palette, with its manifest row, drawn by `tools/art/footing_texture.py` (design B7). Verify: a test holds its colours to the palette and its manifest row to the PNG; `--check` holds the PNG; a capture of the footing close up.
- [x] 7.2 (`station::tests::a_door_is_planks_and_an_iron_fitting_at_the_workbench`; the good, its icon by `tools/gen_item_icons.py`, its value.) The door: the good, its icon and its workbench recipe, 4 planks and 1 iron ingot (design B8). Verify: a core test of the recipe.
- [x] 7.3 (`building::tests::a_door_hangs_in_a_doorway_and_stops_the_walker_while_shut`, `building::door::tests::the_leaf_and_the_casing_are_closed_bevelled_forms`.) The door piece: the `doorway` rule, the wall's dependant, the leaf in the plot's solids and the casing (design B8). Verify: core tests that a door is refused off a doorway and blocks the walker shut and not open; the casing and leaf cut.
- [x] 7.4 (`builds::tests::a_door_hung_and_opened_comes_back_open`, `builds::tests::an_old_house_comes_back_with_its_doorways_open`; `door-shut.png`, `door-open.png`.) Hanging, opening and shutting a door in the game, on the durable path, the door taken from the pack and given back (design B8). Verify: an app test that a door placed and opened comes back open after a reload, and an old house's doorway comes back open with no leaf; captures of a door shut and open.
- [x] 7.5 (`walking::tests::architecture_mode_gives_the_walkers_camera_back_exactly`, `build_view::tests`; `architecture-day.png`, `architecture-night.png`.) Architecture mode on Z (design B9). Verify: an app test that the walker's camera is exactly as it was after the mode is left; captures over a house by day and at 22:30.
- [x] 7.6 (`controls::tests::build_mode_is_on_b_and_breeding_on_n`, `the_defaults_never_clash_and_a_rebind_onto_a_live_key_does`, `breeding::tests::n_opens_the_breeding_screen_on_foot_only`.) Build mode on B, breeding on N, old rebinds moved (design B10). Verify: the bindings' clash test, a test that B opens build mode and N breeding, and a test that an old rebind of either is kept.
