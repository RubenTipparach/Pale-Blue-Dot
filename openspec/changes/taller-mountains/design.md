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

## Rendered: the candidates in the game (2026-09-28)

From a scratch build (task 1.3), each candidate's tallest summit within 55
degrees of the equator: the wide ranges reach **249 m** there (the probe's
291 m is at a higher latitude), and the peaks **310 m**, on today's tallest
ridge (157 m). The wide ranges read as a massif about 2 km across with long
grey slopes; the peaks as a spire of cliffs. With the snow line at today's
150 m the ranges are white from their flanks, which is decision 3's reason.

## Surveyed: what bounds terrain height (2026-09-28)

Read from the code (task 1.2). Everything not listed scales without a cap.

**Hard limits:**
- **The column tier stops at 175 m.** `column::BASE_M` is -145 and
  `LAYERS` 320, so the top layer is 174 to 175 m. A summit above it, inside
  the 90 m tier round the player, is flattened to a 175 m plateau by
  `reconcile_surface` and pops back up outside the tier. Aiming, digging and
  placing refuse anything above 175 m ("above the world"), and worms carve
  nothing above it. Today's 160 m summit clears it by 15 m.
- **The tier's layer fields are 9 bits**: the run word (`from`, `to`), the
  water level and the lamp layer, so no layer index can pass 511 (a top of
  +366 m). `LAYERS` must also be a multiple of 8 (`MATERIAL_WORDS`). The
  largest that fits without widening the fields is 504, a top of +359 m.
  Past that, every field and its copies in `planet_surface.wgsl` widen to 10
  bits, and the lamp kind and lit bits move up one.
- **The cloud base is 300 m above the sea** (`sky::CLOUD_RADIUS`). Ground
  above it breaks four things written for ground below the cloud: rain
  shafts are skipped within 2 m of the base, the lightning bolt runs upward,
  the rain volume is missing for an eye above the base, and the map's
  overlay rejects ground above the base as if it were the moon.

**Soft effects (they change how things look, not whether they work):**
- The haze and the dusk glow are keyed on the camera's height above the sea,
  so a summit sees less haze (0.84 at 180 m, 0.76 at 290 m).
- The lapse rate is 0.08 K a metre, so a 290 m summit is 23 K colder than
  the shore and snowy most of the year. Steeper ground lifts the wind harder,
  so more cloud and rain form on the ranges.
- The map's hillshade exaggerates slope three times and saturates on steep
  ground. The temperature overlay clips at -30 °C.
- `--route clouds` flies over the ranges rather than low in the cloud where
  a range is taller than its cloud leg.
- The sky bake seeds every air layer above the ground, so a taller tier bakes
  more layers (task 3.3's frame cost).

**Pins that move with the summit:** `planet_gen::relief_holds_the_budget_and_the_land_fraction`
(130 to 180 m), `planet::terrain::tests` (120 to 180 m),
`column::the_span_covers_the_measured_relief`, and the relief requirements
in `openspec/specs/world/terrain` and `openspec/specs/planet/scale`. Both
specs have drifted from their tests on the floor and the land band, and the
delta brings them back.

**Found on the way:** the shaders' `COLUMN_BASE_M` and `COLUMN_TOP_M` are
not pinned by any test, though both files' comments say one pins them.
Raising the tier changes them, so the test is added first (decision 7).

## Decisions (the owner accepted them as recommended, 2026-09-28)

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
values are measured when the candidate is chosen. The owner, 2026-09-28, on
today's grey mountains: "Shouldn't top of mountains be like snowy". Today's
snowcap starts at 150 m and the highest summit is 160 m, so almost no
mountain reaches it. With the ranges it has to move up, or the whole massif
turns white from its flanks (the first scratch render, at 150 m): provisionally
200 m, so a 250 to 290 m summit wears a cap of 50 to 90 m over bare rock.

**4. The climate is settled again**, levels 3 and 5, on version 6's ground.
The atmosphere's lapse rate is 0.08 K a metre, so a 290 m summit is 23 K
colder than the shore.

**5. The column tier grows to 504 layers**, a top of +359 m, with `BASE_M`
unchanged. A saved edit is indexed from `BASE_M`, so every save stays valid,
and worlds of versions 4 and 5 only gain air above them. 504 is the most the
9-bit fields hold. It leaves 68 m of building room over a 291 m summit. The
tier's GPU buffers grow from 7.8 to 12.3 MB, and the sky bake does 57% more
layers (task 3.3).
- *Alternative:* grow the tier only on version 6 worlds. It is one more
  thing a world's version would decide, for 4.5 MB.

**6. The summit stays under the cloud base.** The recommended ranges reach
291 m, under the 300 m base. A taller choice (H1's third option) needs
either the cloud layer raised, which is a cloud change inside the owner's
priorities 2 to 4, or the four cloud-base cases fixed for ground above the
cloud, and past 359 m the tier's fields widened. So the budget's band is
250 to 290 m, and a test holds the summit under the cloud base.

**7. The shaders' column span is pinned by a test** before the tier grows:
`COLUMN_BASE_M` and `COLUMN_TOP_M` in `planet_surface.wgsl` and
`planet_visibility.wgsl`, read from the real files and held to
`column::BASE_M` and the top of `column::LAYERS`.

## Risks

- [A limit the survey missed] → The tier and the cloud base are the two
  found. The captures in task 1.3 are taken on the scratch config with the
  tier raised, at a summit, so a third shows up before the owner chooses.
- [Frame cost] → Taller ground shows more faces at distance. Not measurable
  in a cloud session; the owner runs `tools/perf_suite.py`.
- [The owner wants jagged peaks] → The peaks candidate is measured and
  offered (H3).
