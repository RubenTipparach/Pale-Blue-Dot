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
- [ ] 1.5 (Town runs done, 2026-10-08: `docs/benchmarks/2026-10-08-town-frame-time/`,
      https://claude.ai/artifact/UijFaGpci12P1wHjR4otzk. The sitting was
      stopped at the owner's request before clouds, storm and walk.) The sitting: `before` (09-25 baseline) against `after`, and
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
- [x] 3.7 The trace build (`--features trace`, `tools/trace_rank.py`) and
      decisions 5-7: templates cloned only to cut, resting craft sleep
      (`a_resting_craft_far_off_sleeps_and_is_checked`), a material's sun
      rewritten past 2e-3. Measured in the design.
- [ ] 3.6 A fresh world freezes for about 18 s at its start (8.7, 2.5 and
      6.8 s frames, both builds): measure whether opening a saved world pays
      any of it, and write it up.
- [ ] 3.5 The sitting again, after against before; p95 within 16.7 ms in
      `town`, and the report and page updated.
