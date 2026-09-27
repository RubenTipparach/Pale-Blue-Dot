# Design: the sun holds the planet at 15 °C

## Context

See `proposal.md` for the owner's words and the measurements. What the design
has to work with, observed on `main` (2026-09-27):

- **The heat step** (`atmosphere/step.rs`), per cell:
  - `sunlight = solar_wm2 · max(0, cosZ) · (1 − cloud_albedo · cover)`;
  - `absorbed = sunlight · (1 − albedo)`;
  - `outgoing = olr_a + olr_b · T − cloud_greenhouse · cover`;
  - the ground's temperature moves by `(absorbed − outgoing − sensible) /
    heat_capacity`, plus a spread term.
- **The shipped values:** `solar_wm2` 1000, `cloud_albedo` 0.6,
  `cloud_greenhouse` 40, `olr_a` 203, `olr_b` 2.09, ocean heat capacity 3.0e6,
  land 5.0e4.
- **The grid** is level 5: 10,242 cells, each with an `area`. The atmosphere
  already sums area-weighted water (`mod.rs`), so an area-weighted mean
  temperature is the same pattern.
- **The save.** `to_bytes` writes the prognostic fields behind a magic tag.
  `saved_and_resumed_equals_stepped_straight_on` pins resuming.
- **The instruments.** `examples/fish_ranges.rs` runs 200 days and logs the
  mean sea surface every 10 days, in about 45 minutes on the cloud box.
  `examples/climate.rs` reports bands. Both take an `ATMOSPHERE='(...)'`
  override, so a retune is measured before it is made.

## Goals / Non-Goals

**Goals:**
- The planet's average holds at 15 °C, as the owner asked.
- Local climate stays emergent: warm tropics, cold poles, seasons and
  weather.
- A later retune of any weather knob cannot freeze or cook the planet.

**Non-Goals:**
- **Choosing a climate per place.** The thermostat moves one number, the
  planet's average. Where it is warm or dry is still the simulation's.
- **Sea ice as a surface.** The ocean does not render frozen. That is a later
  visual change, and the climate map will show where it would be.
- **A day-night cycle change.** The sun's direction and the clock are
  untouched.

## Decisions

**1. First close the two leaks: heat is moved inside the planet, never made
or lost.** The sweep found that the step loses heat in two places (Measured,
findings 2 and 5). The leaks froze the planet, not the sun or the clouds.
- **The spread becomes a flux of heat across each edge.**
  - Its conductance keeps today's rate in kelvin between two land cells, or
    two sea cells.
  - Across a coast, each side moves by the heat that crosses, over its own
    heat capacity. The land moves toward the sea's temperature, and the sea
    hardly moves, which gives coasts their mild climate.
- **The heat evaporation takes from the ground comes back where the water
  condenses.** The air's latent warming is the same joules, and it reaches
  the ground. `evaporation_cooling` and `latent_k_per_kg` stop being two
  numbers tuned apart: one follows from the other.
- A test pins each: a toy grid's heat total under the spread alone, and a
  column that evaporates and rains in place.
- *Alternative:* keep the leaks and tune the clouds and sun around them.
  Rejected, for two reasons:
  - Each leak's size follows the land-sea contrast and the rain, so a tuning
    that balanced them one season drifts the next.
  - Run E shows what happens when the spread's sign flips: cells blew up.

**2. Then fix the two terms, then add the thermostat.** The owner's words ask for
the sun to maintain the average, and both parts serve that.
- The fixed terms put the natural balance near 15 °C, so the thermostat's
  trim stays near 1.0. The sun is then not propping up a broken cloud term.
- The thermostat holds 15 °C exactly, and keeps holding it after any later
  retune.
- *Alternative:* the thermostat alone. Rejected: the sun would have to run
  far brighter than 1360 W/m² to beat −190 W/m² of cloud in the tropics,
  which would overheat every clear sky and every desert.
- *Alternative:* the retune alone. Rejected: nothing would stop the next
  retune from sliding the planet again, and the drift took 130 game days to
  show.

**3. The sun becomes a top-of-atmosphere value, and the clouds are tuned by
measurement.**
- `solar_wm2` goes to 1360, which is what Budyko's `olr_a` and `olr_b` are
  calibrated against.
- `cloud_albedo` and `cloud_greenhouse` are swept with the instrument's
  `ATMOSPHERE` override. The sweep reports the planet's mean net cloud effect
  (the cloud's reflected sunlight less its returned longwave, averaged over a
  year) and the mean temperature with the trim held at 1.0.
- The chosen pair puts the net cloud effect between −10 and −30 W/m², and the
  untrimmed mean as close to 15 °C as the pair allows. The sweep's table goes
  in this design.

**4. The thermostat is a slow proportional-integral controller on one
scalar.** The owner (survey K1): "well the world has different gradients, the
sun just has a constant solar output. do recommendation I guess". So the sun
has to read as constant. Once the two terms are fixed, the trim settles, and a
test pins its variation after spin-up under 1% over a game year. The
gradients from equator to pole are the simulation's, untouched.
- `sun_trim` multiplies `solar_wm2` everywhere.
- Each step it reads `M`, the area-weighted mean of the surface temperature
  over all cells, summed in cell order in `f64` so it is deterministic.
- It moves toward the target with time constant `sun_trim_s`, which starts at
  5 game days and is tuned on the instrument. That is slow next to a day, so
  the day and night swing never feeds it. It is fast next to the ocean's drift
  of weeks, so the drift never builds.
- It is clamped to `[sun_trim_min, sun_trim_max]` (0.7 to 1.4 to start). A
  trim at its limit is logged, since it means the terms are badly off again.
- The controller holds the instantaneous mean, so the planet's average barely
  swings with the seasons. Local seasons are untouched: the northern winter is
  still cold, balanced by the southern summer.

**5. The trim is saved and versioned with the weather.**
- `to_bytes` appends `sun_trim`, and the magic tag's version goes up by one.
- An old `weather.bin` reads with `sun_trim` 1.0 and is re-saved in the new
  format.
- The resume test covers the trim.

**6. The fish plan's tests get a second year.** A new test runs the balanced
atmosphere for 200 days, at level 3 so it fits in a test run. It asserts
every species has water in its window for part of the second year. It is
ignored by default, and run with the instrument, because it takes minutes.

## Measured: the sweep (2026-09-27)

Task 1.2's first passes run at level 4 (2,562 cells), 200 days a candidate,
with no trim, since there is none yet. The sea column is the plain mean of the
sea cells, as `fish_ranges` logs it:

| run | `cloud_albedo` | `cloud_greenhouse` | `evaporation_cooling`, J/kg | sea, day 50 | sea, last logged |
| --- | ---: | ---: | ---: | ---: | ---: |
| A (sun only) | 0.6 | 40 | 8.0e4 (shipped) | −12.5 °C | −15.6 °C, day 100 |
| B | 0.35 | 40 | 8.0e4 | −9.4 °C | −13.7 °C, day 110 |
| C | 0.25 | 50 | 8.0e4 | −7.2 °C | −9.9 °C, day 200 |
| D | 0.15 | 60 | 8.0e4 | −4.9 °C | −7.4 °C, day 200 |
| E | 0.25 | 50 | 1.0e4 | 21.1 °C | 15.5 °C, day 200, but cells blew up (finding 3) |
| F | 0.35 | 40 | 1.0e4 | 5.8 °C | −1.5 °C, day 200 |

Every run has `solar_wm2` at 1360. A and B were stopped once C and D showed
the trend.

**Finding 1: clouds alone do not fix it.** D has the weakest clouds, and its whole
surface still averages −9.7 °C in year 2.

**Finding 2: the spread between cells is the freeze.** It moves temperature, not
heat.
- In `heat`, each cell moves toward its neighbours' mean at `heat_spread` per
  second, in kelvin.
- At a coast, the sea cell holds 60 times the heat of its land neighbour per
  kelvin (3.0e6 against 5.0e4 J/m²K), yet both move by the same kelvins.
- So where the land is colder than the sea beside it, the sea loses 60 times
  the heat the land gains. A coastal sea cell next to land 1 K colder loses
  about 0.002 × 1/6 × 3.0e6 ≈ 1,000 W/m².

The heat budget measures it (below). The planet's mean, in W/m², at level 4:

| run, day | absorbed | spread | sea's carry | evaporation | stored | mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shipped, day 1 | 154 | −974 | −0.1 | 161 | −1,164 | 10.2 °C |
| shipped, day 7 | 135 | −600 | −0.6 | 67 | −686 | 3.5 °C |
| E, day 1 | 245 | −269 | 0.7 | 33 | −251 | 12.4 °C |
| E, day 7 | 242 | −101 | 0.7 | 29 | −63 | 12.5 °C |

- On the shipped settings the spread takes four to seven times all the
  sunlight the ground absorbs. Nothing else in the budget comes close.
- Evaporation is 67 to 161 W/m², about Earth's 80.
- An earlier reading of 662 W/m² for evaporation was this leak. It was hiding
  in the residual before the spread was measured.
- The sea's carry by the current is within ±2 W/m² of zero, so the advective
  form it uses does not leak.
- Days 1 to 7 are shown because the leak is largest while the land cools from
  the climatology the world starts with.
- The sign follows the land-sea contrast, which matches every run:
  - on the shipped settings, and in F, the land is colder than the sea, and
    the planet cools;
  - in E the land is warmer (day 46: whole surface 19.7 °C, sea 19.5 °C), and
    the planet warmed 0.37 °C a day.

**Finding 3: with the leak running the other way, E blew up.**
- E's sea mean reached 35 °C by day 80, then fell to 14 °C by day 90.
- Its maps hold cells at −10^19 °C: some cells ran away, and the step's guard
  (`step.rs`, `guard`) reset them to their climatology once they went
  non-finite.
- Which term ran away is not measured. The likely candidate is the
  evaporation's cooling in a hot, wet land cell. It is explicit, and it grows
  about 7% per kelvin, so past some temperature one step overshoots. A
  wet column that rains in place can also heat its own ground:
  - each kilogram that condenses warms the air by `latent_k_per_kg`;
  - the air gives that back as sensible heat, about `sensible_wm2k ·
    air_relax_s · latent_k_per_kg` = 15,750 J/kg;
  - E's evaporation cools the ground by only 1.0e4 J/kg.

  The shipped 8.0e4 is well above that return. With the leak fixed there is
  no reason to lower it, so E's setting is dropped.

**The instrument.** `examples/heat_budget.rs` measures where the heat goes,
and nothing in the game reads it. It runs the atmosphere forward and prints
the planet's area-weighted mean surface budget, in W/m², once a game day:
- absorbed sunlight, and the sunlight the cloud reflected;
- emitted longwave, and the cloud's returned longwave;
- sensible heat to the air;
- the change in the ground's and sea's stored heat;
- the heat the spread adds and the heat the sea's carry adds, each computed
  from the state with the step's own functions (`Grid::neighbour_excess`,
  `Grid::fluxes`, `Grid::upwind`);
- evaporation, as the residual;
- the mean temperature, the sea's mean temperature, and the mean cloud cover.

It takes the same `ATMOSPHERE` override as the other instruments.

**Finding 4: with no spread, the planet still cools, more slowly.** Runs G to J set
`heat_spread` to zero through the override, using only the shipped code, and
were stopped at day 50 once the trend was plain:

| run | `solar_wm2` | `cloud_albedo` | `cloud_greenhouse` | sea, day 10 | sea, day 50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| shipped (spread on) | 1000 | 0.6 | 40 | 7.7 °C | about −10 °C |
| G | 1000 | 0.6 | 40 | 12.9 °C | 4.7 °C |
| H | 1360 | 0.6 | 40 | 13.4 °C | 6.9 °C |
| I | 1360 | 0.35 | 40 | 13.5 °C | 7.7 °C |
| J | 1360 | 0.25 | 50 | 13.6 °C | 8.3 °C |

The heat budget on H names the second drain. Days 1 to 12, in W/m²:
- absorbed: 175;
- emitted, less the cloud's returned longwave: 195;
- sensible heat back from the air: 26;
- evaporation: 130 to 200;
- stored: −123 to −190.

**Finding 5: evaporation is the second leak.**
- The outgoing longwave `203 + 2.09 T` is Budyko's law for the top of the
  atmosphere. It already counts everything the air does between the ground
  and space, the latent heat included.
- In such a model, evaporation moves heat from where water evaporates to
  where it condenses. It does not take it out of the planet.
- The step takes `evaporation_cooling` (8.0e4 J/kg) from the ground. It gives
  back only what `latent_k_per_kg` warms the air by, returned as sensible
  heat: about `sensible_wm2k · air_relax_s · latent_k_per_kg` = 15 × 3000 ×
  0.35 = 15,750 J/kg.
- So about four fifths of every kilogram's heat is lost.

With both leaks closed, Budyko's balance is `absorbed + greenhouse = 203 +
2.09 T`. On H's budget:

| clouds | absorbed | greenhouse | balance |
| --- | ---: | ---: | ---: |
| shipped (0.6, 40), sun 1360 | 175 | 25 | about −1 °C |
| 0.35 and 40 | about 210 | 25 | about 15 °C |
| 0.25 and 50 | about 230 | 30 | about 27 °C |

**The next measurement: the planet without either leak.** The override
closes both, using only the shipped code:
- `heat_spread` 0;
- `evaporation_cooling` 15,750 J/kg, what the air column gives back.

| run | `solar_wm2` | `cloud_albedo` | `cloud_greenhouse` |
| --- | ---: | ---: | ---: |
| K | 1360 | 0.6 | 40 |
| L | 1360 | 0.35 | 40 |
| M | 1000 | 0.6 | 40 |
| N | 1360 | 0.25 | 50 |

The full runs, 200 days. The whole-surface means are area-weighted, over the
second year (days 101 to 200), from `tools/temperature_map.py`:

| run | sea, day 10 | sea, day 100 | sea, day 200 | whole surface, year 2 | sea, year 2 | range, year 2 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| K | 15.2 °C | 12.0 °C (day 90) | 9.4 °C | 5.6 °C | 10.0 °C | −28.8 to 21.2 °C |
| **L** | 15.7 °C | 15.8 °C | 16.3 °C | **12.1 °C** | 15.8 °C | −21.0 to 29.0 °C |
| M | 14.6 °C | 6.2 °C | −0.3 °C | −2.6 °C | 2.2 °C | −36.1 to 13.5 °C |
| N | 15.9 °C | 18.1 °C | 20.4 °C | 16.3 °C | 19.2 °C | −12.7 to 33.9 °C |

What the runs show:
- **L holds.** Its sea is within 0.6 °C from day 10 to day 200, and no cell
  runs away: the range stays between −21 and +29 °C. Its whole surface
  averages 12.1 °C. That leaves the thermostat about 3 °C, which is the
  small, steady trim decision 2 wants.
- **N is still warming** at day 200, by about 0.25 °C every ten days.
- **The shipped sun and clouds are too cold even without the leaks** (M).
  The retune of decision 3 is still needed.
- **The year-2 map of L** is `docs/wiki/temperature/preview-leaks-closed-year2.png`,
  and beside today's in `before-after-year2.png`:
  - tropical sea at 25 to 30 °C;
  - mid-latitudes at 10 to 20 °C;
  - the poles near −20 °C;
  - the freezing line near 60° north and south.

  Today's year 2 runs from −48 °C at the poles to −16 °C at the equator.
- The net cloud effect of L's pair (0.35 and 40) is not yet computed from
  the budget. Decision 3 wants it between −10 and −30 W/m². A pair between L
  and N may meet that and land nearer 15 °C untrimmed, and is task 1.4's
  first run on the fixed step.

These bracket the fix; they are not it:
- A spread that trades joules still moves heat between cells. Here none
  moves.
- A latent cycle that conserves gives each kilogram's heat back where it
  condenses. Here the ground is charged what a column that rains in place
  gets back, so a kilogram that rains out elsewhere moves heat only roughly.

## Risks / Trade-offs

- [Closing the leaks changes more than the average] → Coastal land becomes
  milder, since the sea now holds it. Inland stays extreme. Rain may shift
  where latent heat comes back. The climate report's cover and rain figures
  and the temperature maps are compared before and after, and any figure
  that moves past its spread is argued here.
- [A heat-conserving step exposes a runaway the leaks were hiding] → Run E
  blew up with the spread's sign reversed. The fixed step is run 200 days at
  level 5 and checked for any cell outside −80 to +60 °C. The `guard` resets
  only non-finite cells, so a runaway that stays finite would go unseen
  without that check.
- [The controller oscillates against the ocean's heat capacity] → It is
  tuned on the 200-day run, and the scenario's ±1 °C band from day 30 is the
  pass mark. If it rings, the integral term is slowed.
- [Retuning the clouds changes how the sky looks: more or less cover, a
  different rain share] → The climate report's cover and rain figures (median
  cover, p90, the share raining) are recorded before and after. Any that move
  past their measured spread are argued here or retuned.
- [The instrument runs 45 minutes a candidate] → The sweep uses level 4 for
  its first pass, a quarter of the cells. Only the chosen pair is run at the
  shipped level 5.
- [A frozen save warms slowly] → The thermostat acts at once. The ocean's heat
  capacity sets the pace, and the time is measured on a frozen save.
- [Frame cost] → One sum over 10,242 cells per step. It is not measured in a
  cloud session (CLAUDE.md), and the owner's `perf_suite.py` run covers it.

## Migration Plan

- The weather format gains one field under a new tag. An old save reads with
  the trim at 1.0.
- Rollback is the previous build. It refuses the new weather tag and starts
  the weather fresh from the climatology, as it does for a missing
  `weather.bin`. No terrain, edit or record is touched.
