# Tasks

## 0. Measure (done, 2026-11-11; clean re-run 2026-11-12)

- [x] 0.1 The A/B on the owner's desktop: `db25fe2` against `f15a647`, and
      `f15a647` with `PBD_LIGHTING=today`, on `clouds`, `storm`,
      `far-side`, `walk` and `rain-stroll`, two repeats
      (`docs/benchmarks/2026-11-12-lighting-model-clean/`, a quiet machine;
      the 2026-11-11 sitting it replaces was contaminated), plus `town`.
- [x] 0.2 The pricing sitting: each part off in turn on `rain-stroll` and
      `storm` (numbers in the design).
- [x] 0.3 The night sitting: `rain-stroll` at 22:00, new, old, L4 + L7
      off, L8 off.
- [x] 0.4 A read of the model's shaders and systems for costs that buy no
      picture (the design's "Read" lines).

## 1. Instruments (no behaviour change)

- [ ] 1.1 `PBD_REFLECT=terrain|sea|capture` (a comma list of the parts to
      switch off; `0` stays all of them), so L8's three parts are priced
      apart. Re-run the pricing on `rain-stroll` and `storm` and write the
      split into the design; re-rank if it disagrees.
- [ ] 1.2 Per-pass GPU columns in `--frame-log` from the spans Bevy already
      records (`lite_bloom`, `water_compose`, `planet_shadow`, the
      reflection capture, the main 3D pass), and confirm which pass the
      terrain's shading is timed in.
- [ ] 1.3 `perf_suite.py`: count the warm-up from the first frame, not from
      process start (a fresh exe's first frame came 50-55 s after launch on
      a cold shader cache, so frames right after it were counted), and
      leave the fresh world's planet rebuild out of `max` or mark it.

## 2. Fixes, one A/B each against `f15a647` (after 1.1)

- [ ] 2.1 L8 terrain trace ends when the ray is rising above the
      neighbourhood's top solid layer; the screen fallback is skipped for
      such a ray; both traces gated by `rain_near` (design rank 1).
      Shots: Holbrook lane both ways at 10:00 and 22:00 in rain, a puddle
      under a wall. Measure: `rain-stroll`, `storm`, `walk`.
- [ ] 2.2 L8 sea trace skipped where its reach projects under ~4 px; steps
      fall with distance (rank 2). Shots: a harbour from 30 m and 300 m.
      Measure: `storm`, `clouds`, `far-side`.
- [ ] 2.3 L8 capture skipped (history marked invalid) when dry and no sea
      is on screen; its buffer kept (rank 3). Measure: `walk`, `far-side`.
- [ ] 2.4 Cloud-puff sites rebuilt only when the maps, the cloud step or
      the eye's cell changes (rank 4). Shots: a cloud-hop capture
      sequence, frame for frame. Measure: `clouds`, `storm`.
- [ ] 2.5 `near_lamps` and the env-var reads cached (rank 5), with 2.1 or
      2.4 if they touch the same files.
- [ ] 2.6 Bloom's composite folded into the tone pass; its uniforms made
      once (rank 7). Shots: a lamp at night and the sun's disc,
      pixel-diffed.
- [ ] 2.7 Lamp bake and its fade skipped when no lamp is in the set's
      reach (rank 6), only if landing timings show it costs (the night
      sitting did not: -0.04 ms).

## 3. Close

- [ ] 3.1 A final sitting: `f15a647` against the change, all scenarios,
      report and page in `docs/benchmarks/<date>-lighting-perf/`.
- [ ] 3.2 Move the requirements below into `openspec/specs/platform/
      performance` with the tests and the suite rows that prove them.
