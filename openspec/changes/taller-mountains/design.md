# Design

## Context

The owner, 2026-09-28: "can you make mountains taller". Today's ranges are
the reference's ridged field, 686 m across (matched to the reference by slope),
multiplied by the continent so they stand inland, and weighted 0.8 in a
normalised height worth 210 m a unit. A regional uplift (`rocky_*`) was meant
to lift whole mountain countries 60 m, and the Mountains biome is the land
above 105 m.

## Measured: today's relief (2026-09-28)

A scratch probe (`scratchpad`, not committed) sampled 200,000 to 300,000
directions of version 5 (whose heights are version 4's to the bit):

| | today |
| --- | ---: |
| summit | 156 to 160 m |
| median land | 43.9 m |
| 90th / 99th percentile of land | 90 / 122 m |
| land above 120 m | 1.3% |
| Mountains biome (above 105 m) | 2.0% of land |
| step to the next cell over land above 120 m, median / 90th / 99th | 0 / 1 / 2 m |
| the ridge term alone, median / 99th / max | 19 / 66 / 85 m |

**The regional uplift almost never fires.** Its field (`rockiness`) reads
0.506 at the median of the land, 0.699 at the 99th percentile and 0.750 at
most, and the uplift starts at `rocky_above` 0.72. Tripling `rocky_uplift_m`
from 60 to 180 m moves no number in the table.

## Measured: the candidates

Each is today's generator plus one change, measured the same way. A step of
3 m or more between neighbouring cells is a wall the walker cannot climb
without digging or jumping more than once.

| candidate | median land | 99th pct | summit | land over 200 m | Mountains biome* | high cells stepping 3 m or more |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 43.9 | 122 | 160 | 0% | 2% | 0% |
| ridges twice as high | 60.1 | 178 | 238 | 0.2% | 12% | 0.5% |
| ridges twice as high and wide | 60.0 | 182 | 232 | 0.3% | 12% | 0% |
| ridges three times as high and wide | 78.4 | 240 | 295 | 4.6% | 22% | 0% |
| all land twice as high (`land_scale_m` 420) | 83.7 | 244 | 313 | 5.7% | 25% | 0.3% |
| **peaks:** high ridges lifted by `(r / 60 m)^2` | 45.5 | 173 | 314 | 0.3% | | 9.5% |
| **peaks:** the same, knee 50 m | 46.2 | 195 | 381 | 0.9% | | 14.8% |
| **ranges:** a ridge field three times as wide, three times the ridge's weight, in the rockiest quarter of the land | 47.3 | 186 | 291 | 0.6% | | 0.2% |
| **ranges:** the same, four times as wide and high | 48.1 | 180 | 245 | 0.4% | | 0.4% |
| **ranges:** three times, in the rockiest tenth | 44.9 | 140 | 222 | 0% | | 0.1% |

\* at today's 105 m threshold, which moves with the heights in every candidate
that is built (decision 3).

What the table says:
- **Scaling the ridges or the land raises everything.** The ridge term is
  multiplied by the continent everywhere inland, so doubling it lifts the
  median land 16 m and tripling it 34 m, and the Mountains biome eats the
  fields, desert and jungle that `bigger-biomes` just balanced.
- **Lifting only the high ridges makes walls.** The ridged field is 686 m
  across, so a summit three times as high on the same footprint is three
  times as steep: a tenth to a seventh of the high ground steps 3 m or more.
- **Wide ranges in regions of their own do both.** A second ridge field three
  times as wide, added only where the rocky field is high, reaches 291 m,
  leaves the median land within 3.4 m of today's, and keeps the high ground
  as walkable as today's (0.2% of high cells step 3 m or more).

## Decisions (provisional until the owner answers H1 to H3)

**1. A range term in regions of their own.** Version 6 adds to the altitude a
ridge field `range_widen` times as wide as the mountain field, weighted
`range_height`, eased in where `rockiness` passes `range_from` over 0.1 of it.
It replaces the regional uplift, whose threshold its field never reaches.
Recommended: three times as wide, three times the weight, in the rockiest
quarter of the land (`range_from` 0.567): the 291 m row.
- *Alternative:* peaks, for jagged summits over 300 m, at the cost of cliffs
  on a tenth of the high ground (survey H3).
- *Alternative:* scale the ridges or the land, which is one number but moves
  the lowland and the biome shares.

**2. A new version, 6, not a change to 5.** Version 5 is in a draft PR (#18)
with its settled climates being made. Folding the ranges into it would hold
that PR for another three hours of settling and remake climates already
made. Version 6 lands in its own PR; worlds made on 5 keep their ground
(survey H2).

**3. The biome and the snow follow the heights.** The Mountains biome
threshold, the snowcap, the snow line and the stone line move up with the
ranges, so the tall ground reads as rock and snow, and the Mountains biome
stays a few percent of the land rather than taking a fifth of it. The exact
values are measured when the candidate is chosen.

**4. The climate is settled again**, levels 3 and 5, on version 6's ground.
The atmosphere's lapse rate is 0.08 K a metre, so a 300 m summit is 24 K
colder than the shore.

## Risks

- [Something bounds terrain height] → The engine's height limits (the
  column tier's layers, culling, the atmosphere and the clouds, flight) are
  being surveyed now, and each one either holds for the chosen summit or
  becomes a task here.
- [Frame cost] → Taller ground shows more faces at distance. Not measurable
  in a cloud session; the owner runs `tools/perf_suite.py`.
- [The owner wants jagged peaks] → The peaks candidate is measured and
  offered (H3).
