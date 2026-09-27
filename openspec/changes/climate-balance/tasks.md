# Tasks

This lands before the map mockup, whose climate layer shows it, and after the
lights. That is the owner survey's K3 recommendation, and it moves if the
owner answers otherwise.

## 1. Measure

- [ ] 1.1 The instruments print the area-weighted mean surface temperature, the mean net cloud effect and the trim every ten days. Verify: a 20-day run on the shipped settings reproduces the committed log's mean sea surface on day 10 and day 20, and adds the new columns.
- [ ] 1.2 Sweep `cloud_albedo` and `cloud_greenhouse` at `solar_wm2` 1360, first at level 4, then the chosen pair at level 5 for 200 days, with the trim held at 1.0. Verify: the table (pair, net cloud effect, mean on day 100 and day 200, median cover, share raining) is in the design.

## 1b. The leak

- [ ] 1.3 The spread between cells trades heat, not kelvin. Each edge carries one flux of heat, set by the difference in temperature, and each side of the edge moves by that heat over its own heat capacity. Verify: a core test that a toy grid of land and sea keeps its area-weighted heat total to rounding under the spread alone; and the heat budget's `spread` column reads within ±1 W/m² of zero on the shipped settings.
- [ ] 1.3b The heat water takes to evaporate is given back where it condenses: the latent heat that condensing puts into the air reaches the ground as exactly the joules evaporation took, rather than `evaporation_cooling` and `latent_k_per_kg` being two numbers tuned apart. Verify: a core test that a column which evaporates and rains in place ends a day with the heat it started with, less the radiation; and the heat budget's evaporation and its return differ by under 1 W/m² over the planet.
- [ ] 1.4 The clouds and the sun swept again on the fixed step, as task 1.2 says. Verify: the design's table is replaced with the new runs, and its earlier rows are kept, marked as measured on the leaking step.

## 2. The thermostat

- [ ] 2.1 `target_mean_c`, `sun_trim_s` and the trim's limits in `AtmosphereSettings` and `atmosphere.ron`, validated with units. Verify: settings tests refuse a zero time constant and limits in the wrong order.
- [ ] 2.2 The area-weighted mean, summed in cell order in `f64`, and the trim stepped each step and applied to the sunlight. Verify: core tests that a toy grid held too cold warms to the target, that a too-warm one cools, and that the trim stops at its limits and is logged there.
- [ ] 2.3 The trim saved with the weather, with the tag's version bumped and old saves read at 1.0. Verify: `saved_and_resumed_equals_stepped_straight_on` covers the trim, and a format test covers an old save.

## 3. Balanced

- [ ] 3.1 Ship the swept `cloud_albedo` and `cloud_greenhouse`, `solar_wm2` 1360 and the thermostat. Verify: the 200-day instrument run holds the mean within 15 ± 1 °C from day 30 on, and its log is committed beside the old ones.
- [ ] 3.1a The trim reads as a constant sun (survey K1). Verify: over the second game year of the 200-day run, the trim varies by under 1%.
- [ ] 3.1b Temperature maps before and after, drawn by `tools/temperature_map.py` (survey K2: "We have a temperature gradient map of the surface don't we???? make one"). Verify: day 1, day 100 and the second year's maps are in `docs/wiki/temperature/`, for the shipped and the balanced settings.
- [ ] 3.2 The second-year fish test, ignored by default. Verify: it passes on the balanced settings and fails on the old ones.
- [ ] 3.3 Regenerate `docs/wiki/fish-ranges/` from the balanced run, and answer `fishing-and-equipment`'s question 8 with the owner's words. Verify: every species' year-2 map has range.
- [ ] 3.4 Sync the `world/weather` requirement, naming each test. Verify: `openspec validate --all`.

## 4. The owner's check

- [ ] 4.1 The gate video. Its shots are:
  - two time-lapses of the sea temperature over 200 game days, before and after, one frame per game day from the instrument's fields, drawn with the temperature overlay's colour ramp;
  - the mean against the day, for both runs;
  - in the game, the temperature overlay from orbit on day 150 of each run;
  - a fish caught in the second year.

  It is published on the gate page with a note that frame cost was not measured in the cloud session. Verify: the page is linked from the PR.
- [ ] 4.2 The owner watches and accepts. Verify: the quote is in `proposal.md`. Archive.
