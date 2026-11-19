# Tasks

## 1. Tools that stop digging

- [ ] 1.1 `tree_on_ray` skips a felled tree (`tree_standing`, the rule `under_standing_tree` reads).
- [ ] 1.2 App test: the ground at a felled tree's foot digs through `dig_and_place`.

## 2. Bait by the water

- [ ] 2.1 `GatherSettings::by_water` and the new `bait`; `bait_reach_cells` and `bait_water_m` in `gather.ron`, validated, the RON and the defaults held together by the existing test.
- [ ] 2.2 Core tests: river inland, sea, too far, too high or deep, placed, the share.
- [ ] 2.3 The dig asks `by_water` over the fine set's neighbour table and columns.
- [ ] 2.4 App tests on the shipped ground: by an inland river and by the sea, and not in the fields far from water.
- [ ] 2.5 Mission test: three single bait finds close `dig bait 3`.
- [ ] 2.6 Update `starter-missions` G1's text to point here.

## 3. Checks

- [ ] 3.1 `cargo fmt`, the pbd-core tests, the desktop tests named above, `openspec validate --all`. No perf suite (cloud); the dig's frame cost is one neighbour walk per broken block.
