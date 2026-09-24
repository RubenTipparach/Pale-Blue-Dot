# Tasks

## 1. Measure
- [x] `cloud_pace` instrument: wind the GPU drifts with, the steering wind,
      overhead angular speed at the base, cover churn per 1, 5, 30 s.
- [x] Native captures of the halftone and the baseline frame time
      (`FRAME_WALL_MS` p50 9.2 ms, p95 15.6 ms, two runs).

## 2. Smooth
- [ ] Step count from the span, floor and cap; per-pixel white-noise offset.
- [ ] Captures before and after; frame time before and after.

## 3. Pace
- [ ] `cloud_pace` in `AtmosphereSettings` and `atmosphere.ron`, validated,
      scaling the cloud's steering flux in `carry` only.
- [ ] The wind map is the cloud's own carrying wind.
- [ ] Test: with the pace at a half, cloud moves half as far in a step while
      vapour and heat move as before.
- [ ] Re-run `cloud_pace`; numbers in the proposal.
