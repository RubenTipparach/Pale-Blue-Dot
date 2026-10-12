# Design

## Context

The wind at cloud height is not simulated. It is worked out each step from the
surface wind and the air's temperature (`atmospheric-circulation`, design
section 4, "The cloud-level wind"):

```text
upper = surface + cap( (thermal_wind / f) * n x grad(T) * taper )
f     = 2 * omega * max(|sin lat|, 0.25)            (floored at 14.5 deg)
taper = smoothstep(0.1, 0.35, |sin lat|)            (0 inside 5.7 deg, 1 beyond 20.5 deg)
cap   = 45 m/s (jet_max_mps)
```

The design names the floor. The taper came in with the code (523e435,
2026-09-23) and is written up here for the first time. Both exist because
the Coriolis parameter `f` goes to nought at the equator. Near there, the
thermal wind, which balances a temperature gradient against the spin, is not
defined. Divided by the true `f` it is 250 m/s at 0-5 degrees south and
2,700 m/s at 0-5 degrees north.

Cloud rides `lerp(surface, upper, cloud_steering = 0.7) * cloud_pace`, so
this wind is what moves cloud. It also feeds the Jet overlay, drawn on
0-45 m/s. Nothing else reads it.

## Measured (2026-09-29)

A scratch instrument, not committed, restored version 5's settled climate at
level 5 (10,242 cells, 181 m apart) and ran a day. Every two hours it
recorded the wind at cloud height, both as the model makes it and under each
candidate below, from the same surface wind and temperature. It checked its
own copy of the as-built rule against the model's `upper`: the zonal means
agree to 0.1 m/s. The mockup's Jet frames agree too. Their day mean is under
1 m/s within 6 degrees of the equator, and 30-42 m/s at 12-15 degrees.

**Why the band has a hard edge.** Just north of the band, the air cools fast
toward the pole, and the floored `f` is the smallest `f` anywhere. So the
uncapped thermal wind is 90-110 m/s. Its fade-in is multiplied in before the
cap, so it reaches the 45 m/s cap once the fade is a third of the way up.
The fade runs from 6 to 20 degrees, but the map shows it between 6 and 13.

Day-mean speed and its prograde part (positive blows the way the ground
turns: Earth's westerly), zonal means in 2.5-degree bands, m/s:

| band | as built | A | B | **C** | as built, prograde | C, prograde |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 25.0-27.5 N | 29.5 | 29.5 | 29.5 | 27.4 | 25.2 | 23.1 |
| 22.5-25.0 N | 36.4 | 36.4 | 36.4 | 29.6 | 31.8 | 25.4 |
| 20.0-22.5 N | 41.0 | 41.0 | 41.0 | 27.6 | 34.6 | 22.5 |
| 17.5-20.0 N | 42.7 | 41.1 | 40.8 | 22.1 | 33.9 | 16.1 |
| 15.0-17.5 N | 42.3 | 34.5 | 33.3 | 14.6 | 33.5 | 9.5 |
| 12.5-15.0 N | 35.0 | 23.0 | 22.5 | 10.4 | 8.9 | -3.0 |
| 10.0-12.5 N | 30.0 | 13.8 | 17.7 | 11.6 | -21.8 | -11.0 |
| 7.5-10.0 N | 14.5 | 4.9 | 11.7 | 9.5 | -13.1 | -9.4 |
| 5.0-7.5 N | 1.1 | 0.6 | 8.2 | 8.1 | -0.7 | -8.1 |
| 0-5 N | 0.4-0.5 | 0.4-0.5 | 8.0-8.1 | 8.0-8.1 | 0.0 | -8.0 |
| 0-5 S | 0.5 | 0.5 | 8.3 | 8.3 | -0.3 | -8.3 |
| 5.0-7.5 S | 0.6 | 0.6 | 8.2 | 8.3 | -0.2 | -8.3 |
| 7.5-10.0 S | 3.6 | 3.0 | 6.0 | 7.3 | 2.0 | -7.2 |
| 10.0-12.5 S | 11.0 | 9.3 | 7.6 | 5.6 | 5.5 | -4.9 |
| 12.5-15.0 S | 18.8 | 16.4 | 14.7 | 6.6 | 10.0 | -1.8 |
| 15.0-17.5 S | 20.6 | 19.3 | 18.6 | 8.7 | 11.6 | 1.0 |
| 17.5-20.0 S | 22.4 | 22.1 | 21.9 | 11.7 | 15.1 | 5.7 |
| 22.5-25.0 S | 18.1 | 18.1 | 18.1 | 14.6 | 13.6 | 10.5 |
| 27.5-30.0 S | 19.9 | 19.9 | 19.9 | 19.7 | 15.7 | 15.5 |

Poleward of 30 degrees the candidates do not differ from what is built.

The steepest change in zonal-mean speed between neighbouring bands, within 35
degrees of the equator, in m/s per degree: as built **6.2** (at 10 N), A 4.6,
B 4.3, **C 3.0**. The day-mean maps are drawn side by side in
`docs/screenshots/tropical-upper-wind/options.png`, one per candidate.

## The candidates

- **A. Cap before the fade.** `cap(thermal) * taper`. The fade shows at its
  full width, 6-20 degrees, so the edge is half as steep. The band stays calm
  (0.5 m/s).
- **B. A, and easterlies aloft.** Inside the band, the wind at cloud height
  leans to a westward wind of `tropical_easterly_mps` (8 m/s), weighted by
  `1 - taper`: `upper = surface + cap(thermal) * taper - east * easterly *
  (1 - taper)`. The band blows at 8 m/s. Where the easterly turns into the
  jet, about 12 degrees north, a thin line of slower wind is left. That is
  where the wind changes direction, as it does on Earth. The jet's red edge
  at 15 degrees north is still sharp.
- **C. B, with the jet fading in across the tropical cell** (recommended).
  The fade runs from 6 to 30 degrees (`smoothstep(0.1, 0.5, |sin lat|)`)
  instead of 6 to 20. 30 degrees is where the model's own pressure belts put
  the subtropical highs (`belt_pressure` peaks at 30 degrees), which is the
  tropical cell's edge. It is also where Earth's subtropical jet sits. The
  northern jet's zonal-mean peak moves from 17-20 degrees to 22-25, and
  falls from 43 m/s to 30. The equator blows east to west at 8 m/s.
  Everything poleward of 30 degrees is unchanged.

## Decisions

**1. Candidate C (survey W1).** The owner, 2026-09-29: "are the jets fixed?
just do recommended".
- It is the only candidate whose edge is gentler than 3 m/s per degree.
- It is the only one that moves the northern jet off the equator's edge.
- *Cost:* the northern subtropical jet is a third slower. Cloud crossing 15-25
  degrees north drifts about a third slower with it.
- *Alternative:* B, which keeps the northern jet's speed and position, and
  keeps its hard edge.
- *Alternative:* A, the smallest change, which keeps the calm band.

**2. The easterly is a setting, 8 m/s.** It is named `tropical_easterly_mps`
and its unit is m/s. It is validated finite and between 0 and `jet_max_mps`.
Earth's zonal-mean wind aloft over the equator is 5-10 m/s from the east.
8 m/s drifts tropical cloud at about 1.1 m/s (`cloud_pace` 0.2 times
`cloud_steering` 0.7 times 8), up from 0.1 m/s. Zero turns the easterlies
off and leaves candidate A's calm band.

**3. The floor on `f` stays at 14.5 degrees.** The fade already switches the
thermal wind off where the floor matters. Moving the floor as well would
change the jets poleward of 30 degrees, which are right as they are.

**4. The settled climates are made again, for every carried generator.**
They are what a new world's weather starts from, and cloud now drifts
differently in the tropics. The gate is the same one they already pass: a new
world's days hold 15 ± 0.5 °C. They are not the ground, so the rule that
every generator version keeps its ground cell for cell does not apply to
them. A world already made keeps its own saved weather.

**5. Saved worlds carry on.** The wind at cloud height is not saved. It is
worked out afresh each step. An old world opens as it was saved, and from its
next step its tropical cloud drifts on the new wind. Nothing is reset or
dropped.

## Built (2026-09-29)

- `aloft` caps the thermal wind, then fades it with
  `smoothstep(0.1, 0.5, |sin lat|)` (`JET_FADE_SIN_LAT`), and adds
  `tropical_easterly_mps` (8 m/s) westward, weighted by what the fade leaves.
  The floor on `f` is unchanged.
- On the shipped level-5 climate, one step's cloud-level wind within 5
  degrees of the equator blows 8.0 to 8.3 m/s from the east (the rule it
  replaced: 0.4 to 0.5). The steepest change between 2.5-degree bands within
  35 degrees is 3.2 m/s a degree, or 2.9 with no easterly. The old rule
  measures 6.0 on the same state. `the_tropics_blow_easterly_aloft_and_the_jet_has_no_edge`
  and `with_no_easterly_the_tropics_aloft_have_the_surface_wind` pin both,
  and both fail on the old rule.
- **Climate report**, before and after, 10 days from rest at level 5 on day
  0. Zonal means by 10-degree band, the jet prograde in m/s:

  | band | before | after |
  | --- | ---: | ---: |
  | 30-20 N | 35.1 | 29.2 |
  | 20-10 N | 34.3 | 7.9 |
  | 10-0 N | -1.4 | -8.2 |
  | 0-10 S | 1.3 | -7.8 |
  | 10-20 S | 25.9 | 6.6 |
  | 20-30 S | 34.3 | 28.9 |

  Poleward of 30 degrees every band is within 1 m/s. The cloud barely
  moves. Cover is 0.508 before and 0.506 after, clear sky 25.1% and 25.7%,
  full cover 39.2% in both, rain 20.9% in both. The band spread is 0.251
  and 0.248, and the whole surface is 14.24 °C and 14.27 °C. Rain at 10-20
  N rises from 35.3 to 38.9 mm/h, and at 0-10 N falls from 32.5 to 29.8. The
  `atmospheric-circulation` claims hold. The zonal-mean jet's maximum, at
  20-30 N and 30-40 S, lies poleward of its minimum, now the equatorial
  easterly.
- **Old saves.** Four weather states made before the change, the settled
  climates of generators 4 and 5 at levels 3 and 5, were checked with a
  scratch instrument. Each restored byte for byte, and ran 600 steps on with
  every field finite and the surface at 14.9 to 15.1 °C. Each blew 8.1 to
  8.2 m/s aloft within 5 degrees of the equator.
- **The settled climates, as a quick fix.** The owner, 2026-09-29, while the
  level-5 states were being remade: "hours? no dude, I just wanted the jet
  stream quick fix! dont need to simulate a whole day lol", then "we can run
  this sim on my desktop later!". So decision 4 is done in part:
  - Generators 4 and 5 at level 3 are remade under the new wind. A new
    world's first 60 days hold 14.73-15.09 °C (generator 4) and 14.71-15.01 °C
    (generator 5).
  - The level-5 states, the game's own level, are kept as they were, settled
    under the old wind. The climate report above is why that is safe for now:
    the new wind moves the planet's mean by 0.03 °C and its cover by 0.002.
    The owner remakes them on the desktop with
    `cargo run --release -p pbd-core --example settle_climate -- 5 2 30 assets/climate <generator>`,
    for generators 4 and 5.
  - The check that a shipped state was made with the running settings cannot
    see this. It compares the settings, and the new knob reads as its
    default in a file that predates it. A stricter check that refused such
    files was written and taken out again, because it would have refused the
    kept level-5 states and started new worlds from rest. It can come back
    once they are remade.
- `settle_climate` takes the generator as a fifth argument, so generator 4's
  states can be remade.
- The map mockup's weather frames were made again (`map_weather 20`, the
  day after 20 from rest). The Jet overlay's day mean is 8.5 m/s at the
  equator, 4-5 m/s where the easterly turns into the jet near 10 degrees,
  and 21-22 m/s by 20 degrees north and south.

## Risks

- [The tropics' cloud moves, so their cover and rain move a little] →
  the climate report (`examples/climate`) is run before and after, and the
  `atmospheric-circulation` claims are checked again. One of those claims:
  the cloud-level wind's zonal-mean maximum lies poleward of a subtropical
  minimum. It holds for every candidate, since the minimum is now at 10-15
  degrees.
- [A per-cell multiply-add in a stage that runs every step] → the frame
  cost is not measured in a cloud session. The owner runs
  `tools/perf_suite.py`.
