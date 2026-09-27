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

**1. Fix the two terms, then add the thermostat.** The owner's words ask for
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

**2. The sun becomes a top-of-atmosphere value, and the clouds are tuned by
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

**3. The thermostat is a slow proportional-integral controller on one
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

**4. The trim is saved and versioned with the weather.**
- `to_bytes` appends `sun_trim`, and the magic tag's version goes up by one.
- An old `weather.bin` reads with `sun_trim` 1.0 and is re-saved in the new
  format.
- The resume test covers the trim.

**5. The fish plan's tests get a second year.** A new test runs the balanced
atmosphere for 200 days, at level 3 so it fits in a test run. It asserts
every species has water in its window for part of the second year. It is
ignored by default, and run with the instrument, because it takes minutes.

## Measured: the sweep (2026-09-27, in progress)

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

**1. Clouds alone do not fix it.** D has the weakest clouds, and its whole
surface still averages −9.7 °C in year 2.

**2. The spread between cells is the freeze.** It moves temperature, not
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

**3. With the leak running the other way, E blew up.**
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

**The next measurement: the planet without the leak.** `heat_spread` is a
setting, and zero is legal. So the override can preview a planet with no
leak, using only the shipped code:

| run | `heat_spread` | `solar_wm2` | `cloud_albedo` | `cloud_greenhouse` |
| --- | ---: | ---: | ---: | ---: |
| G | 0 | 1000 (shipped) | 0.6 | 40 |
| H | 0 | 1360 | 0.6 | 40 |
| I | 0 | 1360 | 0.35 | 40 |
| J | 0 | 1360 | 0.25 | 50 |

A spread that trades joules would move heat between cells without making any.
With no spread, heat moves only by the air and the sea's current, so these
runs bracket that fix rather than being it.

## Risks / Trade-offs

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
