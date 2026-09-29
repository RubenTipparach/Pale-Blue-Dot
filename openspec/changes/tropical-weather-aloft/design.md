# Design

## Context

As built by `tropical-upper-wind` (decision 1, candidate C):

```text
upper   = surface + fade * cap(thermal) + (1 - fade) * easterly
thermal = (thermal_wind / f) * n x grad(T)
f       = 2 * omega * max(|sin lat|, 0.25)       (floored at 14.5 deg)
fade    = smoothstep(0.1, 0.5, |sin lat|)         (5.7 to 30 deg)
cap     = 45 m/s; easterly = 8 m/s westward
```

Cloud rides `lerp(surface, upper, cloud_steering = 0.7) * cloud_pace`. The
Jet overlay draws `|upper|` on 0-45 m/s. Nothing else reads it.

## Measured (2026-09-29)

**The owner's picture, in numbers.** The mockup's twelve Jet frames store the
speed as a palette index (`round(v / 45 * 254)`), so they read back exactly:
- Within 6 degrees of the equator, the speed is 8.4-8.6 m/s. Its spread round
  the planet is 0.4 m/s.
- The slowest point between 4 and 25 degrees, column by column, is at 11.9
  degrees north and 11.8 degrees south.
  - Its spread round the planet is 0.7 and 1.0 degrees.
  - It blows 1.3 and 1.9 m/s there.

**The instrument.** A scratch instrument (not committed, as for
`tropical-upper-wind`) restored generator 5's level-5 settled climate and ran
a day. Every two hours it worked out the wind at cloud height, as built and
under each candidate, from the same state. It drew each on the map's raster,
and dumped every cell's terms. Its copy of the built rule agrees with the
model's `upper` to 0.02 m/s in every 2.5-degree zonal mean.

**Why the band is straight.** Three causes, all measured:

1. **Inside about 8 degrees, the only wind is the constant.**
   - The surface wind there averages 0.3-0.5 m/s, and the jet is faded out.
   - What is left is the 8 m/s easterly: a constant times a function of
     latitude.
2. **The jet is pinned at its cap.** Equatorward of 20 degrees, `f` is small
   and floored, so the thermal wind is worked out at 1-5 times the cap.
   - Its median is 30-115 m/s, and its 90th percentile reaches 248 m/s.
   - 49-58% of cells within 5-20 degrees of the equator are at 45 m/s.
   - Capped, then faded, the jet there is `45 * fade(lat)`, which depends on
     latitude alone.

   The easterly is `8 * (1 - fade(lat))`. Where they cancel, `8 (1 - w) =
   45 w` gives `w = 0.15`, which is 11.4 degrees. That is where the mockup's
   lines are.
3. **The model's tropics hardly vary round the planet.**
   - The surface wind is under 1 m/s from 20 degrees south to 20 north.
     Nowhere on the planet is its band mean above 3.7 m/s. The only trade
     winds are 1.5-1.7 m/s from the east at 20-40 degrees north.
   - On day 0, northern summer, the warmest 2.5-degree band is at 11-16
     degrees north in 35 of 36 samples (18 sectors of 20 degrees, 2 hours).
   - The air's temperature varies round the planet by 1-3 K at those
     latitudes.

   So anything worked out from the tropics' weather comes out zonal. In the
   northern summer a second calm line lies along the warm ridge. There the
   capped jet flips from 45 m/s one way to 45 the other within a cell or two.

Tried and dropped:
- **A soft cap** (`45 tanh(v / 45)`) moved nothing by more than 1.3 m/s. The
  uncapped wind is too far past the cap.
- **Reversing the surface wind's north-south part aloft** (the Hadley cell's
  upper branch) moved nothing, because the surface wind there is under 1 m/s.
- **An outflow from the gradient of lightly smoothed rising air** was
  speckled at cell scale. H solves for the outflow's potential instead.

## The candidates

Drawn from the seventh of the twelve frames, one above the other, in
`docs/screenshots/tropical-weather-aloft/options.jpg`: as built, D, F, H.

- **D. The jet follows the air.** `cap(fade * thermal)`, with `f` floored at
  `|sin lat| = 0.5` (30 degrees) instead of 0.25.
  - Poleward of 30 degrees the floor does not bind and the fade is 1, so
    nothing there changes.
  - Equatorward, the jet takes the shape of the air's temperature and is
    rarely at its cap.
- **F. D, and the storms' outflow.** The tropics' own wind becomes `easterly
  + grad(chi)`, weighted by `1 - fade` as the easterly is now.
  - `chi` is the potential with `laplacian(chi) = lift - mean(lift)`, so its
    gradient blows away from where air rises.
  - It is scaled to 5 m/s rms within 25 degrees, and the easterly drops to
    6 m/s.
- **H. F, and the wind round the storms' high** (recommended). It adds
  `n x grad(-chi) * f / (f^2 + f0^2)`, scaled to 8 m/s rms, with `f0` at
  about 10 degrees.
  - `-chi` is highest over the storms: the high they build aloft.
  - The wind circles it the way it circles warm air: anticyclonic in both
    hemispheres, through nought at the equator with no jump.

A day of 12 frames, on the map's raster:

| | as built | D | F | **H** |
| --- | ---: | ---: | ---: | ---: |
| 0-10 deg: mean speed, m/s | 8.2 | 8.4 | 9.6 | 11.5 |
| 0-10 deg: spread round the planet, m/s | 0.4 | 0.6 | 5.6 | 6.3 |
| North: dark line (under 4 m/s), share of the way round | 49% | 57% | 55% | 37% |
| North: its latitude's spread, deg | 1.3 | 1.3 | 3.5 | 1.5 |
| South: dark line, share of the way round | 85% | 88% | 67% | 54% |
| South: its latitude's spread, deg | 2.6 | 2.3 | 3.4 | 4.3 |
| Jet at its cap, 5-20 deg (cells) | 49-58% | 0-14% | 0-14% | 0-14% |
| Steepest day-mean change between 2.5-deg bands within 35 deg, m/s a deg | 3.0 | 3.9 | 3.8 | 3.7 |
| Largest zonal-mean change poleward of 30 deg, m/s | - | 0.05 | 0.05 | 0.05 |

The last row's 0.05 m/s is the raster's interpolation across 30 degrees. Cell
for cell, nothing poleward of 30 degrees differs.

**What H does not do.** Over the quiet half of the planet, which is the
western half in these frames, the tropics have little rising air. There the
line stays straight at about 13 degrees north. Nothing worked out aloft can
make it wander where the weather below does not. That is survey W3.

**Not addressed.** Poleward of 30 degrees the jet is at its cap over wide
areas: the flat red of the southern jet. That is the same `thermal_wind`
against `jet_max_mps` balance. The owner has not asked about it, and none of
these candidates touches it.

## Decisions

Pending or decided, on the survey:
- **W2: which rule.** Recommended: H. Alternatives: D, the smallest change,
  which fixes the jet's edge and leaves the band flat; A, keep what is built.
  Chosen: H (below).
- **W3: write up the tropics' surface weather next?** This is why the trade
  winds are 1.5 m/s at most and the warmest air sits on one line of latitude.
  It is the cause under this band, and a bigger change to
  `atmospheric-circulation`. **Decided, 2026-09-29:** "recommended". So it is
  written up as its own change, a write-up only, after W2 is built.
- **W2, decided 2026-09-29:** "H". The owner first wrote "looks good now";
  asked back which rule that meant, they answered "H".

H is chosen, so:
1. **The potential is relaxed, not solved, each step.** Each step runs a few
   Jacobi sweeps, warm-started from the last step's `chi`.
   - The sweep count is a validated setting. The instrument used 3,000 sweeps
     from nought to draw a converged field.
   - How many sweeps a step needs to track the storms is measured before
     building. The test is that the day-mean outflow is within 10% of the
     converged one.
   - `chi` is not saved. A world opened from a save relaxes it from nought
     over its first steps.
2. **Two settings, in m/s:** `tropical_outflow_mps` (5) and
   `tropical_circling_mps` (8). Each is validated finite, 0 to `jet_max_mps`,
   and 0 turns its term off. They scale the terms directly, not an rms, so the
   game's wind does not depend on the whole planet's state. The instrument
   scaled each term to an rms within 25 degrees. The build measures the
   constant that gives that rms on the settled climate, and ships it.
3. **The easterly's default moves from 8 to 6 m/s.** With the storms' terms,
   the band's mean is about 11 m/s, against 8 as built.

## Risks

- **The edge limit.** D, F and H steepen the day-mean edge to 3.7-3.9 m/s a
  degree on the raster's bands, past the 3.5 the requirement holds. The build
  widens the fade's outer edge first (30 to 35 degrees). It moves the number
  only if that fails, and asks the owner before it does.
- **Cloud in the tropics drifts differently.** The climate report is run
  before and after, and the `atmospheric-circulation` claims are checked
  again. Tropical cloud will spread out from its storms.
- **Frame cost.** A few relaxation sweeps and two gradients per cell per step
  are added. This is not measured in a cloud session. The owner runs
  `tools/perf_suite.py`.
- **It is still partial.** On the quiet half, H leaves a straight line (see
  "What H does not do"). If the owner judges that still unnatural, the answer
  is W3, not a stronger tropical term. A stronger term would be the tropics
  aloft running on its own rules again.
