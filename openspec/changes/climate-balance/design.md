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

**1a. How the books close, term by term (written 2026-09-27, before the
code of tasks 1.3 and 1.3b).**
- **The spread** is `Grid::conduct`. Each edge carries a heat flux,
  `g · (T_k − T_i)`, in watts. The conductance `g` is symmetric: `heat_spread
  · min(C_i, C_k)` times the mean of the two cells' area per side. Cell `i`
  moves by the sum of its edges' fluxes over `C_i · A_i`. Between two land
  cells, or two sea cells, that is today's rate in kelvin to within the
  grid's area spread. Across a coast the land moves at today's rate and the
  sea by `C_land / C_sea` of it, a sixtieth. The sum of `C · A · ΔT` over the
  planet is zero to rounding.
- **The air holds heat.** The air relaxes to the ground at `1 / air_relax_s`
  while the ground gives it `sensible_wm2k · (T_g − T_a)`. That is exactly
  the exchange of an air layer with heat capacity `C_air = sensible_wm2k ·
  air_relax_s`, 45,000 J/m²K on the shipped settings, so the air is counted
  as a store of that size. The air's update reads the ground's temperature
  from before the step, as the ground's sensible term does, so the two sides
  of the exchange are the same joules.
- **Latent heat is one number.** A kilogram that condenses warms the air by
  `latent_k_per_kg`, which is `C_air · latent_k_per_kg` joules: 15,750 J/kg on
  the shipped settings. Evaporation takes the same joules from the ground,
  so `evaporation_cooling` is no longer a setting. It is
  `AtmosphereSettings::latent_j_per_kg()`, derived.
- **Cloud that evaporates back to vapour cools the air** by the same
  `latent_k_per_kg`. Without it, a kilogram that condensed, re-evaporated and
  condensed again would warm the air twice for one evaporation.
- **The air's carry keeps its heat (found on the fixed step, finding 6).**
  The wind carries the air's temperature in the advective form, because a
  single layer must not pile warmth up where air converges; in the real
  atmosphere that air rises and its heat leaves aloft. The form is kept. What
  it gains or loses over the planet each step is given back to the air evenly,
  per square metre: an energy fixer, as climate models use one. The heat that
  converging air carries away aloft comes down everywhere, which is roughly
  what the missing upper branch of the circulation would do with it.
- **A lightning strike's cold pool keeps its heat too (finding 7).** The
  pool chills the struck cell's air by `pool_k`, which is what lifts the air
  around it into the next storm, and it stays. The heat it took is given
  back to the air evenly, as the carry's is.
- **What still makes or loses heat, on purpose:** the sun and the outgoing
  longwave; the weather slider's forcing; and the guard that resets a
  non-finite cell.

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

**4, revised (2026-09-27, measured on the second pass): the thermostat sets
the sun from the planet's energy balance, and only nudges it.** The first
version below rang against the sea (runs X and Y). The books now close
(decision 1a), and the outgoing longwave is linear in the temperature, so a
planet that has settled satisfies, averaged over its surface:

```latex
\bar{A}_1 \, t + \bar{G} = a + b \, T
```

where `A₁` is the sunlight the ground would absorb at a trim of 1, `G` the
clouds' returned longwave, `a` and `b` are `olr_a` and `olr_b`, and `T` the
mean. So the trim that settles the planet at the target is
`(a + b · target − G) / A₁`, read straight off the budget.
- **The balance trim.** `A₁` and `G` are averaged over `sun_balance_s`, 50
  game days, long enough that a day's weather and a season's swing barely
  move it, which is K1's constant sun: at 25 days the seasons' clouds still
  moved the trim 1.2% over year 2 (VC2). The trim is that ratio.
- **The nudge.** A proportional term, `sun_trim_per_k` of trim per kelvin off,
  hastens the approach while the sea is still far from settled, and fades
  to nothing as it arrives.
- **The integral** is kept, slow (`sun_trim_s`, 50 game days), for what the
  books still leak (finding 6's last few W/m²), which would otherwise hold
  the planet a kelvin or so off. It only runs within `sun_trim_band_k` (1 K)
  of the target. A toy planet with a slow sea, started 10 K cold, showed why
  (`a_planet_with_a_slow_sea_settles_without_ringing`): the integral built up
  a fifth of the sun during the long approach and the planet overshot to
  18.7 °C, although the balance trim alone was right to within a percent.
- The averages and the integral are saved with the weather.
- **A new world starts at the target.** The balance trim is right once the
  sea has caught up, and the sea takes hundreds of days. Pair V under it,
  started from the old climatology (`28 − 45 sin²(lat)`, 13 °C averaged over
  the sphere), was still at 13.6 °C on day 60. So the climatology a new world
  starts from averages the target over the sphere: `28 − 3 (28 − target)
  sin²(lat)`, 28 °C at the equator and −11 °C at the poles for a target of
  15. It keeps the old equator because the fish's temperature windows were
  set on it: moving the whole climatology up two kelvin put a day-one
  equatorial river at 30.1 °C, past every river species' window
  (`on_day_one_no_open_water_is_without_a_species`). The thermostat then
  holds a planet at 15 °C, rather than having to warm one there. With no
  target, the old climatology stands.
- *Alternative:* a slower integral alone. Rejected: it is slow in both
  directions, so a frozen save would take years of game time to warm, and it
  would still ring against the sea, only more slowly.

**4, as first written. The thermostat is a slow proportional-integral controller on one
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

**Finding 6: the air's carry is a third leak (2026-09-27, on the fixed
step).** With the spread and the latent cycle closed (tasks 1.3 and 1.3b),
the heat budget counts the air and the vapour as stores and reports what
radiation does not account for. At level 4 on the shipped settings, days 2
to 4, in W/m²:

| day | absorbed | emitted less cloud+ | ground stored | air | latent | spread | leak | air's carry |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 129.7 | 201.8 | −99.6 | −5.6 | 1.7 | 0.0 | 31.5 | −30.6 |
| 3 | 126.3 | 199.6 | −102.5 | −4.8 | −1.8 | 0.0 | 35.8 | −31.5 |
| 4 | 124.9 | 199.2 | −100.5 | −3.8 | −2.8 | 0.0 | 32.8 | −31.9 |

- The spread now reads 0.0: task 1.3's check.
- The leak is the heat the planet lost beyond its radiation. The air's carry,
  computed from the sampled state with the step's own `upwind`, is nearly all
  of it.
- The advective form loses heat where warm air converges, at the thermal
  equator, and gains it where cold air diverges. Over the planet that is a
  loss, about a quarter of the absorbed sunlight.
- Decision 1a closes it with an energy fixer.

**Finding 7: the lightning's cold pools are a fourth leak (2026-09-27, on
pair V).** Under the balance thermostat, pair V held steady but cold: 12.8 °C
at a nudge of 0.01 a kelvin, 13.9 °C at 0.04, with the balance trim near
0.9. The heat budget, now counting the pools, found the planet losing 10 to
12 W/m² beyond its radiation, and the pools taking 14 to 15 W/m² of it (level
4, days 1 to 3). Decision 1a said a pool was "local and rare"; with this
much convection it strikes somewhere every few steps. Decision 1a now gives
the pool's heat back.

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

## Measured: the sweep on the fixed step (2026-09-27, task 1.4)

With the three leaks closed (tasks 1.3 to 1.3c), the clouds were swept again
at level 4 for 200 days, `solar_wm2` 1360, the trim held at 1
(`target_mean_c: None`), with `examples/fish_ranges.rs` and its new columns.
The year-2 figures are area-weighted over days 101 to 200 from the fields; the
net cloud effect is the instrument's, averaged over the last ten days.

| run | `cloud_albedo` | `cloud_greenhouse` | net cloud, W/m² | whole surface, day 100 | whole surface, day 200 | whole surface, year 2 | sea, year 2 | range, year 2 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q | 0.6 | 40 | −75 | 9.9 °C | 7.9 °C | 8.8 °C | 11.1 °C | −26 to 16 °C |
| R | 0.45 | 40 | −51 | 12.7 °C | 12.4 °C | 12.6 °C | 14.6 °C | −24 to 20 °C |
| S | 0.35 | 40 | −36 | 14.4 °C | 15.2 °C | 15.0 °C | 16.8 °C | −22 to 24 °C |
| T | 0.25 | 50 | −17 | 16.9 °C | 19.1 °C | 18.3 °C | 19.7 °C | −19 to 30 °C |

- **No cell runs away** on the fixed step, at any of the four: every range sits
  well inside the risk note's −80 to +60 °C. Run E's blow-up is not repeated.
- **No water is frozen all year** in any run. On the old step, year 2 froze
  every point of water.
- **The shipped clouds (Q) still cool**, 2 K over the 200 days, because they
  take 75 W/m², more than twice Earth's.
- **S lands on 15 °C untrimmed**, but its clouds take 36 W/m², just outside
  decision 3's −10 to −30. **T is in Earth's range and runs warm**, still
  climbing 1 K every 50 days at day 200.

**The second pass**, the same way: two pairs between S and T, and T under
the thermostat as decision 4 first wrote it.

| run | `cloud_albedo` | `cloud_greenhouse` | thermostat | net cloud, W/m² | whole surface, day 100 | day 200 | year 2 | trim, days 100 to 200 |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | --- |
| U | 0.3 | 45 | off | −26 | 15.6 °C | 17.2 °C | 16.6 °C | 1 |
| **V** | **0.3** | **40** | off | **−29** | 15.3 °C | 16.6 °C | **16.2 °C** | 1 |
| X | 0.25 | 50 | 0.01 a kelvin, 5 days | −8 | 16.3 °C | 14.1 °C | 15.1 °C | 0.81 to 0.87 |
| Y | 0.25 | 50 | 0.02 a kelvin, 20 days | −10 | 16.5 °C | 15.1 °C | 16.0 °C | 0.85 to 0.92 |

- **V is the pair** (decision 3): its clouds take 29 W/m², inside Earth's
  range, and it is the closest of those to 15 °C untrimmed. It is still
  warming, half a kelvin every 50 days at day 200, which is the thermostat's
  to hold.
- **The thermostat as first written rings.** X rose to 16.7 °C on day 80 and
  fell to 14.1 °C by day 200, and its trim swung from 1.06 to 0.81 and back
  toward 0.87. Y, slower, still peaked at 16.6 °C. Both break the ±1 °C pass
  mark from day 30 and the owner's "the sun just has a constant solar output"
  (K1): the trim moved 7% over year 2.
- **Why it rings.** The land answers a change of sun in days; the sea, with
  sixty times the heat capacity, in hundreds (`C_sea / olr_b` is 1.4 million
  seconds, 500 game days). An integral quick enough to catch the land keeps
  pushing long after the sea has been set on its way, and overshoots.
  Decision 4 is revised below.

**The last two passes (2026-09-27), with the pools' heat kept (finding 7)**,
level 4, 200 days, `solar_wm2` 1360:

| run | pair | thermostat | start | whole surface, day 30 | day 100 | day 200 | trim |
| --- | --- | --- | --- | ---: | ---: | ---: | --- |
| V0 | 0.3 / 40 | off | old climatology | 14.5 °C | 17.2 °C | 19.9 °C | 1 |
| S0 | 0.35 / 40 | off | old climatology | 14.1 °C | 16.1 °C | 18.0 °C | 1 |
| VB1 | 0.3 / 40 | balance, 0.01 a kelvin | old climatology | 13.2 °C | stopped at day 60, 13.6 °C | | 0.87 to 0.91 |
| VC2 | 0.3 / 40 | balance, 0.02 a kelvin | at the target | 14.9 °C | 15.1 °C | see below | 0.85 to 0.90 |
| VC4 | 0.3 / 40 | balance, 0.04 a kelvin | at the target | 14.9 °C | 15.1 °C | see below | 0.86 to 0.90 |

- **With the books closed, the in-range pairs run warm.** V untrimmed climbs
  past 19 °C and is still climbing. So the natural balance is not near 15 °C,
  as decision 2 hoped: the thermostat settles the sun at about 0.89 of 1360,
  a steady 1,210 W/m². K1 asks that it read as constant, and it does (below).
- **Started at the target, the balance thermostat holds.** VC2 and VC4 are
  within 15 ± 0.3 °C from day 20, with no overshoot. The nudge's strength
  barely matters once the start is right; 0.02 is shipped, the gentler.

**The shipped settings at level 5, 200 days (task 3.1, 2026-09-27).** Pair V
under the balance thermostat, at the level the game runs, from a new world:

| day | 10 | 30 | 60 | 100 | 150 | 200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| whole surface | 14.19 °C | 14.25 °C | 14.32 °C | 14.66 °C | 15.21 °C | 15.43 °C |
| sea surface | 16.99 °C | 16.79 °C | 16.64 °C | 16.99 °C | 17.36 °C | 17.63 °C |
| trim | 0.851 | 0.878 | 0.915 | 0.926 | 0.926 | 0.917 |
| net cloud, W/m² | −25.4 | −26.7 | −26.8 | −26.9 | −26.8 | −27.1 |

- **The mean holds within 15 ± 1 °C from day 30** (14.25 to 15.45 °C), which is
  task 3.1's pass mark. The clouds take 25 to 29 W/m², inside Earth's range.
- **The trim varies 1.3% over the second year** (0.917 to 0.929), over K1's
  1% mark (task 3.1a). The sea is still warming, 0.6 K across year 2, and
  the balance trim follows it down as it does. At level 4 the same settings
  held 0.7%. A 400-day run at level 5 is measuring whether it settles once
  the sea has; the mark is judged on that run's last year.

**Finding 8: balanced, the tropics are too cool for the reef (2026-09-27).**
The second-year fish test (task 3.2) fails on one species: the reef fish,
whose water is shallows that reach 23 °C. With the mean held at 15 °C, the
shipped `heat_spread` of 0.002 carries so much heat poleward that the
equatorial sea averages 19 °C, where Earth's is about 27 °C. Measured on the
shipped settings at level 4 (`check4`), and at a half and a quarter of the
spread (`H1`, `H05`), 200 days, all under the thermostat. The figures are
area-weighted over the second year, from `fish_ranges`' fields, and a
"shallow" pixel is sea within 3 m of the surface:

| run | `heat_spread` | whole surface, day 200 | sea 0-10°, mean | sea 70-80°, mean | shallows reaching 23 °C | sea that freezes some day | warmest shallows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| check4 | 0.002 (shipped) | 15.2 °C | 18.9 °C | 6.0 °C | 0.5% | 0.4% | 24.3 °C |
| H1 | 0.001 | 15.4 °C | 20.7 °C | 2.6 °C | 11.6% | 1.6% | 27.9 °C |
| H05 | 0.0005 | 15.2 °C | 22.1 °C | −0.8 °C | 25.5% | 2.4% | 30.7 °C |
| **balanced, level 5** | 0.002 (shipped) | 15.4 °C | 21.7 °C | | 18.0% | | 28.8 °C |

- **The thermostat holds the mean whatever the spread**: all three sit at
  15.2 to 15.4 °C on day 200, with the trim between 0.905 and 0.914.
- **A smaller spread makes a steeper world**, warmer tropics and colder
  poles, which is nearer Earth's. At a quarter of the spread, a quarter of
  the shallows can hold reef fish, and the polar seas freeze in winter.
- **The alternative is the fish's window.** The reef's 23 °C floor could
  drop to 19 °C and leave the gradient as it is. That keeps the climate
  mild everywhere, and a mild world has less to tell its biomes apart by.
- ~~This is the owner's to choose (survey R1).~~ **Withdrawn the same day,
  on the level-5 run (the last row).** At the level the game runs, the
  shipped spread already gives the reef water: 18% of the shallows reach
  23 °C and the equatorial sea averages 21.7 °C, within half a kelvin of H05.
  That is no coincidence. `Grid::conduct` pulls each cell toward its
  neighbours at `heat_spread` per second, whatever their distance, so its
  real strength, a diffusivity, is `heat_spread` times the square of the
  cell spacing. Each finer level halves the spacing and quarters the
  spread: level 5 at the shipped rate IS a quarter spread at level 4. The
  second-year fish test runs at level 3, where the same number spreads heat
  sixteen times harder than in the game, so it tests a flatter world than
  anyone plays in. Decision 7 makes the spread the same at every level.
  Survey R1 is withdrawn, since a measurement answered it.

**Finding 9: a new world starts below the target, most on a coarse grid
(2026-09-27, on decision 7).** With the spread the same at every level, the
level-3 run gives the reef its water, as level 5 does: the equatorial sea
averages 22.5 °C and 33% of the shallows reach 23 °C. But its whole surface
reads 13.4 to 13.7 °C for the first sixty days, reaches 14 °C on day 100 and
15 °C on day 180, so the fish test fails its day-30 check at 13.6 °C. Level 4
reads 13.8 °C on day 10. Level 5, the game's, reads 14.2 °C, inside the mark
but under the target for a hundred days. The start's climatology averages
the target, but its land cools to its own balance in days while the sea,
two degrees colder than it will settle, warms over hundreds. The balance
trim aims at a planet in equilibrium, which one still filling its sea is
not, and the integral that would close the gap is held while the error is
over `sun_trim_band_k`, 1 K.

The thermostat is not the lever. Swept at level 3, 200 days, on decision 7:

| run | change | day 10 | day 30 | day 100 | day 200 | from day 30 | trim, year 2 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| l3 | none | 13.47 °C | 13.37 °C | 14.05 °C | 15.01 °C | 13.37 to 15.05 °C | 1.0% |
| B2 | band 2 K | 13.48 °C | 13.49 °C | 14.70 °C | 15.62 °C | 13.49 to 16.08 °C | 2.8% |
| B3 | band 3 K | 13.48 °C | 13.49 °C | 14.70 °C | 15.62 °C | 13.49 to 16.08 °C | 2.8% |
| P5 | nudge 0.05 a kelvin | 13.69 °C | 13.79 °C | 14.85 °C | 15.17 °C | 13.79 to 15.56 °C | 3.3% |

The dip is set in the first ten days, before any of these acts, and a wider
band only overshoots later and makes the sun less steady. The fix is where a
new world starts (decision 8).

**8. A new world starts from a settled climate (the owner, survey K6,
2026-09-27: "why not start at 15c? why climb it back up?").** It did start at
15 °C on average, but as a climatology by latitude: land too warm, sea too
cold. The land cools to its balance in days, while the sea takes hundreds to
warm. So a new world starts from the settled planet instead, and its first
day is its settled one:
- `examples/settle_climate.rs` runs a new world forward and writes the
  atmosphere's saved state (`Atmosphere::to_bytes`, the weather save a world
  already keeps). It speeds the sea up tenfold for the first years (its
  heat capacity cut to a tenth, so its timescale is weeks, not a year and a
  half), then runs the last year at the true capacity so the seasons come
  back to their true size. The year's mean does not depend on how much heat
  the sea holds, only on how fast it gets there. This is the accelerated
  spin-up climate models use.
- It stops at a whole number of years, at the hour a new world's clock
  starts (`START_HOUR` of day 0), so the season it ships is the season a
  new world opens in.
- The state ships as `assets/climate/settled-l<level>.bin`, beside the RON of
  the settings it was made with. `Air::open` restores it for a world with no
  weather of its own, and falls back to today's spin-up where the file is
  missing or was made with other settings, saying so in the log. A test
  fails when the shipped settings and `atmosphere.ron` differ, so a retune
  cannot ship with a stale climate.
- Level 3's state ships too, for the second-year fish test, which starts
  from it as a new world does.

As built (2026-09-27): two fast years and one true one. At level 3 the fast
years swing between 13.4 and 16.5 °C with the light sea; after the switch
the true year reads 14.7 to 15.2 °C, and a new world's first sixty days from
the shipped state read 14.69 to 14.97 °C, the trim steady at 0.90 to 0.91.
Where a world's clock opens later than the state stands (a capture's
`--time`, a `--day`), `Air::open` steps the state to the clock when it is
within `spinup_s` of it, and otherwise runs the usual spin-up from the
settled state instead of from rest, so the air and ground come round to the
hour and the sea keeps its heat. The app's tests pin both: a new world at
the default clock opens on the shipped bytes exactly, and at noon it is
stepped there and still reads 15 ± 0.5 °C.

**7. The heat spread is a diffusivity, the same at every level (2026-09-27,
finding 8).** `heat_spread` (per second) is replaced by
`heat_diffusivity_m2s` (m²/s), and the step's rate is that over the square
of the grid's mean centre spacing, measured off `Grid::span`. Its default is
the shipped rate times the square of level 5's mean spacing (about 180 m,
so about 65 m²/s; the exact figure is measured and pinned by a test), so
the game at level 5 steps exactly as it does today. A coarser grid, which
the instruments and the tests run on, spreads heat as the game does rather
than four or sixteen times harder.
- *Alternative:* run the fish test at level 5. It takes about an hour, where
  level 3 takes a few minutes, and any other instrument at another level
  would stay wrong.
- *Alternative:* divide each edge by its own span squared. That is the
  finite-volume form, but it changes level 5 by the grid's ±9% distortion,
  and this change is not meant to move the game's climate at all.

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
