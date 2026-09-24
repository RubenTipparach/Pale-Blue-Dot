# Proposal: small clouds where the weather is fair

## Why

The owner: *"Parts of the planet have like no clouds. How can we get smaller
clouds in that area? So we don't have too many bald spots."*

Measured (climate report at level 5, one day, both solstices; and a
transcription of the cloud shader's density, `tools/drawn_cover.py`):

| Share of the planet's cells | Day 0 | Day 50 |
| --- | ---: | ---: |
| No cloud in the simulation (cover < 0.02) | 3.6% | 6.4% |
| Thin cover (0.02-0.3) | 23.6% | 32.3% |
| Cover 0.3 and up | 72.8% | 61.3% |

There are two causes. The renderer's is by far the larger.

**1. Thin cover is not drawn.** The shader draws a sample where the noise
exceeds `1 - cover` (the HZD remap). That rule assumes noise spread over 0..1.
But the shape is three value-noise octaves summed, and a sum bunches toward the
middle: p1 0.22, p50 0.50, p99 0.77, maximum 0.91. The share of the sky the
shader actually fills:

| Simulated cover | 0.05 | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.7 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Share of columns drawn | 0% | 0% | 0.5% | 9.8% | 41% | 78% | 99.9% |

So the quarter to a third of the planet under thin cover renders bald, though
the simulation has cloud there. Thin cover is exactly where small scattered
clouds belong.

**2. Some land has no cloud at all.** These cells are 98-99% land and mostly
within 35 degrees of the equator. Their humidity is 0.49-0.71 (median 0.59):
humid enough for fair-weather cumulus on Earth, but under the land's critical
humidity (0.7). That threshold was set above the sea's (0.45) so the sea would
stay cloudier than the land.

## What

1. **Draw what the simulation says.** Equalize the shape noise before the
   remap, so the drawn share of the sky tracks the cover. At low cover the
   remap then draws only the noise's highest peaks, which is small scattered
   cloud. At low cover the shape also takes a finer octave, so those peaks are
   cumulus-sized, not system-sized.
2. **Fair-weather cloud below the critical humidity.** Air humid enough for
   cumulus but short of the deck threshold gets a thin cover, capped (0.15 by
   default). It rains nothing, and it thins to nought by a lower humidity.

## Not in scope

- Rain, which comes only from condensed water and does not change.
- Higher cover anywhere: the cover of every place already at 0.15 or more is
  unchanged.
