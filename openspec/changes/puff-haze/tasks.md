# Tasks

## 1. Write-up
- [x] 1.1 Reproduce the outline and find its cause (the haze off: no outline); proposal, design, spec delta.

## 2. Fix
- [ ] 2.1 `cloud_puff_vertex` measures `cloud_air` to each corner and passes it on; `cloud_puff_fragment` writes `(lit*a*air, a*air)` and keeps its discard on `a`.
- [ ] 2.2 The composite hazes the history only when the march drew it.
- [ ] 2.3 `shader_tests.rs` pins all three.

## 3. Verify
- [ ] 3.1 fmt, clippy, the tests; `openspec validate puff-haze`.
- [ ] 3.2 Before and after, same args and frame, north and south; crops and a difference image; `docs/screenshots/puff-haze/README.md` says where each was taken. No shader error in the log.
- [ ] 3.3 Frame time on real hardware (the owner's `perf_suite.py`; not run in this change).
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
- [ ] 3.5 Archive with `cloud-puffs`, whose requirements this one builds on and which are not in the main specs yet.
