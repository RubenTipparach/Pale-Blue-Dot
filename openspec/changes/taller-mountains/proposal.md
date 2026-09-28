# Proposal

## Why

The owner, 2026-09-28, looking at the version 5 screenshots: "can you make
mountains taller". Measured on today's generator (`design.md`, "Measured"):
the highest summit on the planet is 160 m, the median land is 44 m, and only
1.3% of the land stands above 120 m. The Mountains biome is 2% of the land.
From the spawn, the ranges read as grey patches a little above the fields.

## What Changes

- A new generator version, 6, whose mountain ranges are taller and wider. The
  lowlands, the coasts and the biome shares of the rest of the land stay as
  version 5 has them. The candidates and the recommendation are in the design.
  The owner picks one from in-game screenshots before it is built (survey H1
  to H3).
- The rocky-highland uplift, which almost never fires (its field tops out at
  0.75 on land, 0.70 at the 99th percentile, and the uplift starts at 0.72;
  tripling it moves no measured number), is replaced by what it was meant to
  be: the regions where the ranges rise.
- The snow line, the stone line and the Mountains biome's threshold follow the
  new heights, so the tall ranges have bare rock and snow caps rather than
  grass to the summit.
- The settled climates are made again on version 6's ground, levels 3 and 5,
  since the climate reads the altitude (a 300 m peak is 24 K colder than the
  shore at the lapse rate the atmosphere runs).
- Worlds made before keep their version (survey B3), as `bigger-biomes`
  already arranges.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `world/terrain`: the relief budget's summit band moves up, and a requirement
  is added that tall ground stays walkable (no more than a set share of cells
  steps more than a block to its neighbour).

## Impact

- `pbd_core::planet_gen`: `TENEBRIS_V6`, the range term in
  `surface_altitude`, and the version table.
- `pbd_core::terrain::GENERATOR_VERSION` to 6.
- Anything that bounds terrain height (the column tier, culling, the
  atmosphere and clouds, flight clearance), listed in the design with what
  each needs.
- `assets/climate/settled-g6-l{3,5}`.
- Frame cost: taller ground is more visible faces and a longer LOD reach.
  Not measurable in a cloud session; the owner runs `tools/perf_suite.py`.
