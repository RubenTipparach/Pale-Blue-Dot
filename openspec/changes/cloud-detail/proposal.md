# Proposal: clouds everywhere the air is humid, with detail finer than a cell

## Why

The owner, looking at the planet from orbit after `atmospheric-circulation`
landed: *"why are the cloud patches not more detailed? they seem to just exist
near the equator"*. Both halves are true, and they have separate causes.

### Where: cloud only forms where air rises fast

In `step.rs::water`, vapour condenses only above a saturation that rising air
lowers: `saturation(air) * exp(-lift_saturation * rising)`. The climate report
(`examples/climate.rs`, level 5, day 0, `docs/screenshots/circulation-climate-day0.txt`)
shows why that is the only way cloud forms:

| Measured | Value |
| --- | ---: |
| Relative humidity, median / p90 / max | 0.52 / 0.67 / **0.80** |
| Lift, median / p90 | 0.10 / 1.5 m/s |
| Planet clear / fully covered | 69% / 12% |
| Partly covered (between) | 19% |

Still air never reaches saturation (the most humid cell anywhere is at 0.80),
so cloud needs rising air of about 0.6 m/s or more: the equatorial convergence
zone and the cores of seeded storms, and nowhere else. By latitude on day 0
(northern summer):

| Band | Cover | Earth, for comparison |
| --- | ---: | ---: |
| 10-20 N (the rain belt) | 0.54 | 0.6-0.7 |
| 40-60 N (summer storm track) | 0.03-0.06 | 0.6-0.7 |
| 20-40 N (subtropics) | 0.12-0.26 | 0.3-0.5, more over cool ocean |
| 40-60 S (winter storm track) | 0.31-0.35 | 0.8 |
| Whole planet | 0.196 (first estimated 0.31) | about 0.67 |

The atmosphere has no cloud that forms from humidity alone: no stratus deck,
no marine stratocumulus under the subtropical highs, and no broad cloudiness
along the storm tracks. It also has no convection from ground heating, which is
why land rain peaks at 08:00 local rather than in the afternoon, and why there
are no daytime cumulus fields over land.

Some of this was tuning. `atmospheric-circulation` first piled cloud up to 77%
full cover. The fix added subsidence drying, faster rain-out and a higher cover
threshold, and it overshot toward clear.

### Detail: the shader adds almost none inside a cloud

A cell is 181 m on a 4,800 m planet, 1/166 of the way round: about 240 km on
Earth. A weather system is a few cells, so the maps hand the GPU smooth blobs
two to six cells across. Everything finer is the shader's (`clouds.wgsl`), and
it adds little:

- **Three octaves of value noise**, with lattices of about 230, 86 and 38 m
  (300, 110 and 50 km Earth-equivalent). Earth seen from orbit shows structure
  down to a few kilometres.
- **The cover remap fills a covered area solid.** HZD's remap is
  `(shape*profile - (1-cover))/cover`. At full cover every lump of the noise is
  cloud, and at the storm's extinction it is opaque within tens of metres. So a
  fully covered area is uniformly white inside, and all the texture is in the
  thin band where the cover is partial. The cover is bimodal (69% clear, 12%
  full), so that band is narrow.
- **The detail is not sheared by the wind.** The noise drifts with a two-phase
  flow map, which moves it without stretching it. Streets, bands and the spiral
  arms of the reference photo come from detail being drawn out along the flow.
- **One kind of cloud.** A tall convective cloud and a flat deck are the same
  noise at different heights. There is no cellular, popcorn texture for
  cumulus, and no smooth sheet for stratus.

## What

Two halves, both measured before and after:

1. **The atmosphere forms cloud from humidity as well as from lift**, and
   ground heating drives convection:
   - a partial-cover scheme from relative humidity (Sundqvist 1989), so humid air
     is partly cloudy without rising;
   - a convective term from the ground being warmer than the air, so land clouds
     over by day and rains in the afternoon;
   - the moisture knobs retuned toward Earth's cover by latitude.
2. **The shader draws detail finer than a cell**:
   - more and finer octaves;
   - a covered area that is not uniformly opaque;
   - detail stretched along the wind at cloud height;
   - two textures chosen by the map: cellular for convective cloud (a tall
     `cloud_top`) and smooth for stratiform.

## Not in scope

- Resolution. Level 6 (40,962 cells) costs about four times a step, and the
  finding above is that the missing cloud is a missing mechanism, not missing
  cells.
- The jet's shape (a broad slab at its cap). That belongs to
  `atmospheric-circulation`.
