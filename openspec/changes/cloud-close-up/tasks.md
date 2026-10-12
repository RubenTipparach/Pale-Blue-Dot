# Tasks

- [x] 1. 3D shape noise in `cloud_density`
- [x] 2. ~~Resolve pass with variance clip and disocclusion~~: built on the branch, replaced on merge by `main`'s `cloud-ghosting` history test (see design section 2)
- [x] 3. Composite fallback: no cloud when every texel is rejected
- [x] 4. Haze on clouds, `cloud_haze` in `weather.ron`
- [x] 5. Stills before and after; perf suite old against new; report; recording (on the branch build)
- [ ] 6. On the merged build: stills of the column fix and the haze, the perf suite against the baseline exe, and a recording
- [ ] 7. `cloud_haze`: 3 thins a cloud 1 to 2 km away from low altitude nearly to nothing (`output/captures/ghost/compare.png`); 1 leaves the horizon band. Fade by the ray's closeness to the horizon, not by distance alone, or choose between them on stills
