# Tasks

After the roadmap, at the owner's word (survey T9). It needs `tenebris-towns`
tasks 1 and 3d (thin solids and surfaces in `stand`, and the ship piece).

## 0. The cog moored (design 6, step 1)

- [ ] 0.1 The export writes the cog (where it lies, its heading, its gangway side) and its gangplank; the ship piece is cut at its berth from the template, with its hull, deck, rail and gangway gap, castles, stair, mast, furled sail, shrouds, barrel and crate, and the gangplank as a ramp. Verify: core walker tests up the gangplank, across the deck and up the stair to the aftcastle; the village and walled town re-export byte-identical; shots beside the mockup's `cog` and `castle` views.

## 1. A turning frame

- [x] 1.1 `LocalFrame` with orientation and angular velocity, composing with the `ω × r` term. Verify: the `world/frames` scenarios as core tests (a point on a turning deck, out and back). (`frame::tests::a_point_on_a_turning_deck_moves_with_the_turn`, `a_position_composed_out_and_back_returns`; the requirement is in `openspec/specs/world/frames`.)

## 2. The cog as a craft

- [x] 2.1 `Kind::Cog` with its hull, square-sail, keel, rudder and seat specs in `vehicles.ron`, validated. Verify: settings tests refuse a hull cell over a third of the beam, and a sail with no yard. (`vehicle::tests::a_cogs_settings_refuse_a_coarse_hull_a_yardless_sail_and_a_backward_luff`, `the_cog_floats_on_its_waterline`, `vehicle_specs_match_the_ship_they_sail`.)
- [x] 2.2 The square sail: a foil on a yard braced round the mast. Verify: core tests that the yard's angle of attack follows the braces and the apparent wind, and that a scripted reach, beat and run log speed and heel, with the heel under 15° on the reach. (`the_cogs_yard_follows_its_braces`, `a_square_sail_luffs_with_the_wind_along_its_yard`, `the_cog_reaches_runs_and_cannot_point_high`; the luff and the measured polar are in design 6, step 3 part 1.)
- [ ] 2.3 The ship piece (`tenebris-towns` 3d) built in the craft's frame, and the craft drawn from the same model the harbour's cog is. Verify: a capture of the cog at its mooring matches the harbour's building cog.

## 3. Walking on the deck

- [x] 3.1 The ground query asks decks of craft in reach in their frames before the terrain. Verify: an app test that a walker set on the moored deck stands on it. (The walker asks the towns' pieces and `CraftDecks` together; `walking::tests::a_walker_stands_on_a_deck_through_a_quarter_turn` on the moored deck, `vehicles::tests::a_walker_rides_a_cog_under_way_through_a_turn` on the craft's.)
- [x] 3.2 The walker aboard keeps a local pose in the ship's frame, composed for drawing and the camera; leaving the deck hands it back to the planet's frame at the composed velocity, and climbing on hands it in. Verify: the `player/walking` scenarios as app tests (standing through a turn, up the stair under way, over the side). (`vehicles::tests::a_walker_rides_a_cog_under_way_through_a_turn`, `a_walker_climbs_a_cogs_stair_under_way`, `a_walker_goes_over_a_cogs_side_with_its_way`; the feet are what is kept, design 6 step 3 part 2. The requirement is in `openspec/specs/player/walking`.)

## 4. The helm

- [ ] 4.1 F at the tiller takes and lets go of the helm; W and S brace the yard, A and D steer. Verify: the `player/vehicles` scenarios as app tests (a reach; letting go of the helm), and the binding table lists the helm keys. (Built: the keys and the COG bindings group, `each_craft_maps_its_controls_and_menus_suppress_keys_and_look`; letting go, `letting_go_of_a_cogs_helm_leaves_the_walker_riding_its_aftcastle`. Open: the reach as an app test, which needs wind in the test app.)

## 5. Saves

- [ ] 5.1 The cog's record, and a walker saved on a deck with the craft's ID and local position, at `RECORD_VERSION` 2; a version-1 file reads with none. Verify: format tests, and an app test that quits on the aftcastle and loads there.
- [x] 5.2 A harbour's building cog becomes the craft at the same mooring (design 6: step 1's piece is never saved, so this is the handover in step 3). Verify: an app test on a world made before this change. (A world made before has no `(site, COG)` in its fleet, and `moor_harbours` makes the cog then, as for a new world: `towns::tests::every_harbours_cog_is_made_once_at_its_berth`; on its swing, cast off, made fast and saved: `vehicles::tests::a_harbours_cog_rides_its_swing_casts_off_and_makes_fast_again`, `a_moored_cog_is_saved_on_its_bollard_at_its_berth`; boarded up the town's gangplank: `settlement::tests::the_craft_cog_stands_at_its_berth_and_is_boarded_up_the_towns_gangplank`.)

## 6. The owner's check

- [ ] 6.1 A `sail` scenario in `tools/perf_suite.py`, and a note that frame cost was not measured in the cloud session. Verify: the scenario runs.
- [ ] 6.2 The gate video (`step-videos`, showcase `cog`), each shot beside the mockup's cog: up the gangplank; casting off and taking the helm; a reach with the walker then letting go and walking to the aftcastle while the ship sails and turns; over the side and swimming back. Verify: the gate page is linked from the PR.
- [ ] 6.3 The owner watches and accepts. Verify: the quote is in `proposal.md`. Sync the three specs, and archive.
