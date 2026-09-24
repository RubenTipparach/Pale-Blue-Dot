# Design: fair-weather clouds

## 1. Coverage that tracks the cover (`clouds.wgsl`)

**Equalize the noise.** Map the shape through an approximation of its own
distribution, so that `shape_eq` is roughly uniform on 0..1. A smoothstep
between the measured p1 and p99 (0.22 and 0.77) is the first candidate. The
instrument (`tools/drawn_cover.py`, transcribing `cloud_shape` and the vertical
profile) then checks the drawn share against the cover at 0.05, 0.1, 0.2, 0.3,
0.5 and 0.7.

- **Target:** within 0.05 of the cover at each point.
- **The column question.** A column is drawn when ANY height in it passes the
  threshold, so the column's maximum is what gets equalized. The instrument
  measures that maximum (p50 0.60, p99 0.79), not the single sample.

**Small clouds at low cover.** Below `fair_cover_max` (0.15) the shape takes
more of its finest octave, so the peaks the remap keeps are 30-80 m across
(cumulus) rather than 230 m (a system's lumps). The fine octaves fade by the
pixel footprint as now, and the deck floor does not apply below that cover.

**What does not change.** Cover above about 0.7 still fills the sky. Lighting,
the march and the maps are untouched.

## 2. Fair-weather cover (`pbd-core`, `atmosphere`)

`cover(i)` becomes the largest of three things:
- the condensed-water cover;
- the humid (Sundqvist) cover;
- a fair-weather cover, `fair_cover_max * smoothstep(fair_rh, critical, rh)`.

The fair-weather cover runs from nothing at `fair_rh` (0.4) up to its cap at
the place's critical humidity. Above the critical humidity the Sundqvist cover
takes over. It is continuous at that point only if the cap is not higher than
the Sundqvist cover there, which is nought, so the combined cover plateaus at
the cap until Sundqvist passes it. That plateau is the intent: fair weather
reads as scattered cumulus up to the deck threshold.

It has no optical depth to speak of (`water_for_cover` of 0.15) and it
condenses nothing, so the water budget, the rain and the lightning are
unchanged. The sea keeps its lead over the land, because both get the same
floor and the sea's deck cover sits above it.

Knobs, in `atmosphere.ron` with units and validation: `fair_cover_max` (0..1)
and `fair_rh` (0..1, below both critical humidities).

## 3. Measurement

**Before, recorded above:** the cover histogram, and the drawn-share table.

**After:**
- the climate report's histogram, with truly bald cells under 1% of the planet;
- `tools/drawn_cover.py`, with the drawn share within 0.05 of the cover;
- the sea still cloudier than the land at both solstices;
- the raining share unchanged within 0.5 points;
- orbit captures at both solstices against the current ones.
