# Tasks

## 1. Measure (instruments)

- [x] 1.1 `PBD_STEP_TRACE`: one row per physics step of the walker's stored
  position and velocity at full precision, the origin, and the world point.
- [x] 1.2 `tools/step_graph.py`: per-step |move - v dt| on a steady walk,
  rebases counted, an excerpt at full precision.
- [x] 1.3 `PBD_WALK_AXIS_SCALE`: a slow scripted walk.
- [x] 1.4 Before: slow walk on the column tier (design 2); fish and float
  measured planet-centred by the guard tests.

## 2. Frames (pbd-core)

- [x] 2.1 `FrameTree`, `FrameKind`, `FrameMotion`, `convert` through the common
  ancestor, `reparent_vessel`.
- [x] 2.2 `WorldPos`, `FrameState`, `SimOrigin` and its conversions; no
  `From`/`Into` between local and world.
- [x] 2.3 `Rebase`, `cell_for_speed`, `Handoff`.
- [x] 2.4 Tests: rebase, slow step at 4 800 m and 4 800 km, round trips, a
  velocity's rate, a ship at 10 km/s with a walker and a rebase every tick, a
  trip through the system and back, an arrival, a reparented vessel.

## 3. The home planet in local space (pbd-app)

- [x] 3.1 `SimFrame`, `WorldFrames`, the rebase at a tick boundary, hooks.
- [x] 3.2 The walker: physics, ground, climbs, decks, camera, teleports.
- [x] 3.3 The f64 floor the walker stands on (`floor_precise`).
- [x] 3.4 The skiff: body, protection, route, readout, teleports.
- [x] 3.5 The fish and the line (`School`, `Line`, the fishery, its drawing).
- [x] 3.6 Every reader of the walker's position: planet points via the origin.
- [x] 3.7 Saves unchanged on disk; converted at the boundary.
- [x] 3.8 `PBD_HOLD_SIM_ORIGIN=1` for the A/B.

## 4. Guards

- [x] 4.1 Types.
- [x] 4.2 `ORIGIN_LEAK` in a debug build (walker, angler, school, float).
- [x] 4.3 Guard tests at 4 800 m, 4 800 km and 1 AU; the walker in the game's
  plugins across a rebase; a rebase moves no body; a pose round-trips.
- [x] 4.4 The source check (`tests/origin_guard.rs`).

## 5. Verify

- [x] 5.1 After: the slow walk, an 8 m/s walk across rebases, the held origin.
- [x] 5.2 Fishing traces before and after (design 7.1).
- [x] 5.2a Across a rebase: physics, drawn motion and pixels (design 7.2);
  `PBD_REBASE_SHOTS`.
- [x] 5.3 Launch a fresh world and an old build's save (design 7.3).
- [ ] 5.3a Dig, place and board a skiff by hand (not scripted here).
- [ ] 5.4 `perf_suite.py` before and after on a quiet machine (owed).

## 6. The second pass (design 10)

- [ ] 6.1 The herd in local space: `stride`, the f64 stand, the hook, the
  drawing through the render frame; `Seen` a query point.
- [ ] 6.2 Drops as body-frame f64; the save's text unchanged.
- [ ] 6.3 Decks in local space (moored and under sail) with a hook; the
  sailing cog's solids asked in local space.
- [ ] 6.4 Towns' floors, stairs and ceilings in f64 (`Frame::local_f64`,
  `world_f64`, `stand_in`, `ceiling_in`).
- [ ] 6.5 The line's sag laid in local points.
- [ ] 6.6 Tests (design 10.6) and the source check extended.
- [ ] 6.7 The rule in CLAUDE.md.
- [ ] 6.8 Measure: an animal's step error before and after; a fresh world
  near animals walked past a rebase (READY, no panic, no LATE_PIPELINE).
- [ ] 6.9 Later: saved points with their frame, when a second frame can be
  saved in; terrain as altitudes for large radii (design 9).

## 7. Gate

- [ ] 7.1 The owner walks and fishes on it.
