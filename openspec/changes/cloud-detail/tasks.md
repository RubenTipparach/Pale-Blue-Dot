# Tasks

## 1. Baseline
- [ ] Record the climate report (cover by band; clear, full and partial
      shares; land rain peak hour) and the covered-area brightness spread from
      an orbit capture, before anything changes.

## 2. Cloud from humidity and heating (pbd-core)
- [ ] Humidity cover (Sundqvist) with `rh_crit`; `cover` is the larger of it
      and the condensed-water cover; rain unchanged.
- [ ] Buoyant lift from ground heating, with two knobs.
- [ ] Retune drying and sea evaporation toward the targets; every knob in
      `atmosphere.ron` with units, validated.
- [ ] Tests:
  - humid still air is partly cloudy and does not rain;
  - heated land lifts, but not at night;
  - water is still conserved.

## 3. Detail in the shader
- [ ] Five octaves, the fine ones eroding edges.
- [ ] Remap floor, so full cover is not uniform.
- [ ] Noise stretched along the wind aloft.
- [ ] Cellular texture weighted by `cloud_top`.

## 4. Check
- [ ] Climate report and captures against the design's targets; before and
      after tables here.
- [ ] Frame and step cost A/B.
- [ ] fmt, clippy, tests, `openspec validate --all`.
- [ ] Owner's in-game check.
