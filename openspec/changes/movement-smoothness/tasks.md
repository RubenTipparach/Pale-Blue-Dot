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

- [x] 3.1 `RenderOrigin` (camera snapped to 16 m) and `PlanetRenderFrame.center = -origin`.
- [x] 3.2 Walking, flight and vehicle cameras write `position - origin`.
- [x] 3.3 Drawn rigid bodies: none is drawn (the walker and the skiff have
  no drawn child), so Avian's sync stays on and their transforms stay in
  physics space; the rebase leaves `RigidBody` entities alone (design 6.9).
- [x] 3.4 Audit every camera and `GlobalTransform` reader (design 4.4) and the
  eleven shaders with a planet centre (design 4.5).
- [x] 3.5 Run the offset-planet test (`--render-offset`) in play as well as in
  a static capture.
- [x] 3.6 Verify: held-tool shimmer under 0.05 px rms (0.0046 walking,
  0.0022 fishing), `--render-offset` in play, `READY` with no
  `LATE_PIPELINE` (design 6.8).
- [ ] 3.10 `perf_suite.py` before and after on a quiet machine (owed: the
  machine was busy when this was built).
- [x] 3.7 The walking camera between physics steps (design 6.5): position
  only, the look stays raw, teleports snap.
- [x] 3.8 Tests: origins at 0, 4 800 m and a random offset agree; no jump
  across a rebase.
- [x] 3.9 Same-frame stills before and after (design 6.7): town by day and
  night, sea, under water, waterfall, flying at 1 km.

## 5. The hooked float and the line (design 7)

- [x] 5.1 Trace column: the line's start and the swayed rod's tip in camera
  space, as drawn.
- [x] 5.2 `DrawnFloat`: a run starting or stopping, or the strike, is eased
  out over a quarter second; float, line end and hooked fish drawn from it.
- [x] 5.3 The line starts at the rod's tip as posed this frame.
- [x] 5.4 `--fish-play` holds the button in a band (0.45 to 0.75), as a hand.
- [x] 5.5 Tests: a jump is eased, nothing else is moved; the tip follows the
  pose.
- [x] 5.6 The trace again: hooked float rms/p99 and frames over 20 px; the
  tip gap.
- [x] 5.7 Video: a bite seen from under water, before and after, one file
  (`--watch-hook`, design 7.6).

## 6. The fish's pose (design 8)

- [x] 6.1 Trace columns: the followed creature's drawn forward and up, and its
  turn per frame; `motion_graph.py` counts turns over 30 degrees.
- [x] 6.2 Measure before: a nibble and a strike under water, and fish swimming.
- [x] 6.3 `DrawnHeadings`: headings eased between steps, kept with no
  velocity, a held fish level along the heading it took the lure with.
- [x] 6.4 Tests: a held fish's bob does not turn it; a step's turn is spread
  over frames; a reversal turns about up.
- [x] 6.5 Measure after; fill in design 8.4.
- [x] 6.6 Video: the nibble and strike from under water, before and after,
  one file.

## 7. The simulation's precision (design 9)

- [x] 7.1 The claim that the simulation may stay planet-centred withdrawn;
  built in `world-frames`.

## 4. Gate

- [ ] 4.1 The owner plays the fishing again.
