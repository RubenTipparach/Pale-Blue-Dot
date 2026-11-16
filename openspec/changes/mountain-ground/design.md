# Design

## Measured (2026-11-16, read off version 10)

- `biome_at` (`planet_gen.rs`): Mountains is altitude over 105 m outside the
  cold band; slope and noise play no part.
- `top_material`: a Mountains cell is Snow above 200 m and Stone otherwise.
  Snow is the top 2% of the biome (`taller-mountains`), so the flanks are
  rock. Its inputs are direction, altitude and latitude; it never sees a
  neighbour. Under Snow or Stone the column holds stone at once, no soil.
- Render: Stone, Rock and Ore all draw as code 5, the sheet's (3,0) slate.
  `mountains.png` has sixteen tiles: scree, scree side, dark gravel, slate,
  iron ore, quartz seam, granite, fractured slate, lichen stone,
  snow-dusted rock, dark shale, worn path, angular pebbles, slate bricks,
  alpine soil, frozen fissure. Two are used: slate and (for the forge) slate
  bricks. Grass on a Mountains cell would draw (0,0), scree.
- Code budget: the column tier keeps 4 bits a layer and all sixteen codes
  are taken, so a new rock material cannot get its own code there. The
  heightfield's top is a byte; the run word has six spare bits.
- The tan rows: rows that follow the contours are terrace risers. A riser
  under snow draws SNOW_SIDE and under grass GRASS_SIDE, and both fade with
  distance (180 to 1400 m) to `GROUND_MEAN`, a lowland tan; from the air
  each snow or grass step is a tan band over the slate. To be confirmed with
  a capture of a flank before the fix.
- Slope at generation: `surface_altitude(cfg, dir)` is pure, so a cell's
  six neighbours (about 2.8 m off) can be sampled; mountain steps are mostly
  0-2 m, so "flat" is no neighbour more than 1 m off.

## Decisions

1. **Flat tops in version 11.** In Mountains, a cell whose six neighbours
   stand within 1 m of it takes: Snow above a snow line lowered to about
   170 m and ragged by a coarse noise (±15 m); alpine Grass below it down to
   the biome's edge, with patches of Dirt by a noise; a steep cell stays
   Stone. Under grass or dirt the column keeps two metres of soil over
   stone, as the lowlands do. Six extra `surface_altitude` calls a column,
   at generation only. V10 keeps its ground: its digest is pinned first.
2. **Alpine soil.** Grass and Dirt on a Mountains cell draw the sheet's
   alpine soil (2,3) and the fields' grass side on their risers, so they
   read as a mountain meadow, not scree. Draw-time.
3. **Rock variety without a new code.** Code 5 picks its tile per cell:
   bands of altitude (about 12 m deep, tilted by a low noise) choose among
   slate, granite, dark shale and fractured slate, and lichen stone takes
   patches below the snow line. One rule in the shader, read by the
   heightfield and the columns alike; mining yields are unchanged (all are
   Stone). It holds for every world, so old saves get the variety too:
   that is a look change and goes to the owner with screenshots.
4. **Risers' far colour.** SNOW_SIDE fades to the snow-over-rock mean, and
   GRASS_SIDE in Mountains to the alpine mean, instead of `GROUND_MEAN`.

## Risks

- A slope rule makes tops depend on neighbours: both tiers (heightfield and
  columns) call the same `top_material`, so they still agree, and a test
  holds that.
- Generation cost: six more altitude samples a column; measured in the
  generation timing the startup log prints.
