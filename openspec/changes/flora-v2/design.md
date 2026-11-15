# Design: flora v2

Everything here is derived decoration drawn by the planet's existing passes.
No CPU mesh, no new pipeline, no new binding, no new texture: the pieces are
built in the vertex shader from the cell record (as `tenebris-ground-clutter`
and `tenebris-tree-geometry` already do), and their pixel shapes are cut in the
fragment from a hash and the biome sheets the atlas already has. The loading
screen's prewarm therefore has nothing new to compile.

## 1. Pixel tuft grass (the owner's request)

**What it is.** The mockup's tuft (`foliage.html`, `tuftTex`, after the Stone
and Timber Play demo): a 16 x 16 px card holding nine one-pixel blades, each at
a hashed column 1..14 and a hashed height 5..13 px, the odd blades stepping one
pixel sideways above row 6; the tone is the ramp's dark for rows 0-3, mid for
4-7 and light above. Three variants. Two crossed cards, 0.8 m across and 0.6 m
tall, nearest-sampled and cut out (a pixel is on or off; no blending).

**Where it is drawn.** In the grass slots of the clutter draw. A slot is a
blade of two stacked quads, twelve vertices; a tuft is two crossed quads, also
twelve. So the toggle is a branch inside the slot and **the vertex budget does
not move for it**. Slot `j` draws a tuft when `j < tufts` for the cell, where
`tufts = round((0.6 + 0.4 * roll) * tuft_count)`, the blade count's own
expression on its own knob (`tuft_count`, 6 by default: the mockup's 5 a
cell at sizes 0.8-1.3 is about that cover). Its seat, facing and size reuse the
blade's rolls, so a tuft stands where a blade would.

**The pattern is drawn in the fragment**, not stored: the card's `uv` carries
the variant in its integer part and the card position in its fraction, and the
fragment asks "is pixel (x, y) of variant v a blade?" by walking the nine
blades - nine integer hashes, cheaper than the texture fetch it replaces. The
same test is the cut-out, so it lives in `dropped`, which the sun's cascades
also call (clutter does not cast today, but a cut-out there is right if it
ever does).

**Per-biome ramps** are data in `pbd_core::flora::TUFT_RAMPS`, one three-tone
ramp per `Biome`, packed into the uniform as `0xRRGGBB` words. The mockup's
nine ramps map onto the game's eight biomes:

| game biome | mockup ramp |
| --- | --- |
| Fields | temperate (the demo's own greens) |
| Jungle | jungle |
| Swamp | swamp |
| Desert | desert |
| Beach | savanna |
| Mountains, Tundra | taiga |
| Ocean | kelp |

A tuft's colour is its ramp tone times the cell's variation, so neighbouring
cells are not stamped; it is lit like every clutter piece (surface-up normal,
the field at the cap).

**The toggle.** `PlayerSettings::pixel_grass`, off by default (the owner asked
for a toggle "to try" it; the default stays the blades until they say
otherwise - recommendation taken, ask only with screenshots). It is a check on
the settings' GRAPHICS page, "Pixel grass", kept in the settings file, which
moves to version 5 (an older file reads it off). A capture takes `--grass
tufts|blades`, and otherwise the isolated default (blades), so no existing
capture moves. The render world sees it through `FloraLook`, an extracted
resource that zeroes `tuft_count` in the uniform when off.

**It sways.** The tip row of each card moves along the card by
`0.06 * height * sin(1.7 t + phase)`, the root fixed: the grass moves because
the wind moves it, which the mockup rules allow (only things nothing moves must
stand still).

## 2. Fading at the clutter's edge

Clutter fades today by shrinking into the ground over `clutter_fade_m`
(`tenebris-ground-clutter`). That is kept for the blades, and every clutter
piece now **also** dithers out over the same stretch through the screen-door
mask `dropped` already uses for trees (`detail-fade`): a tuft is a flat card,
and a card that shrinks reads as sinking, where a dither reads as thinning.
The mockup's fade width (Q3, 0.3) is the shipped `lod_fade_width` and stays.

## 3. Richer per-biome clutter

The clutter instance grows from 432 to 564 vertices:

| vertices | piece |
| --- | --- |
| 0-215 | 18 grass slots (blades or tufts) |
| 216-233 | flower |
| 234-287 | pebble |
| 288-395 | bush |
| 396-431 | dead shrub |
| **432-539** | **mushroom: a stem prism and a cap prism** |
| **540-563** | **fern: four fronds** |

- **Flower heads** take one of four pixel colours (red, yellow, white, blue)
  from the biome's palette (`BiomeFlora::flowers`, a bit mask), chosen by a
  roll and carried to the fragment in the `uv`'s integer part. The head keeps
  its four-triangle fan and the atlas grain under the colour, as a lamp's
  parts do. The glowing flower (`lamps-and-lanterns` decision 8) is unchanged.
- **Mushrooms** stand on grassy cells at `mushroom_chance * weight(biome)`: a
  pale stem prism 0.12-0.2 m tall and a squat cap prism wider than it, red with
  white flecks in the fields, brown elsewhere (`BiomeFlora::red_caps`).
- **Ferns** stand on grassy cells at `fern_chance * weight(biome)`: four
  fronds arching out from a hashed seat, each a quad cut to a pinnate frond
  shape by a pixel mask and coloured from the jungle sheet's *fern ground* tile
  (the mockup's fern texture, "A:6:9").

The weights are the mockup's biome table read onto the game's biomes, in
`pbd_core::flora::BIOME_FLORA`: ferns heavy in the jungle, mushrooms in the
swamp and the forests, neither in the desert, on the beach only a few ferns.
The visibility pass lists a cell for them on the same rolls and the same
weights (both read them from the uniform), so a listed cell always grows what
it was listed for.

## 4. ~~Leaf cards on the hex-prism trees~~ (dropped; Q1 A, the owner's)

The owner, 2026-11-15, on the leaf cards as first built: *"Hmm trees have
weird leave structures on them I don't really like"*.

Eighteen cut-out leaf cards per tree were built round the crown (hanging from
each layer's rim and standing on the top, foliage draw 198 -> 306 vertices)
and are taken off again in the same change. **Q1 is A: trees stay the
Tenebris hex prisms as today** - the owner's own decision, not a
recommendation taken. The foliage draw is back to 198 vertices and the trees
cast and dither exactly as before this change.

**Not built: the baked mid and far tree** (Q2). Recommendation taken (crossed
cards mid, billboard far, baked from the near tree); its build is a
render-to-texture bake and an atlas, deferred to its own task; with the
cards gone the question is open again. The game's mid and far tree is the
prism crown it already draws.

## 5. Seasons on plants (Q5 A)

`pbd_core::flora::season_at(year_fraction, latitude_sin) -> Season { turn,
bare, straw }`, each 0..1, from the one calendar the clock already keeps
(`daylight::Clock::year_fraction`, `orbit-and-seasons`):

- The year begins at the +Y hemisphere's summer solstice, so a place's own
  season is `year_fraction` there and half a year on south of the equator.
- `turn` (autumn colour) peaks a quarter-year after midsummer, `bare` at
  midwinter, `straw` (dry grass) a little before midwinter and wider; each is a
  smooth bump on the circle of the year.
- All three scale by `smoothstep(0.12, 0.45, |latitude_sin|)`: no seasons in
  the tropics, full ones from about 27 degrees.

In the shader, deciduous biomes (`BiomeFlora::deciduous`: fields, swamp,
beach) tint the prism's leaves (material 9) a texel at a time toward a hashed
gold, orange or red by `turn`, and by `bare` fade them toward bare twig
grey-brown (the prism stays: it is the shape you chop). Grass
blades and tufts mix toward straw by `straw`. The rule runs per vertex in WGSL
with the same constants as `pbd_core`, and a test holds the two to one set of
numbers.

Snow is not touched: it is the generator's (material 6, by altitude and
latitude), and changing where snow lies would change cells.

## 6. Data and the uniform

`PlanetParams` gains, before `night` (which stays last, so the visibility
pass's prefix still ends where it did):

```
flora: vec4       x tufts per cell (0: blades), y tuft width m, z tuft height m, w year 0..1
flora_more: vec4  x mushroom chance, y fern chance, zw spare
flora_biome: array<vec4,8>  per Biome: fern weight, mushroom weight, flower colours (bits), deciduous
tuft_ramps: array<uvec4,8>  per Biome: dark, mid, light as 0xRRGGBB, w red caps
```

The visibility shader's copy of the struct gains the same lanes; the existing
struct-size test catches a mismatch. The knobs live in `scatter.ron`
(`tuft_count`, `tuft_width_m`, `tuft_height_m`, `mushroom_chance`,
`fern_chance`), validated: `tuft_count` within the eighteen-slot budget,
chances in 0..1.

## 7. Saved games

Nothing here is stored or generated. The pieces are pure functions of the cell
record's id, material and biome and of the clock; no cell's material, no
collision and no save record moves, and no generator version is needed.
Opening an old world shows the same ground with new decoration on it. The
settings file migrates forward (version 5, the new field defaulted).

## 8. Decisions taken (ask only with screenshots)

From the mockup's notes, the recommendations taken (recommendation taken, ask
only with screenshots) - except Q1, which is the owner's own answer - with
what this change does with each:

| # | question | taken | here |
| --- | --- | --- | --- |
| 1 | near tree | **A, hex prisms as today - the owner's own decision** (2026-11-15, after seeing C) | cards built, then removed (4) |
| 2 | mid ring for trees | A mid, B far, baked | deferred (4) |
| 3 | fade width | 0.3 | kept (2) |
| 4 | grass past its rings | A, the ground tile carries it | kept: no far grass level |
| 5 | seasons | A, one calendar per planet | built (5) |
| 6 | night light on plants | A, baked from static lights | kept: plants take the baked field at their cell, as clutter does |
| 7 | first alien biome | fungal forest | not in this change (a new generator version) |

And one of this change's own: **Pixel grass is off by default** (1), the
owner's to switch on and judge.

## 9. The approved mockup into the engine (M13, group 6)

**The owner approved M13 on 2026-11-15** ("commence implementation of all env
changes"). The mockup gained a run of changes after the slices above were
built (`docs/mockups/mockup-work.md`, row M13, and its captures in
`docs/screenshots/mockup-foliage/`). This section says, item by item, how each
goes into the game, or why it does not. Everything here is still derived
decoration or art: no cell, collision, save record or generator version moves,
and no pipeline, binding or texture is added (the loading screen's prewarm has
nothing new to compile).

### 9.1 Tufts coloured from the ground they grow on

The mockup reads each biome's tuft ramp off its own ground tile
(`foliage.html`, `groundRamp`): the tile's 1,024 texels sorted by luminance
(Rec. 709 weights, a stable sort), cut in thirds, and each third's commonest
colour (the first to reach the highest count, in row-major order) is its
dark, mid and light. The game's grassy caps (codes 2, 3, 7) all draw tile
(0,0) of the biome's sheet, so that tile is the ground the tufts stand on.

`pbd_core::flora::TUFT_RAMPS` keeps its shape and takes those colours. They
are constants because the shader reads them from the uniform, but they are
not hand-made: `planet_terrain`'s test recomputes `groundRamp` off the
committed `assets/tilesets/atlas.png` (after 9.5, so without the painted
flowers) and fails on any difference, printing the table to paste. The atlas
is the one source; the table is its checked copy.

### 9.2 One art style: plant contrast pulled to the texture's mean

The mockup draws every plant and prop in its texture's palette but not its
full contrast (`alb = mix(mean, alb, detail)`): leaves and bushes at 0.45,
bark, pebbles and other solids at 0.7, cut-out cards (tufts, flowers, ferns)
and the ground itself at 1. The game has the same split:

| piece | code | detail |
| --- | --- | --- |
| tree crown, bush | 9 (kinds 2, 3) | 0.45 |
| trunk, dead shrub twig | 8 (kinds 2, 3) | 0.7 |
| pebble | 5 (kind 3) | 0.7 |
| tuft, flower head, fern, mushroom, ground | - | 1 (unchanged) |

The mean is each tile's mean **linear** colour, per biome sheet, for the
three tiles the plants wear: leaves (0,2), bark (2,1) and stone (3,0).
`pbd_core::flora::PLANT_MEANS` holds them as `0xRRGGBB` (the linear mean,
sRGB-encoded so it packs like the ramps), and the same atlas test holds them to
the file. They reach the shader in a new uniform lane (9.8).

**No outlines.** The game's world art is the atlas, which has no outline ring
and no ink seams; the procedural pieces (flower heads, caps, tufts, fruit)
are drawn without one. `world_art.py` has nothing to port: the rule already
holds, and this change keeps it for the fruit (9.6).

### 9.3 Each LOD distance doubled, as the default

The mockup's three plant rings (near 22 -> 44 m, mid 45 -> 90 m, draw 80 ->
160 m) map onto the game's plant reaches as follows:

- **The clutter's reach** (grass, flowers, pebbles, bushes, shrubs,
  mushrooms, ferns, the new fruit) is the game's near ring: `clutter_radius_m`
  60 -> **120 m**, and its dither-out band `clutter_fade_m` 15 -> **30 m**, in
  `scatter.ron`.
- **Trees** already stand to the edge of the terrain's coarsest fine band
  (`FOLIAGE_DRAW_DISTANCE = BAND_M[0]`, 1,200 m, seven times the mockup's
  doubled draw distance). Doubling it would double the terrain's own LOD band,
  which is the terrain's change, not the plants'; it is **not** doubled.
  `tree_fade_m` is a fade width, not a distance, and stays.

Frame cost: the clutter's area grows four times (about 1,300 grassy cells
inside 60 m becomes about 5,000 inside 120 m). **Frame cost was not
measured** in this session (the owner's perf sitting); the knob is one number
in `scatter.ron` to take back.

### 9.4 Fallen leaves in the terrain shader

The mockup's fallen leaves (`#ifdef TERRAIN` block) port to the cap fragment
as written: on an up-facing ground face, the cap's own 32-texel grid is cut
into slots of `max(1.6, 2.4 * leaf_size)` texels; each slot holds two leaves,
each with its own moment in the fall (a hash); a leaf whose moment is under
the current reach is an ellipse of texels in one of six fallen-leaf colours
(rust, ochre, oak brown, tan, dull red, old gold; never a saturated orange),
the first down browned, a lit half and a shaded half. The leaves are texels
of the ground, so they never swim.

- **When.** The mockup's slider is per season (autumn 0.45, winter 0.6 of it,
  none in spring or summer). The game's year is continuous, so a new bump
  in `pbd_core::flora::Season`, `litter`: rising over `SEASON_LITTER_IN`
  (from the start of the turn to its peak), holding through the bare months
  at 0.6 of the peak, and gone over `SEASON_LITTER_OUT` before spring. It
  takes the same tropic fade as the rest, and only deciduous biomes
  (`BiomeFlora::deciduous`: fields, beach, swamp) lay leaves. The shader's
  copy of the constants is held to the core's by the existing season test.
- **Thicker under trees.** The mockup thickens the fall under the crowns
  (`Gather under trees`, 0.7). In the game a crown covers its own cell, so a
  cap whose cell stands a tree (`flora::grows_tree`'s rule, which the
  visibility pass already runs, repeated in the vertex shader with the same
  density table) takes the "under" weight across the cell.
- **Winter.** The fall is browned and wet in the bare months (the mockup's
  "browned on winter snow"). The game lays no seasonal snow (snow is the
  generator's material), so there are no leaves on snow to frost.
- **Knobs** in `scatter.ron`: `leaf_fall` 0.45, `leaf_under` 0.7, `leaf_size`
  1.0, validated in 0..1 (size 0.5..2).

### 9.5 No painted flowers on the ground tiles

`docs/mockups/foliage/no_flowers.py`'s four rules (the daisies off the
fields grass top and sod lip, the pale sprouts off the swamp moss, the orange
dots off the tundra top) become a step of `tools/build_tileset_atlas.py`,
applied to the baked atlas after sampling. The source sheets stay as
generated; the atlas is rebuilt from them and `--check` holds it. The result
is byte-for-byte the mockup's own `foliage/game/atlas.png` (checked when this
was written: the mockup's copy is exactly the game's atlas after
`no_flowers.py`). This is art, not generation: no cell changes.

### 9.6 Wild fruit and pine cones hanging from the trees

The mockup hangs fruit in the crowns of some trees (about half), with a few
fallen under them, kinds by biome (`FRUIT`), "on wood, never floating". The
game's trees are the Tenebris hex prisms (Q1 A), which have no branches; the
mockup's own rule for its hex trees is "fruit sits just inside the leaf
block's rim". So:

- **Where.** The foliage draw grows by **60 vertices** (198 -> 258): four
  hanging fruit, each two crossed cut-out cards (12 vertices), hanging from
  the underside of the lower leaf layer just inside its rim (a pine's from its
  lower skirt, part way out), and two fallen fruit lying flat on the cap
  under the crown (6 vertices each). The clutter's first vertex moves 258 ->
  318 and the column tier's 822 -> 882, in both shaders and the test-only
  arithmetic in `planet.rs`, as the leaf cards once did.
- **Which trees.** A tree fruits on its own roll (`fruit_chance`, 0.5), inside
  the clutter's reach (fruit beyond 120 m is a pixel and costs nothing but
  the degenerate vertex), and dithers out at the reach's edge like the
  clutter.
- **Kinds** (the mockup's table on the game's biomes; recommendation taken,
  ask only with screenshots): fields - apple, pear, plum, cherry; jungle -
  mango, papaya; beach (the savanna's stand-in, as for its tuft ramp) -
  marula, baobab; tundra pines - pine cones; swamp, desert, mountains, ocean -
  none (the swamp's fruit are forage plants in the mockup). Neighbouring
  trees tend to the same kind (patches), on a coarse hash of the tree's
  direction.
- **Drawn** in the fragment like the tufts: a 16 px pixel sprite cut out by a
  mask (round fruit with a stem pixel and a leaf; a pine cone a tapering
  ellipse of scales), in the fruit's three-tone ramp from `fruit_art.py`'s
  palette (`FRUIT_RAMPS`), lit top-left, no outline. The fruit keeps its own
  colour so it can be spotted.
- **Season.** Heaviest in autumn and thinned in winter (the mockup's keep:
  summer 0.85, autumn all, winter 0.08), by the same `Season`; pine cones
  stay all year.
- **Not gatherable yet.** Fruit is decoration in this change: picking it
  needs items, icons and a gather rule, which belong with the forage loop. The
  phase-1 wild plants (`farming-and-smelting` design 14) are the game's
  forage plants and are reused as they are; no second set is added.

### 9.7 Dry olive crowns where the leaf tile does not suit (savanna)

The mockup's savanna acacias and bushes wear the game's leaves through a
dry-olive gradient map (`RAMPS.savannaLeaf`: linear luminance 0.07..0.23 onto
`#3e3a1a`, `#7a7432`, `#b7a95a`), because the pale sage they wore read as
frost. The game's equivalent is the beach (the biome the flora tables already
read as the savanna): its leaf tile (0,2) is "beach grass ground", pale sand
with grass, which on a crown reads as frost in exactly the same way. So a beach
crown or bush draws the **fields** sheet's leaves through that gradient map,
and its mean (9.2) is the gradient map of the fields leaves' mean.

### 9.8 Data and the uniform

`PlanetParams` gains, before `night` (the visibility pass's prefix still ends
where it did, and the struct-size test catches a mismatch):

```
plant_means: array<vec4<u32>,8>  per Biome: leaves, bark, stone mean 0xRRGGBB; w the fruit kinds (bits)
flora_fall: vec4<f32>            x leaf fall, y gather under trees, z leaf size, w fruit chance
```

### 9.9 Not built, and why

- **Winter sod lip under snow.** The mockup's fix whitens the green lip of a
  column side under its seasonal snow cover. The game lays no seasonal snow:
  its snow is the generator's material, whose side already draws the tundra
  sheet's snow-over-soil tile. There is no green lip under snow to fix. Not
  built.
- **Far trees with no baked season.** A fix to the mockup's baked impostors;
  the game has no baked impostor (Q2 deferred), and its prism crowns take the
  season live. Nothing to port.
- **Fungal forest** (violet caps, warm stalks, dark violet moss). No game
  biome is a fungal forest; a new biome is a new generator version, its own
  change (Q7). Not built.
- **Crops and gardens layer.** The game's crops are the phase-1 farming
  (`farming-and-smelting`), grown by the player on tilled soil; the mockup's
  layer is M8's crops shown in place. Nothing to add here.
- **Forage plants.** Reused: the phase-1 wild plants (above).
- **Picking fruit.** Not in the game yet (9.6).
