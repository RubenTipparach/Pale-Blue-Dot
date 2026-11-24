# Tasks

Owner's request 2026-11-16 ("go build all the stuff you just said"); built the same day in a cloud container.

- [x] 1 `pbd_core::scatter::pieces`, the shader's pebble, bush, dead shrub and fern placement ported. Verify: core tests of the rolls against known ids; the GPU test against `planet_surface.wgsl` (design 2).
- [x] 2 The `clutter` record and regrowth. Verify: round trip; a piece taken is taken until three days pass.
- [x] 3 F aims at clutter and picks it up (design 1, 3), `How::Gathered` counted as `gather` and `find`. Verify: an app test picks a pebble and a bush; "Gather by hand" closes on F alone.
- [x] 4 The draw drops taken pieces and dissolves them out and in (design 5). Verify: captures before and after a pick-up; a test of the list's reach.
- [ ] 5 `PickUp` in the showcase and `phase-1-trunk.ron` (design 7). Verify: the parse test; the owner's run.
- [x] 6 Screenshots to the owner; the video waits for the batch.

Built 2026-11-16:
- 1: `pbd_core::scatter` (`hash`, `roll`, `grows`, `seat`, `pieces`); `planet_surface.wgsl`'s `clutter:shared` block (`clutter_grows`, `clutter_seat`, `grassy` and `bare` moved into it), which the vertex shader now calls. Tests: the core's `the_hash_is_the_shaders`, `each_piece_grows_at_its_chance_on_its_ground`, `a_seat_lies_on_the_cap_toward_a_corner`; the GPU test `clutter_gpu_tests::the_shader_and_the_port_put_the_clutter_in_the_same_places` (lavapipe: 1,625 pieces of 2,000 cells, the same decisions, seats at most 1 mm apart).
- 2: `scatter::Taken` (`CLUTTER_KIND` `clutter`, `REGROW_DAYS` 3, `sunk`). Test: `a_taken_piece_sinks_stays_gone_and_grows_back`.
- 3: `clutter::clutter_on_ray`, `FarmAct::Gather`, `farm::gather` (one `store_with_hands` line), `Act::Gather` ("Gather loose stone", "Gather plant fibre"), `How::Gathered` counted as `gather`; "Gather by hand"'s help line. Test: `f_gathers_a_stone_or_fibre_from_the_ground_clutter` (the offer, F, the pack, the record, not offered again).
- 4: `clutter::ClutterTaken` and `refresh`, the uniform's `clutter_taken`, `clutter_sunk` in the vertex shader. Test: `the_list_packs_two_cells_a_lane_with_a_byte_a_piece`; the GPU visibility test still passes with the larger uniform. `--gather-around <metres>` is a capture instrument. Captures: `docs/screenshots/gather-by-hand/` (before and after a 30 m take, and a crop of the bush gone).
- 5: `Act::PickUp(Until)` and `phase-1-trunk.ron`'s gather shot. Not run here (a showcase's real-time limits stall on lavapipe).
- Frame cost not measured (no GPU in a cloud container).

## 9. Fibre from felling (owner, 2026-11-21)

- [x] 9.1 Each log from a tree also gives one fibre when it fits (`take_log` in `desktop/digging.rs`), with its toast. Not tested in the game yet (no launch until the owner's play is done).

## 10. Taking a piece is 4x quicker (owner's playtest, 2026-11-22; `docs/plans/2026-11-22-playtest-todo.md`)

- [ ] 10.1 `scatter.taken_sink_s` 0.25 s, and the sink starting on the take's frame: coded; built, tested, seen.
