# Tasks

## 1. Baseline
- [x] Record the climate report (cover by band; clear, full and partial
      shares; land rain peak hour) and the covered-area brightness spread from
      an orbit capture, before anything changes (design section 4).

## 2. Cloud from humidity and heating (pbd-core)
- [x] Humidity cover (Sundqvist), with separate sea and land critical
      humidities; `cover` is the larger of it and the condensed-water cover.
- [x] Buoyant lift from the sunlight land absorbs (ground minus air never
      fired; design section 4), with two knobs.
- [x] Rain from updrafts and cold cloud at less water; warm threshold 4.0.
- [x] The slider's storm drives an updraft, so the RAIN preset still rains.
- [x] Mesoscale noise re-derived on restore (the save test caught it).
- [x] Retune evaporation and thresholds toward the targets; every knob in
      `atmosphere.ron` with units, validated.
- [x] Tests:
  - humid still air is partly cloudy and does not rain;
  - sunlit land lifts, and night land does not;
  - cold cloud rains out of less water than warm;
  - the RAIN preset rains, and not without its updraft;
  - water is still conserved, and save/resume is still exact.

## 3. Detail in the shader
- [x] Five octaves, the fine ones eroding edges and fading below a pixel.
- [x] Remap floor, so a full deck is not uniform.
- [x] Noise combed along the wind aloft (a squeeze could not work).
- [x] Cellular (smooth Worley) texture weighted by `cloud_top`.
- [x] Per-pixel march jitter (interleaved gradient noise).

## 4. Check
- [x] Climate report and captures against the design's targets; before and
      after tables in the design. Two targets are NOT met: the afternoon rain
      at one solstice, and the 2x interior spread (1.24x).
- [ ] Frame and step cost A/B.
- [ ] fmt, clippy, tests, `openspec validate --all`.
- [ ] Owner's in-game check.
