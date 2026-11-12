## 1. Core

- [x] 1.1 `pbd_core::frustum`: planes from a reverse-Z infinite clip matrix, conservative box test, tests
- [x] 1.2 `cloud_puffs::bound_radius`, with a test that every billboard corner lies inside it

## 2. App

- [x] 2.1 `PuffSites::build` filters by the frustum after the horizon, before the sort; returns the counts
- [x] 2.2 `planet_water.rs` passes `clip_from_body`
- [x] 2.3 Frame log: puffs built, past the horizon, past the view, build CPU ms

## 3. Measure and show

- [x] 3.1 Before/after sitting on `clouds` and `storm`, report in `docs/benchmarks/`
- [x] 3.2 The same frame before and after, diffed, in `docs/screenshots/puff-frustum-cull/`
