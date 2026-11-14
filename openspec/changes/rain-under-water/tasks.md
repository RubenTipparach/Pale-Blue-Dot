# Tasks

- [x] 1.1 Before shots under water at `--at 12.748 9.455` (5.4 m under), night and day, looking up 20, 45 and 70 degrees, the rain volume on and off.
- [x] 1.2 Find where the rain is drawn over the underwater frame (the pass order and the ray's start).
- [ ] 2.1 `water.wgsl`: `rain_seen`, the air ray and its weight; `rain_half` and `rain` use it.
- [ ] 2.2 Tests: the rain's ray leaves the water before it gathers; the whole of `water.wgsl` validates with naga, its imports spliced (closes `water-lit` 3.4).
- [ ] 3.1 After shots, the same set, and the README.
- [ ] 3.2 Open, not this change: the sheet's rain rings print as dark lines at the window's edge (seen with the rain volume off); `--rain 1` by day panics in an f64 clamp.
