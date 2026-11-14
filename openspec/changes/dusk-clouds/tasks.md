# Tasks

## 1. Write-up
- [x] 1.1 Reproduce from the ground at dusk; measure the cloud's sun and sky against the sky's own (proposal, design 1-2), and the stars' leak through the haze (proposal).

## 2. Fix
- [ ] 2.1 `pbd_core::sky_light::cloud_sky` and `cloud_sun`, with their tests (design 1, 2, 4).
- [ ] 2.2 `clouds.wgsl`: the same two functions; `cloud_light` and `cloud_sun_sky` use them.
- [ ] 2.3 `cloud_puff_fragment` uses them; the puff instance carries its site's cover.
- [ ] 2.4 `cloud_night_floor` 0.045 -> 0.008.
- [ ] 2.5 The composite hides the stars under the puffs (design 3).
- [ ] 2.6 `shader_tests` (design 4).

## 3. Verify
- [ ] 3.1 fmt, clippy, the tests; `openspec validate dusk-clouds`.
- [ ] 3.2 Before and after stills from the ground at dusk and at night with cloud overhead, and by day to show the day is kept, in `docs/screenshots/dusk-clouds/` with a README.
- [ ] 3.3 Frame time on real hardware (the owner's `perf_suite.py`; not run in this change).
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
