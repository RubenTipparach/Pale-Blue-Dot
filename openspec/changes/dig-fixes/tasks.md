# Tasks

## 1. Tools that stop digging

- [x] 1.1 `tree_on_ray` skips a felled tree (`tree_standing`, the rule `under_standing_tree` reads).
- [x] 1.2 App test `desktop::digging::felled_tree_tests::the_ground_at_a_felled_tree_s_foot_digs`: through `dig_and_place` on the shipped ground, a standing tree takes the hold (one log, nothing dug) and the ground at a felled tree's foot digs. Run against the old hit-test (every tree counted as standing) it fails on "the ground where it stood digs": the reproduction.

## 2. Bait by the water

- [x] 2.1 `GatherSettings::by_water` and the new `bait`; `bait_reach_cells` and `bait_water_m` in `gather.ron`, validated; the shipped RON equals the defaults (checked against `config::tests`' rule; that lib test was not re-run in the cloud for want of disk, see 3.1).
- [x] 2.2 Core tests `gather::tests::by_the_water_is_a_few_cells_from_a_river_or_the_sea` and `earth_by_the_water_gives_bait_in_a_share_of_cells`: river inland, sea, flowing water, too far, a cliff over and a mine under the water, placed, rock, stone and snow, the share near a third.
- [x] 2.3 The dig asks `by_water` over the fine set's `finest_neighbors` and columns (a rim record answers from its adoptable column).
- [x] 2.4 App tests `bait_tests::earth_by_a_river_inland_gives_bait` (the river run's spawn, water 3 m and more over the sea) and `earth_by_the_sea_gives_bait` (the shore nearest the default spawn): every earth top beside the water is by it, some give bait and not all, none from a placed block, and none 30 m from any water.
- [x] 2.5 App test `bait_tests::three_baits_dug_meet_the_mission`: three single `Gained { bait, 1, Found }` count 1, 2, 3 on "Dig for bait"'s `dig bait`.
- [x] 2.6 `starter-missions` G1's text points here; the mission's help reads "Dig dirt by a river or the sea for bait worms".

## 3. Checks

- [x] 3.1 `cargo fmt`; pbd-core's lib tests (628 pass); the pbd-app desktop tests; `openspec validate --all`. Not run: pbd-app's lib tests (the cloud container's shared disk had under 1 GB left), and the perf suite (cloud, no GPU). The dig's added cost is one walk of at most 37 cells' columns per broken block.
