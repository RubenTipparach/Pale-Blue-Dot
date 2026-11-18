# Tasks

## 1. Measure (an instrument, design 1)
- [x] `BuildPhases` / `LAST_BUILD`: a test-only phase timer in
      `planet_column::build` and `settle_rivers` (columns, flood, settle with
      its queue, ticks, sweeps and changes, relight). Compiled out of the game.
- [x] `planet::column::river_cost::what_a_world_s_rivers_cost_to_build`
      (ignored): the drainage map fresh, and for four river anchors and a
      dry one the fine set, the river terms in altitudes, columns and far
      records, the tier's phases, a second settle, the water mesh.
- [x] Run it in the cloud container (4 cores, no GPU, `--profile fast`);
      numbers in the design, as CPU time on that machine.

## 2. The drainage map's ground on every core (design 2)
- [x] `RiverMap::build_on(cfg, settings, threads)`: the lattice's altitudes in
      ordered chunks on scoped threads; `build` uses every core.
- [x] Test `rivers::tests::the_map_built_on_every_core_is_the_one_thread_map`.
- [x] Existing river tests pass unchanged: `every_river_cell_drains_to_the_sea`,
      `a_rivers_water_sits_on_its_own_bed`,
      `river_tier_tests::a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`,
      and the generator digests.

## 3. Measure again, same sitting
- [x] Before and after interleaved (before, after, before, after). The
      container was shared with other sessions' game runs (load average 10-13
      on 4 cores), so the parallel gain is masked: the map took 419-586 ms
      before and 360-426 ms after on four threads (391-522 ms after on one).
      The cut on an idle desktop is not measured here.
- [ ] The owner's desktop: the `READY` line's world time on a version 8
      world, before and after, with nothing else running.

## 4. The drainage-map cache file (design 3), waiting on its decision
- [ ] Decide where it lives: the slot's `cache/` (recommended) or a shared
      per-user cache keyed by seed and version.
- [ ] Write and read the file: header (magic, format, generator version,
      seed, settings bits), nodes, digest; atomic write; any mismatch or
      failure rebuilds.
- [ ] Tests: a round trip equals the built map; a header of another seed,
      version or setting is refused and rebuilt; a truncated file is rebuilt;
      no file is written for a world without rivers.
- [ ] Measure a second open's map time with the file.

## 5. Cheaper river lookups (design 5), later
- [ ] Each node's centre and downstream centre stored in the map.
- [ ] A per-cell flag that no river is within the window.
- [ ] Measure each with `river_cost`; a test that a tier's altitudes and
      columns are bit-for-bit the same.

## 6. Checks
- [x] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      the river and water tests, `openspec validate --all`.
- [ ] Video or screenshots: none, the picture does not change (the map is
      the same bytes).

## 7. The dry channel (design 7), 2026-11-17
- [x] Reproduce at the owner's spot (`--walk --at 20.885 -6.393`, 91.6 m):
      the channel dry past the fall; the far sheet tinted (an uncommitted
      instrument) shows the gap from the tier's edge outward.
- [x] Probe the data: the river's far surface and column water are whole
      along its 1,593 m (an ignored probe, not kept).
- [x] The cause: the visibility pass tests every water sheet on the sea's
      sphere; a far river's sheet is drawn at its own surface.
- [x] The fix: `compact_visible` tests a far river's sheet at
      `R + max(surface, ground + 0.05)`, the radius `water.wgsl` draws it at.
- [x] GPU test on the shipped shader:
      `actual_gpu_lists_a_far_river_s_sheet_at_its_own_surface` (fails on
      `origin/main`'s shader: one of two listed).
- [x] Requirement "A far river is drawn wherever it is in view" moved to
      `openspec/specs/world/water` with its test.
- [x] Screenshots before and after at the owner's spot
      (`docs/screenshots/river-cache/`).

## 8. The river walk (design 8)
- [x] `RiverMap::course`: a river's cells from its source to the sea, with
      `a_course_runs_from_its_source_to_the_sea`.
- [x] `river_walk::RiverPath`: the bank path, progress and the drawn-water
      measure, with tests.
- [x] `--walk-river`: the walk through the player's inputs, its log lines.
- [x] The recording: before and after, one OBS, real time, cut into one
      video with progress captions (`river-walk-before-after-2026-11-17-900p60.mp4`,
      design 9); uploaded to Drive.

## 9. Measure again
- [x] `RIVER_AHEAD` along the whole walk, old rule against new, summarised
      in design 9.
- [ ] The perf suite on the release build (the fix lists more sheets).
