# Tasks

- [x] 1.1 The 1 km walk from the owner's river, release, frame log: landings are the hitches (proposal).
- [x] 1.2 Trace build, same walk: the landing's main-thread and render-thread spans ranked (design 2).
- [x] 2.1 `play-soak.ron`: a regular playthrough from the owner's river into a town.
- [ ] 2.2 `perf_suite.py` `play` scenario, and a first run of it.
- [x] 3.1 The fix, written in design first (design 4: the render origin at 128 m with hysteresis).
- [ ] 3.2 The walk and `play` before and after, same sitting.

## 4. Every block edit is a 100 ms frame (design 6)

- [x] 4.1 Found on the overnight trunk: every dig 100 to 123 ms, the main thread.
- [x] 4.2 The edit's own split (clone, apply, relight, contact, tier image), measured.
- [x] 4.3 The fix for what the split names, written here first (design 7: the relight off the main thread).
- [x] 4.4 The trunk before and after, same sitting (design 7.1): a change 105 to 30 ms. Not yet under 16.7 ms.
- [ ] 4.5 The contact kept across an edit (its grid and records), and the tier image: the 30 ms left.
