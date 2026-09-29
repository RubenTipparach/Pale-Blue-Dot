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

**1. Candidate C, provisionally (survey W1).**
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
