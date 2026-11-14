# Tasks

## 1. Write-up
- [x] 1.1 Reproduce from 9 km and 6 km up; measure the opacity, the haze from above and the size spread (proposal, design 1-3).

## 2. Fix
- [ ] 2.1 `cloud_puff_optical_depth` 15 -> 4 (`config.rs` default and `weather.ron`).
- [ ] 2.2 The puff fragment scales its haze by the air behind it where the ray stops on a surface (design 2).
- [ ] 2.3 Sites: the clump field, ranked thresholds and sizes, radius factor `0.5 + 1.3 u^2`, nudge half the spacing (design 3).
- [ ] 2.4 Tests (design 5), and `puffs_follow_the_map_and_come_far_to_near` counts the puffs built.

## 3. Verify
- [ ] 3.1 fmt, clippy, the tests; `openspec validate space-clouds`.
- [ ] 3.2 Before and after stills, same arguments: straight down and toward the limb from 9 km by day and dusk, a storm from 6 km, and the ground and 900 m shots `puff-haze` used, in `docs/screenshots/space-clouds/` with a README; the puff counts before and after.
- [ ] 3.3 Frame time on real hardware (the owner's `perf_suite.py`; not run in this change).
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
