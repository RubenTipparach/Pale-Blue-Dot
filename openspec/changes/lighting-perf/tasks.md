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

- [x] 1.1 `PBD_REFLECT=terrain|sea|capture` (a comma list of the parts to
      switch off; `0` stays all of them), so L8's three parts are priced
      apart. Re-run the pricing on `rain-stroll` and `storm` and write the
      split into the design; re-rank if it disagrees. (`57d94b5`, also
      `walk` and `screen` apart. Stroll: the walk is 2.1 of L8's 2.8 ms
      p50; storm: wet ground's screen trace is 1.1 of 1.4 ms. Re-ranked.)
- [x] 1.2 Per-pass GPU columns in `--frame-log` from the spans Bevy already
      records (`lite_bloom`, `water_compose`, `planet_shadow`, the
      reflection capture, the main 3D pass), and confirm which pass the
      terrain's shading is timed in. (`57d94b5`: 17 named columns plus
      `gpu_other_ms`; the capture newly spanned. The terrain draws in
      `main_transparent_pass_3d`; the tone pass has no span.)
- [x] 1.3 `perf_suite.py`: count the warm-up from the first frame, not from
      process start (a fresh exe's first frame came 50-55 s after launch on
      a cold shader cache, so frames right after it were counted), and
      leave the fresh world's planet rebuild out of `max` or mark it.
      (`57d94b5`, `224c137`: from the game's ready line where it prints
      one, else the first frame; the rebuild is left out and listed;
      `--reanalyse` redoes a sitting.)

## 2. Fixes, one A/B each against `f15a647` (after 1.1)

- [x] 2.1 L8 terrain trace ends when the ray is rising above the
      neighbourhood's top solid layer; the screen fallback is skipped for
      such a ray; both traces gated by `rain_near` (design rank 1).
      Shots: Holbrook lane both ways at 10:00 and 22:00 in rain, a puddle
      under a wall. Measure: `rain-stroll`, `storm`, `walk`.
      (`e9ed506`: the walk ends; the screen fallback stays, since houses
      and trees are meshes. Shots match to the pixel. Stroll p50 / p95 /
      p99 10.89 / 17.48 / 24.42 to 10.34 / 16.80 / 21.70 ms, terrain GPU
      4.43 to 3.99 ms.)
- [x] 2.2 L8 sea trace skipped where its reach projects under ~4 px; steps
      fall with distance (rank 2). Shots: a harbour from 30 m and 300 m.
      Measure: `storm`, `clouds`, `far-side`.
      (`2185478`: both screen traces skip a march whose last step lands
      within the three texels every step skips, exact; the approximate
      step falloff was not needed. Storm p50 / p95 6.51 / 11.26 to 6.25 /
      10.79 ms, terrain GPU 2.80 to 2.59 ms.)
- [x] 2.3 Not pursued (measured, 1.2): L8 capture skipped (history marked invalid) when dry and no sea
      is on screen; its buffer kept (rank 3). Measure: `walk`, `far-side`.
      The capture pass is 0.03-0.04 ms of GPU; switching it off only saves
      what its readers then skip.
- [x] 2.4 Cloud-puff sites rebuilt only when the maps, the cloud step or
      the eye's cell changes (rank 4). Shots: a cloud-hop capture
      sequence, frame for frame. Measure: `clouds`, `storm`.
      (`ebe412a`: what the sites read off the pair is kept per pair; a
      test checks the puffs bit for bit. Build 0.41 to 0.28 ms mean.)
- [x] 2.5 `near_lamps` and the env-var reads cached (rank 5), with 2.1 or
      2.4 if they touch the same files. (`6ec3ff8`; measured with 2.4.)
- [ ] 2.6 Bloom's composite folded into the tone pass; its uniforms made
      once (rank 7). Shots: a lamp at night and the sun's disc,
      pixel-diffed.
- [ ] 2.7 Lamp bake and its fade skipped when no lamp is in the set's
      reach (rank 6), only if landing timings show it costs (the night
      sitting did not: -0.04 ms).

- [ ] 2.8 Half resolution (owner, 2026-11-12: "consider like half
      resolutions for expensive stuff like bloom and fog rendering"):
      price every full-screen pass from the per-pass columns (1.2), then
      move the reflection traces (L8, the one expensive full-resolution
      work left after 2.1-2.3) into a half-resolution pass with a
      depth-aware upsample, so a puddle's edge against a wall does not
      halo. Shots: the Holbrook lane in rain at 10:00 and 22:00, a puddle
      under a wall, pixel-diffed; measure `rain-stroll`, `storm`, `walk`.
- [ ] 2.9 Fog and atmosphere at half resolution (the rain volume, the
      compose's distance fog) only where 1.2's columns show a pass costs
      more than the spread; bloom is already built from a quarter-size
      image and is left as 2.6 has it.

## 3. Close

- [ ] 3.1 A final sitting: `f15a647` against the change, all scenarios,
      report and page in `docs/benchmarks/<date>-lighting-perf/`.
- [ ] 3.2 Move the requirements below into `openspec/specs/platform/
      performance` with the tests and the suite rows that prove them.
