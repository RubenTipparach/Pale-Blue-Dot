# Tasks

## 1. Measure
- [x] `--turn DEG_PER_S` capture instrument (the walker's view turns steadily).
- [x] Baseline captures: still and turning, on the build from before.

## 2. Build
- [x] `cloud_depth` a pair flipping with the history; the march reads last
      frame's at binding 4.
- [x] `cloud_prev_eye` lane in `WaterView` and `water.wgsl`; the size test.
- [x] One depth test shared by the upsample and the history read.
- [x] Four-tap history read, per-tap test, renormalised; alone when none kept.

## 3. Verify
- [x] Still: before vs after within two runs of one binary.
- [x] Turning: the difference lies along silhouettes; no trail by eye.
- [x] fmt, clippy, tests, `openspec validate --all`.

## 4. Not done
- [ ] Frame cost on hardware with `--frame-log` in a windowed run.
- [ ] Requirement into `openspec/specs/`. It is verified by captures, not yet
      by a test, so it stays in this change, as `cloud-budget`'s do.
