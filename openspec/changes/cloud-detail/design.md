# Design: cloud detail

## 1. Cloud from humidity (pbd-core, `atmosphere/step.rs`)

**Partial cover.** The cell's cover becomes the larger of two numbers:
- the condensed-water cover that exists today (`cover_of(cloud)`);
- a humidity cover, `1 - sqrt((1 - rh) / (1 - rh_crit))` for `rh > rh_crit`
  (Sundqvist 1989, the scheme GCMs use for sub-grid cloud), where `rh` is the
  vapour over the saturation at the cell's air temperature.

The humidity cover is what a satellite sees of a humid air mass that is not
rising: a deck, a haze of small cumulus. It rains nothing by itself. Rain stays
with the condensed water, so the water budget is unchanged.

`rh_crit` is a knob in `atmosphere.ron`, starting at 0.6. On today's humidity
(p90 0.67, max 0.80) that gives at most 0.29 cover, so part of the work is
letting the air get more humid:
- slower subsidence drying, or drying only above a threshold of sinking;
- evaporation over the sea relaxed toward a higher humidity.

Each moves on the climate report, not by eye.

**Convection from heating.** `rising` gains a buoyancy term,
`convective_mps_per_k * max(ground_k - air_k - convective_threshold_k, 0)`. The
ground outruns the air by day over land, because land's heat capacity is small,
so land lifts in the afternoon and a cumulus field forms and rains. It also
charges the column, so afternoon thunderstorms follow from the lightning rule
already in place. Two knobs, with units.

**The map.** `Sample` gains the humidity cover, and `cover` becomes the combined
one. `cloud_top` stays the convective measure. A deck is low and flat; a
convective cell towers.

## 2. Detail in the shader (`clouds.wgsl`)

- **Finer octaves.** Five octaves, not three, the finest near 8 m (about
  10 km on Earth), with the fine ones only eroding edges (HZD's detail
  erosion), so they cost little inside a solid region.
- **A full cover is not uniformly opaque.** The remap's floor is lifted, so at
  full cover the noise's troughs thin rather than fill:
  `(shape*profile - (1-cover)*a - b)`, with `b` a knob. That leaves lanes and
  thin patches inside a system, which is what a storm's texture from orbit is.
- **Sheared by the wind.** The noise is sampled in a frame stretched along the
  wind at cloud height, by a factor that grows with the speed. Fast flow draws
  detail into streets and bands, and it curls round a low. That is one more
  wind read per sample, which the flow map already makes.
- **Two textures.** A cellular (Worley) term, weighted by `cloud_top` (the
  convective measure), gives cumulus its cells and gaps. Low `cloud_top` keeps
  the smooth value noise of a deck.

## 3. Measurement

**Before any change, recorded here:**
- the climate report's cover by band, clear, full and partial shares, and the
  local hour of land's rain peak;
- from an orbit capture, the brightness spread of cloud pixels inside a fully
  covered area (standard deviation, 0..1), with the pixels chosen from the
  map's cover, not by eye.

**Targets:**
- whole-planet cover 0.5-0.65;
- the summer storm track (40-60 degrees) at least 0.3;
- partial cover at least 30% of the planet;
- land rain peaking between 13:00 and 17:00 local;
- a covered area's brightness spread at least twice the baseline.

Frame cost is A/B'd on llvmpipe (relative only), and the step's cost is
reported by the climate report.

## 4. What was built, and what the measurements changed

### The baseline (day 0, one day, level 5)

The proposal estimated whole-planet cover at 0.31. The report, once it printed
the number, said **0.196**: 18.6% of the planet partly covered, **28.8% of it
raining at once**, and humidity never above 0.80.

### Three findings that changed the plan

- **Ground minus air never fires.** Over land the ground is colder than the
  air at every hour: -3.6 K at 06:00 and -1.0 K at 15:00. Land air is mostly
  warm air off the sea, and the land loses heat to evaporation. So convection
  is driven by the sunlight the LAND absorbs, above a threshold
  (`convection_mps_per_wm2`, `convection_threshold_wm2`). The sea is left out,
  because its heat goes into the water. Land lift now peaks at noon: 1.28 m/s,
  against 0.36 at night.
- **The rain was the other half of the missing cloud.** Cloud rained once it
  held 0.3 kg/m², about its median. So drizzle fell under a third of the
  planet, and the water rained out before it could build a deck. Three rules
  replace the single threshold:
  - a warm cloud rains past `rain_threshold_kg` (4.0);
  - a cold one rains past less, in proportion to what the air holds below
    15 C. Ice grows at the droplets' expense; without this, polar cloud never
    rained or dried and piled up over days;
  - a vigorous updraft rains past less again: the threshold halves at
    `convective_rain_mps` of ascent.

  So storms and afternoon cumulus rain, and a flat deck mostly does not.
- **One critical humidity let the land out-cloud the sea.** Earth has it the
  other way round (about 0.72 over ocean, 0.55 over land), and the circulation
  spec holds it. The decks that humidity alone makes are mostly marine, so the
  land's threshold is higher: `humid_cover_rh_land` 0.7 against the sea's 0.45.

One more fix fell out of the tests. The save did not hold the mesoscale noise,
which only mattered while condensation was rare. `restore` now re-derives it
as of the step it was last refreshed at.

### After

| | Baseline, day 0 | Day 0, 1 day | Day 50, 3 days | Target |
| --- | ---: | ---: | ---: | ---: |
| Whole-planet cover | 0.196 | 0.506 | 0.559 | 0.5-0.65 |
| Partly covered | 18.6% | 54.3% | 55.5% | at least 30% |
| Raining at once | 28.8% | 17.0% | 17.9% | fewer (Earth ~10%) |
| Sea over land | 0.21 > 0.18 | 0.53 > 0.47 | 0.57 > 0.54 | sea higher |
| Summer storm track | 0.03-0.06 | 0.28-0.33 | 0.47-0.50 | at least 0.3 |
| Land rain, afternoon vs before dawn | 5 vs 4% an hour | 30% vs 21% | 23% vs 26% | afternoon higher |

**Not met: the afternoon rain.** The design first asked for "a peak between
13:00 and 17:00". The hourly profile is too flat (3-7% an hour) for an argmax to
mean anything, so the report now gives the afternoon's share against the small
hours instead. Convection moves day 0's land rain into the afternoon, but on
day 50 more still falls before dawn, at every convection strength tried from
0.01 to 0.04. That rain is not convective: it looks like air over land cooling
below saturation at night. So the requirement's scenario is met at one solstice
and not the other, and it stays unmet.

Lightning roughly doubled, from 6,300 to 11,000 strikes a day, because more
cloud is convective. Each strike's cold pool is small, and the circulation's
numbers above did not move with it.

### The shader, and what the captures changed

**The A/B.** Every term is a knob, so "before" is the same build with the knobs
neutral (floor 0, erosion 0, shear 1, cells 0) over the same new atmosphere.
Captures are from orbit, day 0, at a fixed step. The measure is
`tools`-less for now: a scratch script that takes cloud pixels (bright and
grey), erodes that mask by 6 px to keep only interiors, and reports the
standard deviation of their luminance.

Four things went wrong on the way, and each changed the build:

- **The shear did nothing.** Squeezing the noise coordinate along the wind is
  `q - along * dot(q, along)`. But the coordinate is the point's direction and
  the wind is tangent to it, so the dot product is nought everywhere. It is a
  comb now: each point is pushed along the wind by a smooth noise of where it
  is, so bands slide past one another along the flow.
- **Worley creases.** A hard-minimum F1 has a ridge wherever two cells meet, and
  the cover remap sharpened every ridge into a thin dark line. It is a smooth
  minimum now (a log-sum of exponentials).
- **The march jitter was not per pixel.** It was hashed off the ray direction
  with a hash made for integer lattices. On a smooth direction, neighbouring
  pixels got nearly the same offset, and the steps' banding survived as smooth
  contours. It is interleaved gradient noise on the pixel now, passed in by the
  clouds pass.
- **The fine octaves aliased.** From orbit a pixel spans about 14 m of cloud,
  and the finest octave is 9 m. Drawn regardless, the erosion aliased into
  fine wavy lines across every mass, and that was most of the texture the
  first capture measured (spread 0.041 against 0.021). Each octave now fades
  out between three pixels and one and a half of its own wavelength, using the
  pass's `fwidth` of the ray direction as the angle a pixel spans.

**After** (floor 0.45, erosion 0.5, shear 2, cells 0.6):

| From orbit | Neutral knobs | Shipped knobs |
| --- | ---: | ---: |
| Interior luminance spread | 0.021 | 0.026 |

**Not met: the 2x interior target (1.24x).** From orbit the big storm masses
stay smooth: the octaves that would texture their interiors are finer than a
pixel at that range, and they must fade or alias. What changed is at the
scales the eye reads from orbit, which the interior measure leaves out:
- lanes and combed streets open between and inside systems;
- edges break into wisps;
- a full deck is no longer one solid sheet.

The fine erosion and the cells show close up, where a pixel is small.

**The weather slider broke and was fixed.** Its storm only added cloud water
(1.2 kg/m² at full), which rained under the old 0.3 threshold. Under 4.0, the
menu's RAIN preset (0.6) gave full cover and no rain. The forcing now also
drives an updraft (`forcing_lift_mps`, 3 m/s at full), so the slider's storm
rains by the same convective rule as any storm. A test pins both halves: it
rains with the updraft, and not without it.
