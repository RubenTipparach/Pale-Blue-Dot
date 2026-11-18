# Tasks

## 1. Measure

- [x] 1.1 `--stand SECONDS` and the unfocused pacing lifted for `--frame-log` runs. Verify: a stand at the cascade logs `STAND_START` and `STAND_DONE` and quits.
- [x] 1.2 Where the frame goes at the cascade, against the control (proposal, "Where the frame goes"). Verify: `docs/benchmarks/2026-11-17-cascade-perf/report.md`.
- [x] 1.3 `cascade` and `cascade-walk` in `tools/perf_suite.py`. Verify: the scenarios list; a run of each when the screen is unlocked.

## 2. Cut (the same picture)

- [x] 2.1 `water_hidden` first in the water's fragment. Verify: captures at the cascade, `origin/main` against the branch, equal pixel for pixel; `the_flowing_water_s_depth_pre_pass_hides_what_its_shading_hides`.
- [x] 2.2 The flowing water's depth pre-pass. Verify: the same captures; naga validates `flowing_depth`; a normal launch reaches READY with no `LATE_PIPELINE`.

- [x] 2.3 A water tick that moves nothing keeps its version. Verify: the walk's log, 7 empty ticks with no `fine-set upload` and no `water mesh`; `cargo test -p pbd-app` (the water-flow tests).

## 3. Owed

- [ ] 3.1 The A/B on an unlocked, quiet machine: `python tools/perf_suite.py --scenarios cascade,cascade-walk,walk --repeats 2 --variant "before|<main exe>||2de3c03a" --variant after`. Not run in this sitting: the screen was locked throughout (the suite refuses) and other sessions' game windows and builds shared the GPU.
- [x] 3.2 The fine-set upload's light and materials worked out off the render thread (proposal, "Next"). Verify: `openspec/changes/fine-set-upload`.
- [ ] 3.3 The same A/B with `claude/fall-size-rules` (its mist rule) once both are on `main`.
