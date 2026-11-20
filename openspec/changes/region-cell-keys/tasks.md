# Tasks

## 1. The write-up
- [x] 1.1 Regions of 64 cells a side, region and place layouts, the save's form, migration (design).

## 2. Build (on a request)
- [ ] 2.1 `pbd-core::cell_key`: `region` and `place` from an address and back, canonical first; tests at L7, L11, L12 and on face edges and corners.
- [ ] 2.2 `Edits` keyed by region, then place.
- [ ] 2.3 The old-save reader kept; migration on open; a test that old key, address, region and place, and back agree for a sample world's edits.
- [ ] 2.4 Region files as `world-persistence` 8 describes, written with the new form.
