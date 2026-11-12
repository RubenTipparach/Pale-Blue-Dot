# Tasks

## 1. Write-up

- [x] 1.1 Proposal, design and spec deltas (this change); `openspec validate
      loading-screen`.

## 2. Build the world once

- [ ] 2.1 `towns::grounds_of` and `towns::read_or_lay` out of `settle` and
      `start_towns`, both calling them; no behaviour change.
- [ ] 2.2 `planet::PreparedPlanet`: `build_planet` behind a public type;
      `create_planet` takes one when present and builds otherwise.
- [ ] 2.3 The loading thread: sites, towns (waiting on the writer's mark),
      ground, planet; `WorldSites` and `Towns` start from what it made.
- [ ] 2.4 `tests/loading_terrain.rs`: the planet and the stored records are
      identical between the loading path and the old build-then-rebuild.

## 3. The screen and the warm-up

- [ ] 3.1 `Main` replaced by a gated runner; the `Loading` schedule; game
      systems in `PostUpdate` and `Last` that need `Startup`'s resources run
      only once the world is open.
- [ ] 3.2 The screen: its own camera while loading, the game's UI camera
      after, the steps and the bar.
- [ ] 3.3 Pipeline progress from the render world; the look-round; the lift
      conditions and the 90 s cap; the clock held.
- [ ] 3.4 `READY` and `LATE_PIPELINE` lines.
- [ ] 3.5 Captures wait for `READY`; `--capture-loading` photographs the
      screen itself.
- [x] 3.6 A capture is deterministic (decision 5): time paused while
      loading, no warm-up, `--frames` from `READY`; two captures of one scene
      compared pixel for pixel.

## 4. Measure and show

- [ ] 4.1 Before and after, release build, cold (a fresh copy of the exe) and
      warm, a new world and a saved one: time to playable and the worst frame
      in the first 60 s (`--frame-log`, `tools/frame_graph.py`); in
      `docs/benchmarks/2026-11-12-loading-screen/`.
- [ ] 4.2 GPU `--capture` shots of the loading screen and of the first
      playable frame in `docs/screenshots/`.
- [ ] 4.3 `cargo fmt`, clippy, the tests, `openspec validate --all`.

## 5. Left for later

- [ ] 5.1 Switching worlds from the saves page through the same screen and
      the same build-once path (`load_world` still rebuilds in place).
- [ ] 5.2 The perf suite counts its warm-up from `READY`.
- [ ] 5.3 Prewarm what `LATE_PIPELINE` names (map screen, vehicles, water
      from below, rain at night) by showing each once behind the screen.
- [ ] 5.4 The step's video, for the owner's batch.
