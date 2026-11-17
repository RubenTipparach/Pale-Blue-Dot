# Tasks

## 1. Measure

- [x] 1.1 `PBD_MOTION_TRACE`: one row per frame from `Last`, with the camera,
  the body, the float, a followed fish and the held tool in camera space as
  the GPU gets them, plus the held tool with the origin rebased (the A/B).
- [x] 1.2 `tools/motion_graph.py`: shimmer, neighbour residuals on screen,
  the fish's per-frame speed, and plots.
- [x] 1.3 Real-time `--fish` runs count from `READY` and recast, so a live
  run fishes (captures are unchanged).
- [x] 1.4 Before traces, walking and fishing: `docs/screenshots/movement-jitter/`.
- [ ] 1.5 The same traces on a quiet machine (other agents were building
  during these; frame time was not the measure).

## 2. Fish between steps (built)

- [x] 2.1 `School::before` and `School::drawn`; a caught fish is taken out of
  both.
- [x] 2.2 `Fishery::step_alpha`; `draw::schools` builds from `drawn`; the fish
  on the line is drawn at the lure.
- [x] 2.3 Test: `a_school_is_drawn_between_its_steps_and_a_held_fish_at_the_lure`.
- [x] 2.4 After trace: frames the fish did not move fell from 588 of 1 594 to
  106 of 1 297.

## 3. Camera-relative render origin (go-ahead 2026-11-17; design 6)

- [ ] 3.1 `RenderOrigin` (camera snapped to 16 m) and `PlanetRenderFrame.center = -origin`.
- [ ] 3.2 Walking, flight and vehicle cameras write `position - origin`.
- [ ] 3.3 Drawn rigid bodies' `Transform` from `Position - origin`; Avian's
  position-to-transform sync off for them.
- [ ] 3.4 Audit every camera and `GlobalTransform` reader (design 4.4) and the
  eleven shaders with a planet centre (design 4.5).
- [ ] 3.5 Run the offset-planet test (`--render-offset`) in play as well as in
  a static capture.
- [ ] 3.6 Verify: held-tool shimmer under 0.05 px rms, `--render-offset`
  captures unchanged, `READY` with no `LATE_PIPELINE`, and `perf_suite.py`
  before and after.
- [ ] 3.7 The walking camera between physics steps (design 6.5): position
  only, the look stays raw, teleports snap.
- [ ] 3.8 Tests: origins at 0, 4 800 m and a random offset agree; no jump
  across a rebase.
- [ ] 3.9 Same-frame stills before and after (design 6.7): town by day and
  night, sea, under water, waterfall, flying at 1 km.

## 4. Gate

- [ ] 4.1 The owner plays the fishing again.
