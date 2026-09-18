# Proposal: the world is 16 times too coarse, and that is three complaints

## Why

**The owner, on the ported generator: "pbd still looks way smoother across
terrain surface, I wish for it to have more noise and height variation, rivers
and every biome in tenebris-rs."** Three complaints. They have one cause, and
it is arithmetic rather than taste.

Every noise field in the generator is sampled on the **unit sphere**, so a
feature's size is an ANGLE. The reference's body is 300 m and ours is 4,800 m,
exactly sixteen times bigger, and the cell stayed 2.833 m. So every feature the
generator makes is sixteen times wider in metres than the reference's, measured
in the cells a player walks over. Measured on both generators the same way,
over 40,000 directions of land and their neighbours one cell away:

| | Tenebris | ours | ratio |
| --- | ---: | ---: | ---: |
| adjacent cells differing by a block or more | **44.0%** | **14.2%** | 3.1x |
| mean step between adjacent cells | **0.56 m** | **0.17 m** | 3.3x |
| finest continent feature | 11.7 m = 4.1 cells | 188 m = 66 cells | **16x** |
| finest mountain feature | 11.3 m = 4.0 cells | 180 m = 64 cells | **16x** |
| finest hill feature | 15.0 m = 5.3 cells | 240 m = 85 cells | **16x** |
| finest detail feature | 12.5 m = 4.4 cells | 200 m = 71 cells | **16x** |
| finest river feature | 28.8 m = 10 cells | 462 m = 163 cells | **16x** |
| finest moisture feature | 23.4 m = 8 cells | 375 m = 132 cells | **16x** |

Sixteen across the board, because sixteen is the radius ratio. And the three
complaints fall straight out of that one row:

- **"Way smoother."** There is nothing in our height field with a wavelength
  under **180 m**, which is sixty-four cells. A player crosses sixty-four
  hexagons before the ground has anything new to say. Tenebris's finest
  feature is eleven metres, which is four hexagons. Its surface steps at a
  block boundary nearly half the time; ours steps one time in seven.
- **"Rivers."** They ARE generated - the channel field, the bed, the lowland
  gate, all ported. But a channel is a fraction of the field's finest
  wavelength, so at 462 m ours is a hundred-metre estuary that reads as a bay,
  not a river. Tenebris's is at 29 m and comes out a few cells wide.
- **"Every biome."** All eight are implemented and every one occurs. But the
  moisture field's finest feature is 375 m, so a biome is a region kilometres
  across: a player walks a very long way inside one, and the rarest (swamp at
  0.1% of the sphere, mountains at 0.5%) are effectively unreachable. At
  Tenebris's 23 m you meet several on one walk.

**This was not a mistake in the port.** The generator is faithful; what was
never decided is which of its fields describe a PLANET and which describe
LAND. A planet sixteen times bigger should have the same few continents and
sixteen times as many hills.

## What changes

Each noise field is declared as one of two kinds, and the distinction is the
whole proposal:

- **Planet-scale (angular).** The continent field and the rocky-highland
  region field. A world has a handful of continents whatever its radius, so
  these keep the reference's unit-sphere scales and the map keeps the shape the
  owner has already seen.
- **Land-scale (metric).** Hills, detail, rivers and moisture. These describe
  features a player walks over, so their size is a number of METRES, and their
  unit-sphere scale is derived from the body's radius rather than authored.

The mountain field sits between the two and is the one judgement call in the
change; the design puts a number on it and says why.

A second, independent gap the measurement exposes: our relief is **3.2% of the
radius** (153 m on 4,800) against the reference's **13.3%** (40 m on 300). So
even once the wavelengths match, our land is about four times gentler at every
scale. The owner has already fixed the summit at 150 m, so the design does NOT
propose raising it; it proposes that the land-scale terms carry their amplitude
in metres rather than as a share of the summit, which is what lets the ground
underfoot be the reference's without the mountains growing.

## What does not change

The cell size, the LOD, the water, the sky, the trees, the biome rules, the
top-block rule, the summit at 150 m. The generator's terms and their order are
untouched: this changes what a scale MEANS, not what the generator does.

Every world changes, so `GENERATOR_VERSION` moves with it.
