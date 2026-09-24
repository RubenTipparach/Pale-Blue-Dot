# Tasks

## 1. Measure
- [x] `cloud_pace` instrument: wind the GPU drifts with, the steering wind,
      overhead angular speed at the base, cover churn per 1, 5, 30 s.
- [x] Native captures of the halftone and the baseline frame time
      (`FRAME_WALL_MS` p50 9.2 ms, p95 15.6 ms, two runs).

## 2. Smooth
- [x] Tried, in captures of the same view (numbers in the proposal): white
      noise instead of interleaved gradient noise (the halftone goes, a heavy
      grain replaces it); 64 steps (clean-ish, frame 9.2 -> 22.7 ms p50); 32
      steps (still grainy); light held over several samples (no saving, the
      density is the cost); the span clipped to the tallest cloud on the ray
      (still grainy at the edges). None is clean at an affordable cost.
- [ ] Temporal accumulation of the cloud pass: a history target, reprojected
      by direction, blended with each frame's jittered march. Written up
      before it is built.

## 3. Pace
- [x] `cloud_pace` in `AtmosphereSettings` and `atmosphere.ron`, validated,
      scaling the cloud's steering flux in `carry` only.
- [x] The wind map is the cloud's own carrying wind.
- [x] Test: with the pace at a half, cloud moves half as far in a step while
      vapour and heat move as before.
- [x] Re-run `cloud_pace`; numbers in the proposal. Requirement moved into
      `openspec/specs/world/weather/spec.md`.
