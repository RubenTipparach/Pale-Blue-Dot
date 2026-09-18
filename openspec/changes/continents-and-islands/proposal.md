# Proposal: break the supercontinent into continents and islands

## Why

**The owner asked whether the main land mass can be broken into a few
continents and some islands.** It can, and the orbit captures understate how
single that mass is. Measured by flood-filling the land on a level-6 dual
sphere (40,962 cells) with the shipped generator:

| | |
| --- | ---: |
| Land | **48.1%** of the body |
| Connected masses | 28 |
| The largest | **90.2% of all land**, 43.4% of the whole body |
| Second | 7.4% of the land |
| Third | 1.7% |
| Everything else | 25 scraps under 40 cells, most under ten |

So it is one supercontinent, one large island, and gravel. A count off a
screenshot could not have said that: a land bridge one cell wide joins two
masses that read as separate, and only the flood fill knows.

## The cause is TWO things, and the weaker one is the obvious one

The obvious cause is frequency. `continent_scale` is **0.8** on the unit
sphere, which is less than one full period across the whole body, so the
fractal's first and loudest octave is close to a single gradient. Land ends up
on one side.

The stronger cause is the **land fraction**. Sweeping the field shows the
largest mass's share of the land is governed more by how much land there is
than by how finely it is cut:

| scale | bias | land | masses | biggest, % of land | top five |
| ---: | ---: | ---: | ---: | ---: | ---: |
| **0.8** | **0.00** | **48.1%** | **28** | **90.2%** | 99.6% |
| 1.6 | 0.00 | 52.4% | 44 | 96.6% | 99.5% |
| 2.4 | 0.00 | 43.4% | 97 | 90.8% | 97.0% |
| 4.0 | 0.00 | 44.9% | 118 | 61.4% | 90.5% |
| 1.6 | -0.05 | 39.7% | 62 | 40.4% | 97.9% |
| 2.4 | -0.05 | 30.6% | 128 | 18.9% | 68.8% |
| 5.0 | -0.05 | 32.9% | 209 | 17.0% | 48.3% |

Read the `bias: 0.00` rows on their own and raising the frequency barely helps:
at 2.4 the largest mass is still 90.8% of the land, and at 4.0 it is 61.4%
across 118 pieces. **Above about 45% land the sphere is past percolation** and
the masses join up however finely the field is cut. Dropping the land fraction
to around a third is what actually separates them - which is also where Earth
sits, at 29%.

That is worth writing down because the intuitive fix, "make the noise finer",
is the half of it that does not work on its own.

## What changes

`TerrainConfig.continent_scale` and `land_bias`, and nothing else in the
generator. Both are already planet-scale fields with the right units: the
terrain-scale change established that a continent field keeps a unit-sphere
frequency, because a world has the same handful of continents whatever its
radius, and this is that knob being turned rather than a new mechanism.

**`GENERATOR_VERSION` bumps to 4 in the same commit.** It is part of saved
world IDs; a body whose coastlines moved is not the body that was saved.

## Three candidates, for the owner to choose

Rendered from orbit with the numbers burned into the frame, which is how the
moisture scale was settled.

| | scale | bias | land | biggest mass | reads as |
| --- | ---: | ---: | ---: | ---: | --- |
| **A** | 1.6 | -0.05 | 39.7% | 40.4% of land | a few big continents, close to the present world |
| **B** | 2.4 | -0.05 | 30.6% | 18.9% of land | Earth-like: several continents and real archipelagos |
| **C** | 5.0 | -0.05 | 32.9% | 17.0% of land | an ocean world of many islands |

B is the recommendation: it is the only one of the three where the largest mass
is under a fifth of the land, its 30.6% land fraction is Earth's, and its top
five masses hold 69% of the land, so there is a genuine long tail of islands
rather than two continents and gravel.

## What this costs, and what it does not touch

- **Every capture changes**, and so does where the spawn stands. The presets
  search for their own ground (`--view meadow` walks to pasture, `--view river`
  finds a channel with banks), so they keep working; they will photograph
  different places.
- **Nothing about the tile, the relief budget or the biomes moves.** The summit
  stays at the owner's 150 m, the cell stays 2.833 m, and the biome
  classification is untouched: what moves is where the land is.
- **The islands come mostly from the continent field**, not the island field.
  `island_scale`, `island_threshold` and `island_height_m` still only lift
  ground that is already in shallow sea, so they add atolls near coasts rather
  than mid-ocean chains. Tuning them is a separate, smaller decision and is
  left alone here so that one change moves one thing.
