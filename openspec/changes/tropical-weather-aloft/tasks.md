# Tasks

## 1. Measure and choose

- [x] 1.1 (2026-09-29, the design's "Measured" and table; `docs/screenshots/tropical-weather-aloft/options.jpg`.) Read the mockup's Jet frames back into speeds. Take the built rule apart on generator 5's level-5 settled climate through a day, and draw and measure candidates D, F and H. Verify: the numbers are in the design and the picture is checked in.
- [ ] 1.2 **Gate:** the owner answers W2 (which rule) and W3 (write up the tropics' surface weather). Record the words in `proposal.md`, and move the spec delta to the chosen rule.
  - [x] 1.2a (2026-09-29: "recommended".) W3: the tropics' surface weather is written up next, after W2 is built.
  - [ ] 1.2b W2: answered "looks good now", which reads as H or as keeping what is built. Asked back in the survey's comment thread.

## 2. The wind (if D or H)

- [ ] 2.1 `aloft`: fade the thermal wind, then cap it, with `f` floored at 30 degrees. Verify: a core test on the level-5 settled climate: no more than 20% of cells within 5-20 degrees are at the cap (as built: 49-58%). Nothing poleward of 30 degrees changes, cell for cell. The edge test still holds (see the design's "Risks").
- [ ] 2.2 (H) The outflow's potential: relaxed by warm-started Jacobi sweeps each step, the count a validated setting. Before building, measure how many sweeps keep the day-mean outflow within 10% of the converged one. Verify: that measurement is in the design; `chi` is finite after a restore and after 600 steps.
- [ ] 2.3 (H) `tropical_outflow_mps` and `tropical_circling_mps` in `AtmosphereSettings` and `assets/config/atmosphere.ron`, with units, validated finite and within 0..`jet_max_mps`, and the easterly's default at 6 m/s. Verify: the settings tests, including a refused negative or non-finite value.
- [ ] 2.4 (H) The two terms in `aloft`, weighted by `1 - fade`. Verify: a core test on the settled climate. Within 10 degrees, the speed's spread round the planet is at least 3 m/s (as built: 0.4). With both settings at zero, the rule is D's.
- [ ] 2.5 The climate report before and after, its numbers in the design. Verify: the `atmospheric-circulation` claims still hold.

## 3. What new worlds start from

- [ ] 3.1 The settled climates made again for every carried generator. Verify: a new world's days hold 15 ± 0.5 °C, and the tests that restore them pass.
  - [ ] 3.1a Level 3, generators 4 and 5, in the cloud.
  - [ ] 3.1b Level 5, generators 4 and 5, on the owner's desktop. This is the same run as `tropical-upper-wind` 3.1b, which then closes with it.
- [ ] 3.2 An old save opens and carries on. Verify: a save made before the change restores byte for byte, and its next steps are finite.

## 4. Seen

- [ ] 4.1 The map mockup's weather frames made again, and the mockup republished. Verify: the Jet overlay's band varies round the planet.
- [ ] 4.2 Before and after `--capture` shots of the globe's and the map's Jet overlay, in `docs/screenshots/tropical-weather-aloft/`.
- [ ] 4.3 Sync the requirement into `openspec/specs/world/weather`, naming the tests. Verify: `openspec validate --all`.
- [ ] 4.4 Frame cost not measured in the cloud. The owner runs `tools/perf_suite.py`. Verify: the note is in the PR.
