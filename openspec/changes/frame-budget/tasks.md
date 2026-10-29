# Tasks

## 1. Write-up
- [x] 1.1 Attribute the clouds-off tail from the `cloud-puffs` logs: proposal, design, spec delta.

## 2. Instruments (first; each one only measures)
- [x] 2.1 Frame log `WINDOW_FOCUS`/`WINDOW_OCCLUDED` lines (`desktop.rs::log_window_attention`); the suite invalidates a run whose window lost focus or was hidden, and on macOS refuses to start while the screen is locked (`screen_locked`). The cause on 2026-10-28 was a locked screen (`CGSSessionScreenIsLocked`), which no setting in the game can undo, so the continuous-unfocused and App Nap parts are not done: a run under a locked screen is refused instead.
- [x] 2.2 Re-run focused: `storm` with clouds off is within 0.3 ms of `clouds` at p50 (design 2e); the 600 ms phase was the locked screen. `cloud-hop` still finds no cloud in the benchmark world.
- [ ] 2.3 Pass-boundary GPU timestamps for compose plus cap, rain, lens, lens mist and the main opaque pass, as frame-log columns on Metal.
- [x] 2.4 (switch built; the sitting runs) `PBD_PASS_OFF=shower`. Run `clouds` and `walk` with `rain`, `lens`, `shower` and all three off, to split the near-ground cost (design 2b).
- [x] 2.5 A measurement test: `build_planet` at 1 and N threads, with and without town ground installed (design 2c).
- [ ] 2.6 A `clouds-running` scenario with the clock running (design 3).
- [ ] 2.7 Log when each render pipeline becomes ready (`PipelineCache`), to split the start-up frames of 1.7-1.9 s and 0.9 s between pipeline compiles and the first planet build, and to catch any pipeline first compiled mid-flight (first rain, first dive). If any is, compile it at load, behind the start-up.
- [ ] 2.8 A CPU profile (`samply`, release with debug symbols) of `clouds` near the ground and `walk`, to find the main thread's hot loops before any data-layout or SIMD work (shower rebuild, fine-set upload, planet build, puff build).

## 3. Landing hitch
- [x] 3.1 Key the town bake on the field it reads, not the set's pointer (coverage by `ColumnTier::reaches`; time-sliced, swapped whole; 67-75 ms to at most 3.1 ms a frame, measured). Still to write: the test that a landing 2 km away leaves a bake and an edit in a town's margin re-bakes it. Test: a landing 2 km away does not re-bake; an edit in a town's margin does.
- [ ] 3.2 Bake off the main thread, and ease the light so it never steps.
- [ ] 3.3 Benchmark `clouds` and `walk`, old exe against new.

## 4. World open
- [x] 4.1 Read the installed ground without a lock per height (a per-thread copy by generation; 16-thread height 8.4 µs to 0.59 µs, world-open rebuild 9.08 s to 1.28 s, heights bit-identical by checksum). Byte-identical planet test; generator digests unchanged.
- [ ] 4.2 The towns' rebuild as a pool task behind the old planet, with the epoch guard.
- [ ] 4.3 Benchmark: the `planet rebuilt` and `landed after` lines, `walk` and `clouds` max.

## 5. Near ground and steady frame (order from 2.3-2.4)
- [ ] 5.1 Shower as an instanced GPU draw; captures at 2 m and 150 m in rain.
- [ ] 5.2 Rain volume noise baked, or the volume at reduced resolution.
- [ ] 5.3 A 3D render scale: screenshots at 1.0, 0.75 and 0.5 into the owner survey, plus benchmark numbers.
- [ ] 5.4 Field light per mesh only on change, if 2.6 shows a cost.

## 6. Verify
- [ ] 6.1 fmt, clippy, tests, `openspec validate --all`.
- [ ] 6.2 Report in `docs/benchmarks/<date>-frame-budget/`, with the HTML page republished.
- [ ] 6.3 Video for the batch (CLAUDE.md: screenshots stand in for now).
