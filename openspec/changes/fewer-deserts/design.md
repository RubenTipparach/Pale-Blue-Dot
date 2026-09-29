# Design

## Context

The temperate land, meaning dry land below the Mountains threshold and
outside the cold band, is split by one moisture field. Below `desert_below`
it is desert, above `wet_above` it is jungle (swamp where it is low), and
between the two it is fields. Version 5 set both thresholds at the field's
thirds (0.468 and 0.544), so each temperate biome is a third of that land
(survey B2). The owner, 2026-09-29: "Too much dessert. Biome is the right
size, but that's a lot of deserts."

## Measured: the candidates (2026-09-29)

A scratch instrument (not committed) drew version 6's ground at 1,024 x 512
through `pbd_core::map::base_texel`. It took the moisture field's
area-weighted quantiles over the temperate land, and placed `desert_below`
at the desert's share of it and `wet_above` halfway through the rest, so
fields and jungle split evenly what the desert gives up. Shares are of all
the land, by the mockup tool's count, which counts beach twice as wide as
the game's map key does (`docs/screenshots/bigger-biomes/README.md`).

| candidate (desert's share of the temperate land) | `desert_below` | `wet_above` | desert | fields | jungle | swamp |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| a third (version 5, and version 6 as built) | 0.468 | 0.544 | 21.0% | 21.0% | 19.9% | 1.0% |
| a quarter | 0.447 | 0.534 | 15.7% | 23.6% | 22.5% | 1.1% |
| **a sixth: half today's desert** | 0.422 | 0.524 | 10.5% | 26.2% | 25.0% | 1.2% |
| a tenth | 0.396 | 0.517 | 6.3% | 28.3% | 27.0% | 1.3% |

Beach 10.2%, Mountains 4.4% and tundra 22.6% of the land are the same in
every row: height and latitude decide them before moisture is read.

The four maps are side by side in the survey (B6). As the desert shrinks, it
keeps its inland regions and loses its edges. The regions stay the size the
owner chose, and there are fewer of them. At a tenth, most continents keep
one desert or none.

## Decisions

**1. The desert shrinks to half, provisionally (survey B6).** Recommended: a
sixth of the temperate land, 10.5% of all the land, half of today's.
- Deserts stay distinct regions a player can find on most continents.
- Fields (26%) and jungle (25%) each gain about five points, so grass does
  not become the majority again. That majority was the owner's reason for
  `bigger-biomes`.
- *Alternative:* a quarter, the gentlest cut.
- *Alternative:* a tenth, where deserts become rare.

**2. The freed land splits evenly between fields and jungle.** Moving only
`desert_below` would hand it all to the fields, which would then hold 31% of
the land against the jungle's 20%.

**3. In version 6, not a version 7 and not version 5.** Version 5 is in PR
#18 and may already have made worlds, so it keeps its deserts. Version 6 is
built but unreleased (`taller-mountains`): no world names it, so its
thresholds can still move. Its digest is taken when it lands, after this.

**4. The settled climates are made again.** The atmosphere reads the biome
for the surface's wetness and albedo, so version 6's two settled states are
remade after the thresholds move. Their gate is the same: a new world's days
hold 15 ± 0.5 °C.

## Risks

- [Less desert, more jungle, so more trees] → frame cost is not measurable
  in a cloud session. The owner runs `tools/perf_suite.py`.
