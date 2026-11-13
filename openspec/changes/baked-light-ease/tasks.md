# Tasks

## 1. Write-up and instruments
- [x] 1.1 Proposal, design (what swaps, the measurement plan), spec delta.
- [x] 1.2 Instruments: `PBD_CLOCK_RUNS`, `PBD_CAPTURE_RUN`, `BAKED_LIGHT` lines.
- [x] 1.3 Measure each swap: which bake, how big the step, how often (design 2-6).

## 2. Ease
- [ ] 2.1 The easing rule, one constant and one function, with a test.
- [ ] 2.2 L4: a new set's bounce baked with it (its 2-degree re-bakes measured and left, design 3).
- [ ] 2.3 L7: a new set's lamp bake baked with it, shown at once over a set it continues; its fade takes the rule.
- [ ] 2.4 Dusk and dawn: the light field blends across the relight.
- [ ] 2.5 Towns: the last bake kept per vertex and blended from.
- [ ] 2.6 Field-lit corners ease when the field changes under a still mesh.

## 3. Verify
- [ ] 3.1 Before-and-after frames at each swap in `docs/screenshots/baked-light-ease/`, with a README naming each place.
- [ ] 3.2 fmt, clippy, the tests, `openspec validate baked-light-ease`.
- [ ] 3.3 Frame cost: not measured here; the owner's `perf_suite.py` sitting.
