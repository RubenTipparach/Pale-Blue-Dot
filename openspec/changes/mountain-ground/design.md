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

1. **Flat tops in version 13.** (Planned as 11; `swamps` took version 12
   and `river-downhill` version 11 on 2026-11-18, so this is version 13.
   Version 14 went to `ocean-environments`' sea cells on 2026-11-19, built
   on version 12 and now a new world's; 13 was left free for this. Built
   after it, this takes the next free number on version 14's config, so a
   new world has both, and 13 stays unused.)
   In Mountains, a cell whose six neighbours
   stand within 1 m of it takes: Snow above a snow line lowered to about
   170 m and ragged by a coarse noise (±15 m); alpine Grass below it down to
   the biome's edge, with patches of Dirt by a noise; a steep cell stays
   Stone. Under grass or dirt the column keeps two metres of soil over
   stone, as the lowlands do. Six extra `surface_altitude` calls a column,
   at generation only. V11 keeps its ground: its digest is pinned first.
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

## Mountain flora and features (2026-11-19)

The owner, 2026-11-19: "nice, can you add more stuff to the mountain biomes?
add stuff to foliage detail playground". Shown on M13's Mountains tab
(`docs/mockups/foliage.html#mountains`, version 6), Today against New.

### 5. Measured: what a mountain grows today

- **Trees**: `TreeTable::V4` and `V10` give Mountains 0 trees per 256 cells.
  The kit's species for a mountain cell is the taiga pine, but no cell stands
  one.
- **Fruit**: none (`flora-v2` design 10: "swamp, desert, mountains, ocean -
  none").
- **Clutter** (`planet_surface.wgsl`, `clutter_grows`): a pebble on grass or
  bare ground, a bush and a fern only on grass, a dead shrub on desert sand
  and snow (codes 4, 6), a mushroom and a flower on grass. A version 12
  mountain is stone and snow, so it shows pebbles and, on the snow, dead
  shrubs, and nothing green.
- **Ground cover**: `TUFT_RAMPS` row 6 is read off the scree tile (0,0), so
  a tuft on a mountain cell is slate grey. `BIOME_FLORA` row 6 gives white and
  blue flowers at fern 0.3 and mushroom 0.5, which almost no mountain cell
  reaches.
- **The sheet**: `mountains.png` holds no grass. Its alpine soil (2,3) is a
  brown gravel; decision 2 would draw alpine grass with it, which reads as
  dirt, not meadow.
- **Clutter cost**: one instance of 564 vertices per clutter cell inside
  120 m, degenerate where a piece does not grow (`planet_visibility.wgsl`,
  `args[3]`).

### 6. The bands

Heights on a version 13 range (`taller-mountains`: the biome from 105 m,
summits to about 290 m). The snow line `S` is decision 1's, about 170 m and
ragged by ±15 m; everything above the meadow follows it.

| Band | Altitude | Flats | Steep cells |
| --- | --- | --- | --- |
| Montane forest | 105 - 135 m | alpine turf, firs and spruce, dense | rock, lichen |
| Tree line | 135 - 160 m | firs thinning to none at 150 m; krummholz 142 - 160 m | rock, lichen |
| Alpine meadow | 145 m - `S` | turf and tufts, flowers, cushions, juniper, heather, bilberry; soil patches | rock, lichen; scree and boulders at a cliff's foot |
| Snow edge | `S` - 14 m to `S` + 5 m | snow patches on shaded flats, a rare hardy flower | rock, moss in seeps |
| Snow | above `S` | snow; ice on the high flats (`S` + 35 m) | snow-dusted rock to `S` + 20 m, then bare; ridges scoured bare |

### 7. What is generation and what is drawn

The rule (CLAUDE.md, "Saved games survive every change"): anything that
decides a cell, its material or whether a tree stands on it is generation and
is version 13 only; anything drawn from a cell that already exists is
draw-time and reaches every world, as a look change.

| Part | Kind | Old worlds |
| --- | --- | --- |
| Alpine turf, soil patches, snow on flats (decision 1) | version 13 | unchanged |
| Firs, spruce, krummholz, the tree line | version 13 (`TreeTable::V13`) | no mountain trees, as now |
| Ridges scoured bare above the snow line | version 13 (`top_material`) | unchanged |
| Tarns (proposed) | version 13 (a liquid) | unchanged |
| Rock variety (decision 3), lichen, moss | draw | gets it |
| Scree tile at a cliff's foot, boulders | draw | gets it |
| Meadow flowers, cushions, juniper, heather, bilberry | draw | only where a mountain cell is grass, rare before version 13 |
| Snow patches, ice on high flats, the hardy flower | draw | gets it |
| Cairns (proposed) | draw, by a rare hash on summit cells | gets it |

### 8. The meadow in the clutter slots

No new vertices: on a Mountains cell the existing slots draw alpine pieces.
The rolls, salts and the gather rule (`clutter_grows`, `gather-by-hand`) are
unchanged, so the visibility pass, the CPU's rule and a taken piece stay in
step.

| Slot | Elsewhere | On a Mountains cell |
| --- | --- | --- |
| flower | a flower head | gentian, edelweiss, alpine avens (yellow) or moss campion (pink), by a hash; the hardy flower near the snow edge |
| bush | a leafy bush | dwarf juniper below 155 m, heather above, bilberry on soil patches |
| mushroom | a cap | a cushion plant (moss campion or white saxifrage): a low dome |
| fern | a fern | a dry sedge clump |
| pebble | a pebble | a boulder, 0.5 to 1.1 m, on a scree cell; a pebble elsewhere |
| dead shrub | on snow | as today |

- **Flower colours**: two new bits, `FLOWER_VIOLET` (gentian) and
  `FLOWER_PINK`, and `BIOME_FLORA` row 6 becomes violet, white, yellow and
  pink at fern 0.6 and mushroom 1.4. Each is kept to low saturation: the
  colour is the bloom's, the leaves and stems take the turf ramp.
- **The pieces are the foliage kit's, not new art** (owner, 2026-11-19, on
  version 5's sprites: "you should use the foliage kit instead of making new
  plants"). The tufts are the kit's pixel tufts coloured from the turf, the
  flowers its stem and head with the existing blue, white, yellow and red
  heads, juniper and heather its bush (needles, and the tundra sheet's
  leaves), a cushion its bush flattened with heads on top, bilberry its
  blueberry plant, the boulder its rock. Version 6 of the tab draws them so.
  The two new flower bits above wait on the owner's look at the kit's own
  colours; until then the kit's blue stands for gentian and its red for
  campion.
- **Every piece is lit as the terrain is** and has no outline (CLAUDE.md):
  stems and leaves from the turf's ramp, juniper and heather a step darker
  than the turf, the boulder from the cell's own rock tile.
- **Bilberry** is drawn only. As a forage item (picked by hand, eaten raw or
  baked into a tart at the campfire) it is a proposal for the forage loop,
  recommended and not built here.
- **Boulders** would be taken by hand like a pebble, which reads oddly for a
  metre of rock. Recommended (recommendation taken, ask only with
  screenshots): a boulder is not taken by hand; mining it with a pickaxe
  yields stone. That touches the gather rule and is its own task.

### 9. Alpine turf

The sheet's alpine soil is gravel, so a meadow needs a grass tile. A
candidate, **alpine turf**, is drawn in code
(`docs/mockups/foliage/alpine_turf.py`, written to
`docs/mockups/foliage/mountain/alpine_turf.png`): short cool sedge green with
straw tips, soil and slate grit between the blades, 32 texels, seamless, no
outline. It takes the sheet's unused "worn stone path" (3,2), which no code
reads. Decision 2 changes: a Mountains cell's grass draws the turf, its dirt
the alpine soil (2,3), and its riser the turf over soil. `TUFT_RAMPS` row 6
is then read off the turf, so the tufts are green on grass, and the atlas
test prints the new row.

### 10. Rock, scree, lichen and moss

- **Lichen** is drawn into the rock tile's texels, not as a decal: on a
  Mountains rock face (code 5, top or side) below `S` + 10 m, a coarse noise
  in the texel grid (32 a tile) picks blots of 2 to 6 texels in three tones,
  orange (`#b0703a`), yellow-green (`#9aa25c`) and grey (`#9a9f98`), each
  mixed with the rock's own texel so the shading stays, and denser on faces
  turned away from the sun. No new tile.
- **Scree**: a rock cell whose uphill neighbour stands 3 m or more above it
  draws the sheet's scree (0,0) on its top, and its pebble slot a boulder.
  The heightfield pass already reads the six neighbours' heights for its
  risers; the column tier reads its record's. Both call one function.
- **Moss in seeps**: the lowest metre of a rock face under a snow patch or
  beside water takes a dark moss tone in its lower texels.

### 11. Snow patches and ice

- **Patches**: a flat cell between `S` - 14 m and `S`, in shade (the cell
  light field's sky term below 0.6: a hollow or a face away from the sun),
  draws the snow tile (tundra (2,2)) over its own top, cut by a coarse texel
  noise so it is ragged, more of it nearer the line. Its material is
  unchanged: dig it and it is grass or stone. The tab's snow-line slider
  moves `S`.
- **Ice**: a snow cell above `S` + 35 m takes the sheet's frozen fissure
  (3,3) by a coarse noise.
- **The hardy flower** (a glacier buttercup, white with a yellow eye) grows in
  the flower slot within 6 m of the line.

### 12. Trees: `TreeTable::V13`

- Mountains: grass and soil stand trees, at a density that falls with
  altitude: 60 in 256 below 135 m, to none at 150 m. The density test does not
  read altitude today, so V13 adds a tree line per biome (the start and end
  of the thinning, optional, none for the other biomes), read by the CPU's
  `grows` and the visibility pass alike (one function; a test holds them).
- **Species**: the kit's own (owner, 2026-11-19: "use the foliage kit").
  **Fir**: the taiga pine `tree_kit` already draws, scaled 5 to 8 m, below
  the line; no new species. **Krummholz**: one new near draw built by the
  pine's own builder with parameters, not new art: its tapered trunk leaning
  downwind, whorls of branches with needle sprays rooted along them, stunted
  to a 0.6 to 0.9 m mat whose branches reach far to the lee and barely to
  windward; a flag tree (about one in three) stands 2 to 2.7 m out of its mat
  with branches on the lee side only; from 142 to 160 m. Four variants and
  its impostor bake.
- **Cones**: firs carry the pine cone (`flora-v2` design 10) in its season.

### 13. Proposed, not built in this change

- **Tarns**: still water in a closed basin on a bench above the tree line, a
  few cells, dark and clear, with a soil and moss rim. Version 13, at
  generation (a liquid in the column). Recommended only once `water-flow`
  holds a basin's level; otherwise left to a later version.
- **Cairns**: a stack of five stones on a rare summit cell, a landmark seen
  from the meadow. Draw-time decoration; not gatherable.
- **Creatures** (a note for animal work, not this change): mountain goats on
  the ledges, marmots in the meadow, an eagle overhead.
- **No waterfalls**: falling water's look is designed only in its own mockup
  (CLAUDE.md); a tarn here has no fall.

### 14. Frame cost (estimated, not measured)

- **Clutter**: no new vertices (section 8). A Mountains branch in the clutter
  fragment picks the sprite: a few instructions per fragment. Estimated under
  0.02 ms.
- **Terrain fragment**: lichen, scree and snow patches are a texel noise and
  a tile choice on Mountains cells only, a hash and a lookup per fragment.
  Estimated under 0.05 ms at 1440p.
- **Trees**: a mountain gains trees where it had none, but at most 60 in 256
  cells, below the fields' 82, so a range costs less than a forest at the
  same distance. Two more near draws, empty when no fir or krummholz is near,
  and two more impostor bakes (memory, not frame).
- The cloud session has no GPU; the owner's perf sitting measures it on a
  mountain walk, before and after (task 6.5).
