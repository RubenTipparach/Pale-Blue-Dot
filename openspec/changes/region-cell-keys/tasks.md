# Tasks

## 1. The write-up
- [x] 1.1 Regions of 64 cells a side, region and place layouts, the save's form, migration (design).

## 2. Build (on a request)
- [x] 2.1 (Built 2026-11-21 for `multiplayer` design 16: `cell_key::region`, with `region`, `place`, `whole`, `address` and `of_key`; `every_point_of_l7_and_l8_has_one_region_and_place_and_back`, `l11_and_l12_edges_corners_and_a_sample_round_trip`, `at_l11_a_region_is_a_level_5_triangle`.) `pbd-core::cell_key`: `region` and `place` from an address and back, canonical first; tests at L7, L11, L12 and on face edges and corners.
- [ ] 2.2 `Edits` keyed by region, then place.
- [ ] 2.3 The old-save reader kept; migration on open; a test that old key, address, region and place, and back agree for a sample world's edits.
- [ ] 2.4 Region files as `world-persistence` 8 describes, written with the new form.
