# Tasks

## 1. Write-up
- [x] 1.1 Proposal, design, spec delta.

## 2. Instrument
- [x] 2.1 `tools/perf_suite.py` runs on macOS (`pgrep`, exe path without `.exe`).
- [x] 2.2 `PBD_CLOUD_RENDERER=march|puffs` overrides `cloud_renderer` for an A/B on one exe.
- [x] 2.3 The clouds' GPU time on Metal: pass-boundary timestamps, first start to last end (`planet_cloud_timer.rs`).

## 3. Puffs
- [x] 3.1 `pbd_core::cloud_puffs`: sites, and the puffs a site carries for a cover, top and optical depth; tests (stable by index, count, share of sites carrying a puff follows cover, fade not a step, towers stack).
- [x] 3.2 `weather.ron` values with validation; `cloud_renderer` defaults to `Puffs`.
- [x] 3.3 The instances are built each frame from the shown maps (the extracted `WeatherMapsNow`, in the render world's prepare), sorted far to near.
- [x] 3.4 Puff entry points in `water.wgsl` (beside the passes whose view and helpers they share): vertex (facing on the sphere), fragment (Crab-breeze's puff, the march's light), scene depth fade, fade with the eye inside.
- [x] 3.5 Water node: with Puffs, the march pass makes the near fog only, the puffs draw into the history targets, the resolve is skipped; the composite reads the cap as no cloud.
- [x] 3.6 The shader passes `shader_tests.rs` (the workspace suite) and draws; the march path is unchanged (`PBD_CLOUD_RENDERER=march`).

## 4. Verify
- [x] 4.1 fmt, clippy clean, tests pass except `planet_gen::tests::version_4_*`/`version_5_*`, which fail the same on clean `main` on this Mac (their digests were pinned on the Windows desktop), `openspec validate --all`.
- [x] 4.2 Stills, march above and puffs below: `docs/screenshots/2026-10-28-cloud-puffs-{orbit,surface,coast}.jpg`.
- [x] 4.3 Benchmark march / puffs / off on the Mac: `docs/benchmarks/2026-10-28-cloud-puffs/`, https://claude.ai/artifact/PuWVaY1gADriGx1KjfGrbx.
- [ ] 4.5 The same benchmark on the owner's desktop (RTX 3070).
- [ ] 4.8 Distant puffs (design 3b): depth test first, horizon cull, detail by size on screen; benchmark against `puffs-v1`.
- [ ] 4.6 The eye inside cloud under puffs (`-coast.jpg`: a light grey with a dark smear where the march draws an even fog).
- [ ] 4.7 Skip the near-fog pass when the eye is far from the layer; the puffs' remaining cost is mostly it and the composite.
- [ ] 4.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
