# Tasks

- [x] 1.1 `desktop/showcase/path.rs`: the search over `(cell, layer)` (design 1, 2), on a ground trait the game and the tests both give.
- [x] 1.2 Unit tests: flat, pit, wall, drop, station, deep water (design 4).
- [ ] 2.1 The bot follows plans (design 3); `dig_through`, `Through` and `blocked_here` removed.
- [ ] 2.2 In the game: the paused fresh-water shot on `tree-run-5` reaches the river and fishes it.
- [ ] 2.3 The rest of the tree (three crops, farm meal, feast, reef, town, boat) played on it.
- [ ] 3.1 Building out (design 5): pillar, step and bridge moves with their places, the follower placing them, tests (out of a pit taller than a stair, over a gap).
- [ ] 3.2 `dig_through`, `Through` and `blocked_here` removed once the plan has carried a whole branch (they are the fallback when no plan is found).
- [x] 4.4 Long walks over the surface (design 9).
- [x] 4.3 Fields not dug through (design 8).
- [x] 4.2 Up and out from the middle of the cell (design 7).
- [x] 4.1 A block between the walker and an out-of-reach dig is dug on the way (design 6).
- [ ] 3.3 Up and out first (design 5.1): dig overhead, jump, a block under the feet, to open sky; or a ladder up a wall.

## 14. The searches off the main thread (owner, 2026-11-23)

- [x] 14.1 The budget checked every 8 expansions, not 64; a failed surface route waits a second before the next.
- [ ] 14.2 `GroundSnapshot`: what `path::plan` and `HexRoute::plan_where` read, copied once per request; the searches run on it with tests that a snapshot and the live ground give the same plan.
- [ ] 14.3 One search in flight on a worker thread; the bot walks its leg until the answer comes; stale answers dropped.
- [ ] 14.4 Measured on the farming shot before and after (frames over 50 ms, main thread p50, plans per run).
- [x] 14.5 `FineSet::finest_index_of`: the key-to-record index, built with the set; `record_of`, `fruit`, `tree_fx`, `tree_shake`, `beaver` and the bot's lookups use it.
- [ ] 14.6 Measured on the farming shot, 5 minutes before (42 fps, main thread p50 18.2 ms, 198 frames over 50 ms, the bot roaming stuck) and after (54 fps, 16.0 ms, 175 including the launch, the bot tilling and watering). Not a like-for-like pair: the bot did different work. A clean A/B (old exe and new, the same still scene) is still owed.
