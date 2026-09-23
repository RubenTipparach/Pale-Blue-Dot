# Tasks

## 1. Measure
- [x] Transcribe the march's lighting (`tools/cloud_light.py`); record the top,
      the base and the shadow profile (design section 1).
- [x] Baseline at half cover, noon: from below (flat grey) and from 900 m
      above (deck median 0.57, brightest 0.69 of white).
- [ ] The rest of the baseline set, before the model changes: fair and storm
      cover, sun 20 deg, and a true side view. `--view shore --pitch 0` looks
      down at the ground, not across at the clouds, so the side view needs a
      level camera.

## 2. The model, in `pbd::clouds`
- [ ] The light march reads the layer's extinction; the `0.05` literal goes. A
      test reads the shader and fails on a numeric extinction there.
- [ ] Six-step light march with growing steps, clipped to the slab.
- [ ] Multiple-scattering octaves; two-lobe HG phase.
- [ ] Sky ambient from above through `tau_up`, ground bounce from below through
      `tau_down`; the night floor kept as the floor.
- [ ] `cloud_base_dark`, `cloud_storm_dark` and the `day*0.90` term removed;
      `CloudLayer` re-laid out and its size test moved with it.

## 3. Knobs
- [ ] The design's table into `WeatherSettings` and `weather.ron`, with units,
      validation and the code-defaults test.

## 4. Check
- [ ] `tools/cloud_light.py` updated with the new arithmetic; before and after
      tables in the design; the proposal's four claims hold on its numbers.
- [ ] Captures repeated and measured; frame time A/B on llvmpipe, stated as
      relative only.
- [ ] fmt, clippy, workspace tests, `openspec validate --all`.
- [ ] Owner's in-game check; then the requirement moves to `openspec/specs`.
