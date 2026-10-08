# Tasks

## 1. Measure (instruments only)

- [x] 1.1 `town` scenario in `tools/perf_suite.py` (Ashenstead, 19.644
      59.178, forced rain, 10:00), with its validity check.
- [x] 1.2 `spent()` around `bake_town_outsides` and `light_from_the_field`
      (the numbers are in the design).
- [ ] 1.2b The same around the weather upload, the blend and the rain-map
      refill.
- [ ] 1.3 A log line per weather publish (steps, held seconds, step/map split).
- [ ] 1.4 Frame-log columns `baked_verts`, `mats_touched`, `air_gen`,
      `blend_t`, `cover`, `rain`; `frame_graph.py` marks pair moves.
- [ ] 1.5 The sitting: `before` (09-25 baseline) against `after`, and
      `after` with `--no-shadows` and with `PBD_NO_CLOUDS=1`, on `town`,
      `clouds`, `storm`, `walk`, two repeats; the report in
      `docs/benchmarks/2026-10-08-town-frame-time/`, and a published page.

## 2. Decide

- [x] 2.1 Write each measured cost into the design beside its suspect, and
      pick the fixes that the numbers justify.

## 3. Fix (after 2.1)

- [x] 3.1 Town bake: points kept, towns off the tier baked once, a 4 ms
      budget (design decisions 2-4); `a_cap_the_tier_cannot_cover_samples_the_open_sky`.
- [x] 3.2 Field corners: points and reads kept until the mesh moves or the
      field changes (decision 1). The material rewrite threshold is not
      changed.
- [ ] 3.3 Weather upload: `to` only; rain-map buffer kept.
- [ ] 3.4 Whatever 2.1 adds (shadow casters, held steps, wind drift), each
      with before-and-after shots where it changes the picture.
- [ ] 3.5 The sitting again, after against before; p95 within 16.7 ms in
      `town`, and the report and page updated.
