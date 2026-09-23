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
