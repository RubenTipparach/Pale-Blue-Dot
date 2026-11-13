# Tasks

## 1. Write-up and instruments
- [x] 1.1 Proposal, design (what swaps, the measurement plan), spec delta.
- [x] 1.2 Instruments: `PBD_CLOCK_RUNS`, `PBD_CAPTURE_RUN`, `BAKED_LIGHT` lines.
- [x] 1.3 Measure each swap: which bake, how big the step, how often (design 2-6).

## 2. Ease
- [x] 2.1 The easing rule, one constant and one function, with a test (`pbd_core::light::bake_blend`, `BAKE_EASE_S` 1 s).
- [x] 2.2 L4: a new set's bounce baked with it (its 2-degree re-bakes measured and left, design 3).
- [x] 2.3 L7: a new set's lamp bake baked with it, shown at once over a set it continues; its fade takes the rule.
- [x] 2.4 Dusk and dawn: the light field blends across the relight (two halves on the GPU; edits and landings whole at once).
- [x] 2.5 Towns: the last bake kept per vertex (vertex colour, given at spawn) and blended from.
- [x] 2.6 Field-lit corners ease when the field changes under a still mesh.

## 3. Verify
- [x] 3.1 Before-and-after frames at each swap in `docs/screenshots/baked-light-ease/`, with a README naming each place (design 8).
- [x] 3.2 fmt, clippy (no new warnings), the tests (`pbd-app --lib`: all but the known `puffs_follow_the_map_and_come_far_to_near`), `openspec validate --all`. Requirements synced into `openspec/specs/planet/light/spec.md`, each scenario naming its test.
- [ ] 3.3 Frame cost: not measured here; the owner's `perf_suite.py` sitting (a landing's task now bakes 8-16 ms more on the pool; the GPU reads the light field's second half only in a dusk blend).
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now): a night walk past lanterns, a day walk past shaded walls, dusk in a town; then archive.
