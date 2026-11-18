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

## 4. Leaf cards on the hex-prism trees (dropped), and the record corrected

The owner, 2026-11-15, on the leaf cards as first built: *"Hmm trees have
weird leave structures on them I don't really like"*.

Eighteen cut-out leaf cards per tree were built round the Tenebris hex-prism
crown (the mockup's flora-v2 option C: hanging from each layer's rim and
standing on the top, foliage draw 198 -> 306 vertices) and taken off again in
the same change. That much stands: cards stuck on prisms are gone.

**What this section used to say was wrong.** It read the owner's dislike as
choosing option A, "trees stay the Tenebris hex prisms as today", and called
that the owner's own decision. It was not. The dislike was of leaf cards
stuck on the hex prisms. The look the owner approved is M13, the Foliage
Detail Playground (`docs/mockups/foliage.html`), whose default tree geometry
is *"Detailed: parametric trunk, branches, leaf cards"* (`treeStyle:
"detail"`), with mid and far trees baked from the near model. The owner,
2026-11-15, on the shipped prisms: *"why didnt you implement new trees for env
2"*. The detailed trees are section 10.

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
only with screenshots) - except Q1, which is the owner's approved mockup - with
what this change does with each:

| # | question | taken | here |
| --- | --- | --- | --- |
| 1 | near tree | **M13's approved default, the detailed trees.** The owner disliked leaf cards stuck on the hex prisms (option C); that was misread here as choosing A, prisms as today. The owner, 2026-11-15: "why didnt you implement new trees for env 2" | cards on prisms removed (4); detailed trees built (10) |
| 2 | mid ring for trees | A mid, B far, baked | built with the detailed trees (10) |
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
game's trees were the Tenebris hex prisms when this was built, which have no
branches; the mockup's own rule for its hex trees is "fruit sits just inside
the leaf block's rim". So (superseded by section 10.6, which hangs the fruit
from the detailed model's own anchors):

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
- **Fungal forest** (violet caps, warm stalks, dark violet moss). Dropped
  for the home world (owner, 2026-11-18: "fungal forest should not be on
  this planet"); it stays an alien biome for another body, if any.
- **Crops and gardens layer.** The game's crops are the phase-1 farming
  (`farming-and-smelting`), grown by the player on tilled soil; the mockup's
  layer is M8's crops shown in place. Nothing to add here.
- **Forage plants.** Reused: the phase-1 wild plants (above).
- **Picking fruit.** Not in the game yet (9.6).

## 10. The detailed trees (M13's approved default)

The owner approved M13, whose default tree geometry is *"Detailed: parametric
trunk, branches, leaf cards"*; the game shipped the Tenebris prisms instead
(section 4 says why that was a misreading). The owner, 2026-11-15: *"why didnt
you implement new trees for env 2"*. This section ports the mockup's trees
(`foliage.html`: `branchTree`, `TREE_BUILD`, `leafCard`, `maskOf`, the
impostor bake `bakeImpostors`, the cross-fade `updateLOD`) into the engine.

### 10.1 What does not change: where a tree stands, and chopping it

The trees are a new **look** for the same trees. The placement rule
(`pbd_core::flora::grows_tree`, the visibility pass's `has_nearby_foliage`,
the surface pass's `stands_tree`) is not touched, so the cells that stand a
tree, the cells the axe can work, the fallen leaves gathered under them and
every saved world are exactly as before. No generator version moves.

**Which species a tree is, and its variant, turn and size, are decoration
derived at draw time** from the tree's own id (the record's `spare.x`, the
finest cell's id), hashed with salts of their own, and the biome's species mix
(the mockup's `BIOMES[...].trees`). Nothing is stored and nothing is
generated; the same id draws the same tree on every machine and every visit.
The mix, on the game's biomes (the same mapping the tufts and fruit use):

| game biome | species (share) | mockup biome |
| --- | --- | --- |
| Fields | oak 0.6, birch 0.4 | temperate forest |
| Jungle | kapok 0.55, palm 0.45 | jungle |
| Beach | acacia 1 (in the dry olive, 9.7) | savanna |
| Swamp | willow 0.75, birch 0.25 | swamp |
| Tundra | pine 0.85, birch 0.15 | taiga |
| Desert | cactus (scenery, below) | desert scrub |

**Cacti are scenery, not trees.** The game's desert grows no tree (the
generator's own words: "sand broken by rock, no trees"), so a cactus there
cannot be a tree without changing which cells can be chopped. A desert sand
cell grows a cactus on a roll of its own (`cactus_chance`, 8 in 256, the
mockup's `treeP` 0.07 times the cactus share), drawn by the tree draw with the
tree's LOD, but `grows_tree` stays false for it: it is not choppable and lays
no leaves, like a bush. The mockup's desert palm (0.2 of its desert trees) is
not drawn: there is no oasis in the game's desert to stand it in.

The glowcap, crystal and kelp species are the mockup's alien biomes, which the
game does not have (Q7); they are not ported.

### 10.2 The near tree: the mockup's model, built once on the CPU

`pbd_core::tree_kit` ports the mockup's builders line for line: the seeded
generator (`rngOf`, xorshift32 on `9000 + 31 v + 7 len(name)`), `branchTree`
with each species' parameters (`TREE_BUILD`: oak trunk 2.4 m, radius 0.24,
four crown limbs, depth 2, three kids, and so on), the willow's hanging
strands, the acacia's flat crown, the pine's whorls with needle sprays rooted
on each branch, the kapok's buttresses and long limbs, the palm's leaning
trunk and nine four-segment fronds, the cactus's ribbed arms; `leafCard` (root
edge half the tip's width, on the branch, normal blended toward the crown's
outward); and the fruit anchors (the last twigs' ends and 0.6 along them; the
pine's branches part way out; the palm's head). **Four variants per species**,
as the mockup builds, with the mockup's seeds, so a variant is the same tree
the mockup draws.

The meshes are built once at startup into a **tree kit**: one read-only
storage buffer (a new binding, 11, of the planet draw layout) holding a header
per species and variant (vertex range, anchors, height and width, the baked
views' offsets, the textures' means), the vertices (position, normal, uv, the
card's id, its root-to-tip AO and whether it is bark or leaf: six words), the
anchors, the four tiles the trees wear that the atlas lacks, and the baked
impostor pixels. The vertex shader pulls vertices from it by `vertex_index`,
as every other part of the planet is pulled; no vertex buffer and **no new
pipeline**: the draws are more indirect draws on the same pipelines, so the
loading screen's prewarm compiles what it did. The kit is built behind the
loading screen (its time is logged).

**Textures, nearest-sampled, the game's own** (the mockup's `SPECIES` table):
oak, kapok and willow wear their biome sheet's bark (2,1) and leaves (0,2);
acacia the fields sheet's bark and the settlement `leaves.png` through the dry
olive; birch `leaves.png` and the mockup's outline-free birch bark
(`docs/mockups/foliage/bark_birch.png`, from `world_art.py`, copied into
`assets/textures/flora/` with its provenance); pine the redwood sheet's bark
and the settlement `needles.png`; palm the mockup's palm bark
(`bark_palm.png`, from `tools/art/foliage_art.py`) and the jungle sheet's
leaves; cactus the desert sheet's (2,1). A leaf card's shape is cut from its
tile by the mockup's pixel masks (`maskOf`: leaves on a twig, a needle spray,
a frond), alpha-tested, never blended. Contrast is pulled to each texture's
mean (leaves 0.45, bark 0.7, design 9.2); the means are computed off the tiles
when the kit is built. No outlines.

**Wind.** The mockup's sway: a bend growing with the square of height over
the tree's base (leaves 0.16, bark 0.1, over 8 m), a slow sway and a rolling
gust, and the cards' own flutter masked by v squared so a card's root stays on
its branch. Trees move because the wind moves them (CLAUDE.md allows it).

### 10.3 Mid and far: baked from the near model, on the CPU

The mockup bakes each variant's near model from 8 bearings into a strip of
48 x 72 px views (orthographic, tilted 0.55 rad, rendered at 3x), snaps every
pixel to Sweaft-64 in OKLab (chroma weighted 8; the acacia's olive ramp
colours added to the palette), and folds each 3 x 3 block to one pixel (the
commonest colour where 6 of 9 are covered; the bark laid over the crown only
where it fills a block). The engine does the same **on the CPU** with a small
rasteriser in `pbd_core::tree_kit` (depth-tested triangles, nearest texels,
the bake's own key light and half-metre bands): deterministic, testable,
nothing read back from the GPU, no render pass. Deciduous species (oak,
birch, willow) bake twice, leafy and bare (bark only). Seasons are laid on
live, as the mockup does: the bake is in no season.

- **Mid** (the mockup's Q2 A): four crossed cards through the tree's axis,
  each showing the view baked from its own side.
- **Far** (Q2 B): one billboard turned to the camera, showing the baked view
  nearest the camera's bearing in the tree's own frame.

A coarser level's tree (`distance-lod-fade`: a quarter of the trees, each
standing for four) draws its billboard as wide as the prism grew, and half as
much taller, so a far forest keeps its cover.

### 10.4 No pop: one tree, cross-faded

The mockup's "fade" LOD, as `detail-fade` does elsewhere: each ring's end has
a band of 0.3 of its distance; across it the outgoing level keeps the pixels
whose screen-door threshold is under `1 - t` and the incoming one the rest,
so every pixel of the tree is drawn exactly once and the tree never appears,
vanishes or changes. The threshold is the 4 x 4 Bayer mask offset per tree
(the mockup's `floor(seed * 4) * (1, 2)`). Distances from the camera to the
tree's base:

| ring | to | band | drawn by |
| --- | --- | --- | --- |
| near | `tree_near_m` 44 m | 30.8-44 m | the near draws (one per species) |
| mid | `tree_mid_m` 90 m | 63-90 m | the impostor draw's crossed cards |
| far | the foliage range (unchanged) | `tree_fade_m` (unchanged) | the impostor draw's billboard |

The mockup's doubled values (design 9.3), as `scatter.ron` knobs
(`tree_near_m` validated 10..60, `tree_mid_m` above it). Fruit fades with the
near tree it hangs on. The landing dissolve and the band cross-fade of
`distance-lod-fade` apply to every level as they did to the prisms.

### 10.5 The draws and their budgets

- **The impostor draw** is the old foliage draw (`args[1]`), every tree the
  visibility pass lists, at **30 vertices** (four crossed cards, 24, and the
  billboard, 6), down from 258; a part the tree's distance does not need is a
  degenerate triangle. The clutter's first vertex moves 318 -> 90 and the
  column tier's 882 -> 654.
- **The near draws** are eight more indirect draws (`args[5..13]`, one per
  species), each at its species' largest variant's vertex count plus 60 fruit
  vertices, its first vertex `NEAR_TREE_FIRST_VERTEX + species << 16` (so the
  shader knows the species from the vertex index). The visibility pass lists a
  tree within `tree_near_m` into its species' region of a new list (binding 8
  of the cull; 8 regions of 4,096: the most finest cells inside 60 m, twice
  over for a landing's dissolve, is about 3,300). The sun cascades draw the
  near trees as well, so the detailed tree casts its own shadow.
- The vertex counts per variant and the instance counts are logged at
  startup and in the write-up. **Frame cost is not measured in this session**
  (the owner's perf sitting); the knobs to take it back are `tree_near_m` and
  `tree_mid_m`.

### 10.6 Fruit on the model's anchors, and seasons

- **Fruit** moves from the prism's rim to the near model's own anchors (the
  mockup's `anchors`: "on wood, never floating"): four hanging fruit, each at
  an anchor picked by the tree's roll, and two fallen under the crown, with
  the kinds, chances and season keep of 9.6 unchanged. The near draw's last 60
  vertices are the fruit.
- **Autumn**: a near leaf card turns as the mockup's do, the whole card to
  one colour between rust and orange by its id, more cards as the turn
  deepens; on the impostors, the crown's green pixels take the same turn.
- **Winter**: a deciduous tree's leaf cards drop out one by one as `bare`
  rises (each card on its own roll, so the crown thins rather than switching),
  and its impostors cross from the leafy bake to the bare one through the same
  roll per pixel. Pines, palms, kapok, acacia and cacti keep theirs.
- Snow on branches is not added: snow is the generator's material (5).

### 10.7 As built, and where it departs from the mockup

Built on `claude/env-trees` (2026-11-15); captures and an honest read in
`docs/screenshots/flora-v2/README.md` (the `trees-*` set).

- **The kit**: 32 models, 6,042 KiB, built in about 370 ms on the M4 Max in
  the fast build, on its own thread behind the loading screen. Near vertices
  per tree (largest variant, plus the 60 for fruit): oak 2,904, birch 1,584,
  kapok 588, palm 492, acacia 696, pine 6,126, willow 1,296, cactus 588.
- **The bake's palette** takes the crown's own three tones (the leaf tile as
  the bake shades it, cut in thirds by luminance) beside Sweaft-64, as the
  mockup takes its acacia's olive: Sweaft has no olive leaf green either, and
  a fields crown snapped to it alone came out a darker, cooler green than the
  near crown beside it (`trees-rings.jpg` shows the four levels side by side).
  The mockup's fixed pull of a baked crown toward the near one (`* (0.92,
  0.95, 0.72)`) is replaced by a measured one: the bake records its crown's
  mean, and the shader meets its brightness to the near leaves' mean under
  their root-to-tip shade.
- **Crown and bark pixels are told apart** by the bake (alpha 255 and 254),
  so the season and that match touch the leaves alone; the mockup guessed
  leaves by a green test, which an olive or autumn crown fails.
- **A mid card seen from behind** shows the view baked from that side; the
  mockup shows its front view mirrored.
- **Winter** thins a deciduous crown card by card as `bare` rises, the last
  ones in their autumn colour, where the mockup drops them all at once; its
  impostors cross from the leafy bake to the bare one a texel at a time.
- **Not seen in the game, though built**: the acacia. Every beach cell is sand
  (`planet_gen::top_material`) and a tree needs grass (`grows_tree`), so no
  beach tree has ever stood, prism or detailed. Placing one would change
  where trees stand, which saves forbid; it waits for the owner.

### 10.8 Not built

- The mockup's vines on jungle and swamp trees (a layer of its own), the
  glowing fungal and crystal and kelp species (no such biome, Q7), the
  desert palm (no oasis), and snow on the branches (5).
- The mockup's per-plant colour variation and baked canopy occlusion: the
  game lights the trees with its own sun, sky, shadow cascades and voxel
  field instead.

## 11. The crops and wild plants in the world's style

The owner, 2026-11-15: *"I see some plants are still using the old sprites, so
please get new sprites that have a more unified art style"*.

**Finding.** Every other plant in the world is drawn by the shaders: tufts,
flowers, ferns, mushrooms, fallen leaves and the detailed trees. The only plant
sprites left are the farm's: the four stages of each crop
(`assets/items/crops/<crop>_<stage>.png`), which also draw the wild plants (a
wild plant is its crop's ripe stage on the wild clump mesh). These are still
M8's outlined stages: a 1 px ink ring, dark seams between parts and near-black
shadow tones. That is the look the "No outlines in the world; one art style"
rule (CLAUDE.md, owner 2026-11-15) keeps to UI icons.

**The art already exists and is approved.** `docs/mockups/farming/world_art.py`
draws each stage again from M8's own drawing functions, for the world:
- no outline ring and no seams;
- each near-black ramp tone lifted one step.

It writes the stages to `docs/mockups/farming/world/` and to M13's
`foliage/crops/`. M8's plot and M13's "Crops and gardens" layer both draw
them, and the owner approved M13. They are 32 px, like the game's.

**Change.**
- **Art:** copy the 16 stages (wheat, turnip, chili, frost kale) byte for byte
  over the game's.
- **Same in the game:** the meshes, sizes, alpha cut-out, the wild plant's
  ripe stage, and every saved plot and picked record.
- **Icons:** the goods and seed icons stay outlined, because they are UI.
- **The forge's flame:** `flame.png` is fire, not a plant, and stays.

**Not plants, still outlined.** The light sprites in `assets/items/lights`
(campfire, torch, brazier) are outside this request. They are listed here
for a later pass.

## 12. Generator version 10, snow on the branches, vines, and fruit to pick (planned 2026-11-15)

The owner, 2026-11-15: "finish up fishing wiki, herding, and flora". This is
the plan for what 9.9 and 10.8 left, written before its code: forests as
dense as M13, acacias that stand, snow on the branches in winter, jungle and
swamp vines, and fruit a player can pick. Not in this pass: the alien species
(no such biome), the desert palm (no oasis) and the fungal forest (Q7, a
biome of its own).

### 12.1 Where a tree stands is the generator's, so it gets a version

`pbd_core::flora::grows_tree` decides which cells stand a tree from the
cell's tree id and its material word, through one table, `TREE_DENSITY`
(trees per 256 cells by biome). The foliage pass keeps the same table as
literals (`planet_visibility.wgsl`, `has_nearby_foliage`). A tree is a thing
a player works: chopping one writes a `tree` record keyed by the cell
(`tool-crafting`). So moving a tree moves what a save's records point at,
and changing the table for an existing world is the bug CLAUDE.md's "Saved
games survive every change" exists to prevent.

So the table becomes part of the generator:

- **`TerrainConfig` gains `trees: TreeTable`**: trees per 256 cells for each
  of the eight biomes, and the material codes a tree may stand on in each.
  Versions 4 to 9 carry today's table (`TreeTable::V4`), byte for byte.
  **Version 10** is version 9 with M13's table (12.2), and it is the version a
  new world is made on.
- **`grows_tree` takes the world's table.** It is called with the open
  world's `terrain_config().trees` (the aim, the gathering, the spawn's
  tool-crafting ground), so an old world's trees are the ones it had.
- **The shader reads the table from the uniform** (`PlanetParams` gains
  `tree_density: array<vec4<u32>, 2>`, the eight biomes, and
  `tree_ground: vec4<u32>`, the per-biome eligibility as bits). Its
  literals go. This closes `fishing-and-equipment` task 4's open half: a test
  runs the WGSL rule (through naga's validator for the layout, and the rule
  transcribed by a Rust mirror that reads the shader's text) against
  `grows_tree` over 10,000 cell ids and every biome and material, for
  version 9's table and version 10's.
- **Digests first.** Version 9's ground digest is pinned before version 10
  lands (it is: `version_9_makes_the_ground_its_worlds_are_made_on`), and a
  new **tree digest** pins where version 9's trees stand on 10,000 sample
  cells, taken before the table moves. Version 10 gets the same pair. Its
  ground (heights, biomes, top blocks) is version 9's, digest for digest:
  only the trees move.

**A finding that comes with this, measured in this session**: the ground
digests are not the same on every machine. Versions 6 to 9 were pinned on the
M4 Max Mac (aarch64) and fail on x86_64 Linux (this container); versions 4
and 5 were pinned on x86_64 and the Mac session reports them failing there.
The generator's `f32` arithmetic (`sin`, `cos`, `powf`, fused multiply-adds)
differs between the two targets in the last bits, and a threshold somewhere
lands on the other side. A world made on the Mac and opened on the desktop
(RTX 3070, x86_64) can therefore differ in the odd cell. This pass does not
fix that (it is a determinism change to every version, and needs its own
write-up); it pins version 10's digests per target
(`cfg(target_arch)`), records the x86_64 values of 6 to 9 beside the
aarch64 ones so both machines' test runs mean something, and lists the
cross-machine drift as an open question for the owner.

**Versions 4 and 5 on the Mac, bisected and measured (2026-11-16,
`claude/old-ground-digests`).** `version_4_makes_the_ground_every_old_world_was_made_on`
and `version_5_makes_the_ground_its_worlds_were_made_on` failed on the M4 Max
Mac. There is no breaking commit to bisect to: each test already fails on the
Mac at the commit that added it, `69cf22a` (version 4's digest) and `792a6f0`
(version 5's), with the very values the branch gives today (version 4
`7957956841201382435`, version 5 `10767986828596518258`). The pinned values
were taken in the cloud container (x86_64 Linux). Nothing moved the old
ground; the digest was never the Mac's.

The proof, run with each commit's own source (`git archive`, a probe test
added to the scratch copy only): for each of the 10,000 digest directions,
the direction's bits, the altitude's bits, the biome and the top block were
written out for versions 4 and 5.

| Run | Version 4 dump (sha1) | Version 5 dump (sha1) |
| --- | --- | --- |
| `69cf22a`, aarch64 Mac | `b2fce595...` | (version 5 not yet made) |
| `792a6f0`, aarch64 Mac | `b2fce595...` | `3c748530...` |
| `b924ff8` (`claude/env-v2`), aarch64 Mac | `b2fce595...` | `3c748530...` |
| `b924ff8`, x86_64 Mac (Rosetta) | `b2fce595...` | `3c748530...` |

Byte for byte the same ground, from the commit that pinned each version to
today. The top-block digests (`3219747905282479865`, `13661571992430582298`)
match the pinned values on the Mac, as on Linux.

What this corrects in the paragraph above: the split is the **operating
system's maths library**, not the processor. An x86_64 build on the Mac makes
the aarch64 values for versions 4 to 9, so `cfg(target_arch)` was the wrong
switch. The digests are now pinned per `cfg(target_os = "macos")`, versions 4
and 5 included, with the Linux values kept as they were. Version 10's column
digest (materials only, pinned on x86_64 Linux) passes on both Mac targets
too, so its `cfg(target_arch = "x86_64")` gate is dropped: on those 2,000
columns the platforms part in an altitude's last bits, not in which material
a cell holds. Which call differs (the test's own `sin` and `cos` that lay out
the sample directions, or the generator's `f64` `powf`) was not isolated
without a Linux machine; Windows (the desktop) was not measured, and gets the
Linux values as before. Whether worlds made on one OS and opened on another
can differ in the odd cell stays the open question above.

### 12.2 M13's densities

M13 rolls a tree on a cell with probability `treeP × (0.25 + 1.5 × forest)`,
`forest` a slow noise. Its mean over a biome is about `treeP`. Version 10
takes the means, in trees per 256 cells (recommendation taken, ask only with
screenshots: the forest-and-clearing noise is not ported, because the shader
and the CPU would each have to compute it and agree at every threshold):

| Biome | M13 biome, `treeP` | Version 9 | Version 10 |
| --- | --- | ---: | ---: |
| Fields | temperate forest, 0.32 | 13 | 82 |
| Jungle | jungle, 0.55 | 115 | 141 |
| Swamp | swamp, 0.26 | 34 | 67 |
| Tundra | taiga, 0.30 | 2 | 77 |
| Beach | savanna, 0.06 | 13 (on grass, which a beach never has) | 15, on sand |
| Desert, mountains, ocean | | 0 | 0 |

The tundra's pines stand on snow as today, and now on its grass too.

**Acacias stand** (10.7's finding): a beach is sand, and only grass grew a
tree, so the acacia was built and never seen. Version 10 lets the beach's
sand (material code 1) stand a tree at 15 in 256, which the kit draws as
the beach's species, the acacia. The cactus rule is untouched (desert only).

**Frame cost** rises with the tree count (fields about six times as many
trees). Not measured here (no GPU); the owner's perf sitting judges it, and
the LOD distances are the lever if it is too much.

### 12.3 Snow on the branches

Section 5 left snow alone because the generator's snow is a material and a
seasonal one would move cells. Snow on a tree's branches moves nothing: it is
decoration on a drawn model. The near tree's fragment whitens the up-facing
side of a bark or leaf card by a **winter cover**:
`cover = bare(season) × cold`, where `cold` is 1 on a tundra or mountain
cell and on any cell above 50° latitude, and falls to 0 by 35°. Pines, which
keep their needles, carry the most; a deciduous tree's bare twigs a thin
line. The impostors take the same cover on their up-facing pixels (the
bake's normal is not kept, so the top third of the crown's pixels). A test
holds the Rust `winter_cover` and the WGSL to one set of numbers, as 5 does
for the season.

**The winter sod lip** stays not built (9.9): the season model lays no snow
on the ground, so there is no green lip under snow to fix.

### 12.4 Vines

M13 hangs vines from the jungle's and the swamp's trees (`vines: 0.7` and
`0.5`). They join the near model in `tree_kit::build`: on the kapok, the
palm and the willow, a strand of leaf cards hangs from some of the model's
anchors (a vine's chance by species, seeded like the rest of the model), each
a chain of 3 to 6 narrow cards falling straight down with a little sway,
on the leaf tile in the crown's colours. They are part of the model, so they
are baked into its impostors and fade with it, and the near draw's vertex
count is the model's (`near_vertices` reads the kit). A core test pins that a
vine hangs from an anchor and that no vine is longer than the trunk is tall.

### 12.5 Fruit to pick

9.6 drew the fruit and left it "not gatherable yet". Now:

- **Goods**: one good per fruit kind (`apple`, `pear`, `plum`, `cherry`,
  `mango`, `papaya`, `marula`, `baobab`, `pine-cone`), appended to the goods
  table with 16 px icons reduced from M13's own fruit sprites
  (`docs/mockups/foliage/fruit_art.py`'s ramps), outlined as UI icons are.
- **Which tree bears what** is the drawing's rule, moved to the core so the
  CPU and the shader read one answer: `flora::fruit_of(tree_id, biome,
  direction)` gives the kind (the patch hash on the tree's direction) and
  whether this tree fruits at all (`fruit_chance`). How many hang today is
  the season's keep (summer 0.85, autumn all, winter 0.08), so a tree in
  winter mostly has none to give.
- **Picking**: E on a fruiting tree (the aim's `tree_on_ray`) gives its
  hanging fruit, up to four. What was picked is a **stored record**,
  `fruit` by the cell's key (schema 1, the picked day and how many), written
  with the hands in one line, like the `tree` record. A tree's fruit grows
  back after `regrow_days` (8 world days): a picked record older than that
  reads as a full tree again.
- **Seeing it go**: a picked tree's hanging fruit must not still hang. The
  tier's record build sets a **picked bit** (bit 25 of the material word,
  beside the town's cleared bit) on a cell whose `fruit` record is younger
  than `regrow_days`, and the fruit vertices of that tree are skipped. The
  same path sets a **felled bit** (bit 26) on a cell whose `tree` record says
  it was felled, which is `fishing-and-equipment` task 4's other open half: a
  felled tree stops being drawn, after a tier rebuild too.
- **Saves**: `fruit` is a new record kind at schema 1, noted in
  `identity.ron` the first time one is stored; nothing else in a save
  changes. Fruit decided by the generator's table is derived and never saved;
  only what a player took is.

### 12.6 The ground follows the seasons

The owner, 2026-11-15: "winter isn't changing terrain texture". A winter
capture of the fields (`--day 52`) shows the deciduous trees bare, a few
yellow leaves left, over ground still summer green: grass tops, tufts and
terraces as in July.

**Finding.** Section 5 put the season on the plants only: the tufts and
blades mix toward straw by `straw`, the leaves turn and fall. The ground's
own faces (the grass top tile, the green sod lip down a column's side, the
terrace steps) are drawn from the atlas with no season at all, and at
midwinter `straw` (a bump at 0.47 of the year, 0.22 wide) is already
falling, so even the tufts are greener than the trees above them.

**Change** (draw-time only: derived, never saved, no generator version, no
save change):

- **One calendar.** `pbd_core::flora::ground_season(year_fraction,
  latitude_sin) -> GroundSeason { dry, frost }` reads `season_at`, the trees'
  calendar. `dry` rises through autumn and holds through winter (the larger
  of `straw` and `bare`), and is gone by spring. `frost` is `bare` times the
  cold of the latitude, the snow-on-branches ramp (`SNOW_COLD`, 35 to 50
  degrees). Both scale by the tropic strength, so the equator and the
  tropics never change. The desert, beach sand, rock and snow tops are not
  grass and are never tinted.
- **The tint keeps the tile's palette.** A grass face's texel keeps its own
  light and dark: the season moves its hue, not its pattern. Dry
  multiplies the texel's luminance into the dried-grass ochre
  (`GROUND_DRY`), mixed in by `dry × 0.6`. Frost lifts it toward a pale
  blue-grey (`GROUND_FROST`) by `frost × 0.45`, a light rime and never a
  white sheet. No outline; the tufts' straw follows the same `dry`, so
  ground and tufts move together.
- **Where.** The grass top faces (codes 2, 3, 7, the swamp's 7 included),
  the grass sod lip on a column's side (`GRASS_SIDE`), and the tufts and
  blades (their straw now follows `dry` instead of the narrower `straw`).
- **The winter sod lip** (9.9) follows: the green lip on a column side
  takes the same dry and frost as the top above it, so a cold winter's
  terrace is dull and frosted, not a green band.
- **No seasonal snow cover.** The game's climate lays no snow on the
  ground by season (snow is the generator's material), so none is invented
  here; frost is the most a cold winter shows.
- **Held together.** The WGSL `ground_season` mirrors the core's numbers;
  a test reads the shader's constant lines against `GROUND_DRY`,
  `GROUND_FROST` and the mixes, as the season test does.
- **Verified** with captures at the same fields spot and forest edge at
  `--day 2` (summer), `--day 27` (autumn) and `--day 52` (winter), side by
  side.

### 12.8 Ore veins, in the same version 10 (owner, 2026-11-15)

The owner: there is no ore anywhere in the generated ground, so mining and
smelting cannot be played. They approved adding veins in this same new
generator version, so worlds take one bump, not two.

**Finding.** `column::material_at_depth` gives sod, then soil, then stone,
and nothing else: no generated cell is ever `Material::Ore`. Everything after
the ground is already wired:
- digging an ore cell yields coal one time in three (a seam hashed on the
  cell and layer) and mixed ore otherwise (`station::ore_yield`);
- `Material::Ore` is tier 1, so a stone pickaxe takes it (`dig::ore_tier`);
- the forge's `mixed-ore` recipe smelts four mixed ore and two coal into two
  copper, one iron and one tin, and `bronze` makes bronze from copper and
  tin.
So the whole mining and smelting loop is one missing generator rule.

**Change** (generator version 10, beside version 9; derived from the seed
and the cell, never saved):
- **`TerrainConfig` gains `ore: Option<OreVeins>`**, `None` for versions 4
  to 9 and `Some(OreVeins::V10)` for version 10. `column::generate_solid`
  turns a stone layer to `Ore` where `OreVeins::is_ore` says so; nothing
  else in a column moves.
- **Veins are clusters.** Space is cut into 3 m cubes in the body's frame. A
  cube holds a vein on its own hash at a share that depends on depth (18%
  of cubes within 14 m of the surface, 7% deeper), and inside a vein cube
  each 1 m cell is ore on its own hash, 55% of them. So ore comes in
  clumps a few cells across, often just under the soil of a hillside, and
  a stone pickaxe finds it in the first minutes.
- **Which ore.** Recommendation taken (ask only with screenshots): the
  game's one ore block, mixed ore, rather than four new blocks. It already
  yields coal (a third of cells), and its smelt gives copper, tin and iron
  together, so bronze is reachable from the first vein. Separate copper,
  tin, iron and coal blocks, with iron deeper, need new blocks, tiles and
  icons, and wait for the owner (a mockup of the veins first).
- **Saves.** Version 9 keeps its columns cell for cell. A test holds version
  10's columns to version 9's everywhere except stone turned to ore, so
  the comparison holds on every machine whatever the cross-machine drift
  (12.1). A digest of version 10's columns is pinned per target. A world on
  version 9 opens on version 9 and finds no new ore; a new world is on 10.
- **Verified** by a test that most land columns sampled have ore within 14
  layers of the surface, and by a capture of a dug hillside showing the
  veins.

**As built** (2026-11-16): `pbd_core::ore::OreVeins`, `TerrainConfig::ore`
(`None` to 9, `V10` at 10), and `column::generate_solid` turning stone to ore.
Tests: `version_10_is_version_9_s_ground_with_ore_veins` (3,000 columns:
nothing but stone to ore; ore 2 to 12% of the stone; version 9 none),
`version_10_has_ore_near_the_surface_on_land` (ore within 14 m in 390 of
768 sampled land columns, 51%), `version_10_s_columns_are_pinned` (x86_64).

**A finding that stops the capture: ore is drawn as stone.** The column's
material word packs eight 4-bit codes, and `planet_terrain::render_code`
gives `Ore` the stone's code (5); all sixteen codes are taken (water 13,
dirt 10, the sod and snow sides 11 and 12, tilled 14, the forge 15, wood
8). So a vein is real (a pickaxe on it gives coal or mixed ore,
`station::ore_yield`) but a dug wall shows it as stone, and a capture cannot
show the veins. Drawing ore needs a fifth bit in the material word (six
codes a word, both the Rust packing and `material_at` in
`planet_surface.wgsl`, and the runs' code field) and an ore tile on the
atlas's stone sheet. That is a rendering change of its own, proposed as the
next step (task 9.9) and not taken here. Superseded by 12.12: no fifth bit;
the placed-sheet field's spare value marks ore.

### 12.10 Captures draw the grass the player sees (owner, 2026-11-16)

The owner, on a capture: "this screenshot shows the wrong grass", and on
their own play: the pixel tufts. Measured: a run with `--capture`,
`--frame-log` or a showcase starts from `PlayerSettings::ISOLATED` (the
engine before player settings existed), which kept `pixel_grass: false`, so
every capture since the tufts became the player's default (design 1) drew
the old blades cut from the ground tile. A test pinned it ("captures keep
the blades").

`ISOLATED` now draws the tufts, as the player's default does; `--grass
blades` still draws the old look. A perf-suite number taken before this and
one after differ by the grass as well as the change measured, so a
comparison runs both exes in one sitting as it always does, or passes
`--grass blades` to both. Rain shadows stay on in `ISOLATED` (player
default off) because the perf suite measures them; a capture meant to look
like play adds `--rain-shadows off`.

### 12.9 What this pass leaves

- The forest-and-clearing noise (12.2), alien species, the desert palm, the
  fungal forest.
- The cross-machine digest drift (12.1), written up, not fixed.
- Frame cost, not measured (no GPU in the cloud session).

### 12.10 Nothing grows where water stands; a drowned top is drawn as a bed (owner, 2026-11-16)

The owner: "tree and grass is spawned there, river should be dirt"
(`docs/screenshots/rain-artifacts/owner-river-43.webp`, `-44.webp`). On a
river's bed stand grass tufts, a tree and red mushrooms, on a green sward.

**Reproduced** at a version 10 river 60 m from the spawn
(`docs/screenshots/river-fixes/before-*.jpg`).

**Why, measured in the code.** The generator writes a column's top from
`top_material`, which knows nothing of rivers: a river carves its bed two
layers under its water and the bed keeps the meadow's top, `DryGrass` or
`Grass` (`rivers::water` puts the water over it; at 28.25, 0.75 the ground
is 63 m, `DryGrass`, with water to 65 m). Every plant is then decided at
draw time from the cell record's material word: tufts, flowers, ferns and
mushrooms on a grassy code (`grassy` in `planet_visibility.wgsl` and
`planet_surface.wgsl`), a tree on a grass, snow or (version 10's beach)
sand code (`TreeTable::grows`, the foliage pass). The record's code is the
column's top (`reconcile_surface`) on the column tier and `surface_code`
beyond it. No rule asks whether water stands over the cell.

**Two ways to make the bed dirt.**

1. *A new generator version, 11*, writing river beds as dirt. Honest for
   digging (a bed would give dirt), but only for new worlds: every world
   made so far keeps green beds, and version 10's digests and a new
   version's would both have to be pinned and carried forever.
2. *A draw-time rule*: a top with water standing on it is DRAWN as a bed.
   The material the column holds is unchanged, nothing new is saved, no
   generator version moves, and every world, old and new, shows it.

**Chosen: 2, the draw-time rule**, because it is derived state (CLAUDE.md:
"what the seed and the rules decide is derived and never saved"), fixes
old worlds' beds as well as their plants, and puts no save at risk. The
rule, one function in each place that turns a material into a drawn code:

- `Material::drowned` (core): a grass of any kind or snow under water is
  drawn as dirt; everything else as itself. `Column::drawn_material(layer)`
  applies it where the layer over is water; `packed_runs` and the tier's
  per-layer codes and `reconcile_surface` draw from it.
- Sand under water is drawn as the seabed's sand (code 0), as
  `material_index` already draws sand below the sea, so a version 10
  acacia cannot stand in a river's mouth either. On the column tier this
  applies only above the sea's level, so the near seabed is unchanged.
- Beyond the column tier the record's word asks the generator's river
  (`rivers::floods`: a reach's water stands over the ground here), the
  same answer the column will give.

Dirt (code 10) is neither `grassy` nor `bare`, so it grows no tufts,
flowers, ferns, mushrooms, bushes or pebbles, and no tree's table names
it. The CPU's tree list (`grows_tree` over the records, used by the axe
and the fruit) reads the same words, so a tree that is not drawn cannot be
chopped. A `tree` record for a cell that no longer stands one is kept and
harmless: the tree is simply not there.

**What it leaves.** Digging a drowned bed still gives what its material is
(a grass block gives fibre and sod): the column is unchanged. If that
matters, version 11 can write beds as dirt for new worlds on top of this
rule. Water the player pours over grass draws the grass under it as dirt
too, and the sward comes back when the water goes: one rule for every
water.

### 12.11 Nothing grows where the sea or the tide stands (owner, 2026-11-16)

The owner, on the sea-shore waterline video
(`docs/screenshots/river-fixes/videos/sea-shore-after-sheet.jpg`): "is that
tree in your screenshot in the river?" An acacia stands in the sea a few
metres off a version 10 beach (`18.86 27.54`).

**Why, measured** (a probe over the finest cells within 80 m of that spot,
version 10, the far record against the column tier's):
- The far records stand no tree there: every tree within 80 m is on grass
  11 m or more over the sea.
- The column tier stands 18 beach trees in the sea and 11 in the tide.
  - **In the sea:** cells whose top is sand at -1 m, the sea's water over
    it. `reconcile_surface` draws a top under water as a bed only above the
    sea's level (12.10 kept the near seabed as it was), so the record's
    code is `render_code(Sand)`, 1, the beach's sand. The far record draws
    the same cell as the seabed, 0. Version 10's beach (biome 1) grows trees
    on code 1, so the tier stands an acacia in the sea.
  - **In the tide:** sand tops at 0 m and 1 m, dry at the mean sea but
    under the drawn sheet at high water. The sheet stands at the sea level
    less `depth_offset_m` (0.5 m) plus the tide, which reaches
    `tide_high_m(3.0)`, 2 m, so a top under 1.5 m is under water for part
    of every day. 12.10 knows nothing of the tide.

**The rule.** A cell water stands on at any tide grows nothing: no tree,
cactus, tuft, flower, fern, mushroom, bush or pebble. It is a draw-time
rule, as 12.10 is:
- **A new bit in the record's material word, bit 25 (`FLOODED_BIT`).** It is
  set where the top is under the high-water line (the sea level less the
  sheet's offset plus the tide's highest, 1.5 m with the shipped
  `water.ron`), and on the column tier also where the column holds water on
  its top (a river, a pool, water the player poured). `surface_code` sets it
  for the far records; `reconcile_surface` sets or clears it for the tier's,
  so a pool that drains grows its plants again.
- **The material code is not changed.** The beach keeps its sand's look and
  the near seabed keeps its code: the bit says only that nothing grows.
- **Every reader of where plants stand reads the bit.** The tree rule in the
  visibility pass (`has_nearby_foliage`, which also places the cactus), its
  copy in the surface pass (`stands_tree`, which gathers the fallen leaves),
  the CPU's `TreeTable::grows` (which the axe and the fruit read, so a tree
  that is not drawn cannot be chopped or picked), and the clutter rule
  (`has_clutter`).
- **The tide line is read from the water settings.** `water.ron` is loaded
  before any record is built and sets the line once
  (`planet::terrain::set_high_water`); a test or tool that loads no settings
  reads the shipped defaults.

**Saves.** Nothing generated changes and nothing new is saved:
- no generator version moves, and the v4 and v5 ground digests and the tree
  digests (`every_version_keeps_where_its_trees_stand`, which folds bits up
  to 24) are untouched;
- a `tree` record (a felled or picked tree) in the band is kept, and its tree
  is simply not drawn, as in 12.10.

**What it leaves.**
- A beach's trees now start at the 2 m line. That is a look change on every
  beach, which the owner's question asks for.
- A tide range set to nought in `water.ron` puts the line at the sheet,
  0.5 m under the sea level, which is what the sea covers then.

**Verify.**
- A test at the owner's spot: no tier or far record under the high-water
  line grows a tree or clutter. It fails before: 29 tier trees.
- A test that bit 25 stops `grows_tree`.
- A shader test holding the visibility and surface passes to the bit.
- Before and after `--capture` of the shore in `docs/screenshots/water-gameplay/`.

### 12.12 Ore drawn as ore: the placed-sheet field's spare value (task 9.9; owner, 2026-11-18)

Roadmap phase 2.1, "Ore drawn as ore in a dug wall (it shows as stone)".

**Finding.** 12.8 proposed a fifth bit in the column's material word. Before
taking that, the other per-layer field was measured:
- `placed-blocks-keep-material` added a second four-bit field per layer, the
  PLACED sheets (`ColumnTier::gpu_placed_sheets`, `PLACED_SHEETS_BASE`,
  `placed_sheet_at` in `planet_surface.wgsl`), and the same four bits in the
  record's material word at bit 26 (`PLACED_SHEET_SHIFT`) for the cap. It
  holds nought for generated ground and a placed block's sheet plus one.
- There are 14 tilesets, so it holds 0 to 14. **Fifteen is never written**:
  the only guard was `TILESETS.len() < 16`.
- The four-bit render codes are a different story. A per-layer nibble never
  holds 4, 7, 9, 11 or 12 (`render_code` returns none of them), but the
  shader's code space is shared with the record's eight-bit cap code and the
  face codes it derives (4 is the desert's ground, 9 the leaves, 11 and 12
  the sod and snow sides). Taking one for ore means teaching every reader of
  a code what it now means in which buffer. Not a cheaper route.

**Decision.** The spare value 15 is `ORE_SHEET` (`planet_terrain::ORE_SHEET`,
the WGSL `ORE_SHEET`): "this stone is ore".
- **Rust.** `planet_column::drawn_sheet_mark(column, layer)` is
  `ORE_SHEET` for an ore layer, generated or placed, and the placed sheet
  (`placed_sheet`, unchanged) otherwise. `pack_placed_sheets` and the
  record's bits in `reconcile_surface` write it. `placed_sheet` keeps its
  meaning ("somebody placed this"), so the drowned-bed rule is unchanged.
- **The layer keeps the stone's code (5).** Every rule that reads a code
  sees stone as before: the side rule, the plant tint, the reflections'
  `vox_albedo`, the visibility and rain passes. Only the albedo changes.
- **Shader.** Where the mark is 15, the face keeps its biome's sheet, and
  stone faces (code 5) get ore flecks. The cap (kind 0), the heightfield's
  top metre (kind 1) and a column-pass face (kind 4) read it, as they read
  a placed sheet.
- **Guard.** `TILESETS.len() < ORE_SHEET` at compile time, so a fifteenth
  tileset fails the build instead of turning into ore. That is the price:
  the field is now full, and the next tileset widens it.

**Why not the fifth bit.** Six codes a word turns 63 words a column into 84:
the material buffer, its upload and `PLACED_SHEETS_BASE` grow by a third for
every column, for one material, and the packing changes in Rust and in every
WGSL reader. The spare value costs no byte and no new layout.

**The look.** Procedural in the shader, on the stone tile's own 32-texel
grid, rather than an ore tile on the atlas. The atlas has no free tile: every
sheet's 16 slots are named materials (`assets/tilesets/biomes.json`). An ore
tile would need new art on all 14 sheets, or a fifteenth sheet that draws
one stone everywhere. The flecks keep each biome's own stone and overwrite
only some of its texels:
- the tile is cut into 4 by 4 texel blocks, and a block holds a nugget on its
  own hash: about a third of blocks;
- a nugget is two to five texels, never smaller than the stone's own grain;
- the ore is a third coal (near black), a third copper (rust orange) and a
  third tin (a cool steel grey, which reads on pale stone as well as dark), as
  the mixed ore yields;
- each colour is shaded by the stone texel under it, with its upper left
  lit and its lower right in shade (the light the sheets are drawn under),
  so the ore stays in the stone's palette and value range;
- no outline (owner, 2026-11-15);
- the hash takes the cell and the layer, so neighbouring blocks differ, and
  the pattern is fixed per block face (nearest sampling, nothing moves).
The far fade to the flat mean is the stone's, unchanged.

**What this leaves.**
- **A placed ore block takes the stone of where it stands**, not its home
  sheet (the fields'). Fifteen carries no sheet, and ore is a vein in the
  local rock. This is the one exception to `placed-blocks-keep-material`.
  Mining is unchanged: the CPU column says `Ore`, and `station::ore_yield`
  decides the drop.
- The far records (`surface_code`) never show ore. A generated top is never
  ore; only a dug face shows a vein, and that is a tier column.
- Reflections (`vox_albedo`) draw ore as plain stone.
- Saves: nothing stored changes. The mark is derived from the column's
  material on every pack, so an old world shows its veins on open, and a
  version 9 world has none to show.
- Frame cost not measured (cloud session, no GPU). The pack now walks every
  layer of every column for ore, as `pack_materials` already does. The
  shader adds a hash and a branch on stone faces only.

**Verify.**
- Rust tests on the packing:
  - an ore layer packs 15 and its neighbours 0;
  - a placed ore block packs 15;
  - an exposed ore top's record carries 15;
  - a placed stone still packs its sheet;
  - generated ground with no ore packs nothing.
- An `include_str` test holding the WGSL's `ORE_SHEET` to the Rust constant.
- Shader validation.
- Before (`claude/river-cache-full`) and after captures of a dug wall with a
  vein, in `docs/screenshots/ore-veins/`.

## 13. Trees in rain, and a fade that ends (owner, 2026-11-16)

The owner, on a terraced valley in rain: "theres massive artifacts here";
then "distant forest stuff has that weird pale ghost look too" (a jungle in
rain), "no rain drops on tree leaves either" (looking up into an oak), and
"dithering should either stay or go, dont have it linger if the player isnt
moving" (a jungle at night, the camera still). The pictures are in
`docs/screenshots/rain-artifacts/` (`owner-*`), and the captures that
reproduce them are listed in its README.

### 13.1 What the ghost trees are

Two causes, measured apart.

**The wet ground's whole block runs on the trees.** `planet_surface.wgsl`'s
rain branch (`wet_amt > 0.001`) gates on `rain_open`, which is 1 for every
tree vertex, and its puddle test `floor_like` is "not grass, not leaves"
by material code, which every tree code passes (`TREE_BARK_CODE` to
`IMPOSTOR_CODE` are none of 2, 3, 7, 9). A mid card and a far billboard
take the tree's up axis as their normal (`normal = axis`; the billboard
`0.4` sideways), so to the wet block they are a flat floor: a puddle
(`flat` 1 on a card, about 0.5 on a billboard), with the full Fresnel
mirror of the sky at the grazing angle a standing card is seen from, the
rings, the sheet's waves, the GGX glint and a reflection request. A card
in rain is drawn as a pale sky-coloured sheet in the shape of a tree: the
"ghost trees". The near model's leaf cards turn to the eye, many of them
tilted up, and take the same rings, sheet and sheen ("no rain drops on tree
leaves"). Evidence: `before-valley-dry.jpg` against `before-valley-rain.jpg`,
one spot, one frame count, the forcing the only change: every mid and far
tree turns from green to pale grey-blue blotches.

**The cross-fade dither lingers.** The near model, the mid cards and the
far billboard cross-fade over the last 0.3 of each ring (`ring_t`, design
10.4) by a Bayer mask, by distance. A tree that stands inside a ring is
drawn half in one representation and half in the next for as long as it
stands there: on a still camera, a permanent screen-door of tree pixels and
background pixels. Against a pale sky or the rain's haze that reads as a
grey checker ("pale dithered checker patches at the canopy line"). Evidence:
`before-jungle-rain.jpg` with the clouds pass off (`PBD_PASS_OFF=clouds`)
and with the rain passes off still shows the checker behind the near
kapoks, so it is neither the clouds' upsampling nor the rain volume. The
foliage range's outer edge thins trees by the same mask over
`tree_fade_m` (the same lingering, at a longer range).

### 13.2 The fix for the first: a tree is not a floor

The rain branch gives a detailed tree (`kind == 2`) and every piece of
ground clutter (`kind == 3`: tufts, blades, flowers, ferns, mushrooms,
pebbles, fallen fruit) one thing only: a uniform darkening by
`rain_plant_darken` (`weather.ron`, 0.88 at full wetness) times the
wetness. No puddle, rings, sheet, rivulets, mirror, glint, lamp glints or
reflection request. Bark takes the same: the weather-rain design's
"rivulets down trunks" were Tenebris's prism trunks, and on the detailed
trees' 20 cm limbs the kernel is noise; recommendation taken (ask only with
screenshots).

### 13.3 The fix for the second: a fade is a moment in time

The rule (CLAUDE.md, priority 1, owner 2026-11-16): "A dither cross-fade
starts when something crosses its distance, runs to the end over its fade
time, and finishes. Nothing is drawn half-dithered for as long as it sits
inside a distance band, and a still camera sees no dithered object once the
fade time has passed."

**Two camera snapshots, no per-tree state.** The CPU keeps two points in
the body's frame, `from` and `to`, and a clock. Each tree's representation
is decided at both points by hard thresholds (no ring): the near model
inside `0.85 * tree_near_m`, the mid cards inside `0.85 * tree_mid_m`, the
far billboard beyond, and nothing past the foliage range's edge (its
`TREE_FADE_MARGIN_M` and half the old thinning band inside the reach). If a
tree's representation at `from` equals the one at `to`, it is drawn whole in
it: no dither at all. If they differ, the one at `to` comes in and the one
at `from` goes out through the existing Bayer split, by the fade's progress
`t = (now - start) / fade_s`, not by distance. When `t` reaches 1 the CPU
sets `from = to`; while the camera has moved since, it then sets `to` to the
camera and starts the next fade. A still camera therefore has `from == to`
one fade after it stops: nothing dithered. Hysteresis comes free: the
snapshot moves only with the camera, so a tree beside a threshold cannot
flap while the camera stands.

**Fade time and speed.** `tree_fade_s` 0.5 s (`scatter.ron`). The level is
decided at `to`, which trails the camera by at most the fade time; so the
fade is shortened to keep that lag under `tree_fade_lag_m` 5 m
(`fade = min(tree_fade_s, tree_fade_lag_m / speed)`): 0.5 s on foot, about
0.17 s at 30 m/s, a frame or two in flight, where the camera's own motion
already hides a fade. The 5 m lag sits inside the margins the visibility
pass lists with (the near list to `tree_near_m`, the cards from
`0.7 * tree_near_m` to `tree_mid_m`, the billboard past `0.7 * tree_mid_m`),
so both representations of a tree in its fade are always listed: the
switches at 0.85 of each range leave 6.6 m and 13.5 m either side.

**What stays.** The billboard's growth past the fine band
(`distance-lod-fade`) is a size, not a dither, and stays. The terrain's own
band cross-fade (`fade_t`) and the clutter's reach are spatial too; the
terrain's is measured and written up in `distance-lod-fade`; the clutter's
reach takes the same two snapshots (a piece is shown or not at each, faded
by `t` between).

**Verify.** A still camera in a forest captured at 0 s, 0.3 s and 2 s after
it stops (`PBD_CAPTURE_RUN` frames): dithered tree pixels at 2 s, none. A
walk through a forest shows no pop (every change fades over its time). A
launch to READY with no `LATE_PIPELINE`.

### 13.4 The far trees glow at night

The owner, at night by a jungle and the sea: "distant trees appear to be
glowing evenly, there should be some sort of lighting? idk"
(`owner-far-trees-glow-night.webp`). Reproduced at the owner's spot
(`--walk --at 28.2398 -19.596 --yaw 330 --pitch -5 --time 2`,
`before-far-trees-night.jpg`): the mean sRGB of the far band is
(0.4, 12.9, 14.1), the near kapoks' crowns beside it (0.5, 9.5, 12.4), the
ground (0.3, 7.6, 22.8). The far band is a third brighter than the near
crowns and flat.

**Cause.** The impostors are lit by the terrain's own fragment, like the
near trees, the ground and the cascades' shadow, so they do take the sun,
the sky, the lamps, the field's sky light and the haze; what differs is the
normal. A mid card's is the tree's up axis, the billboard's nearly so
(`0.4` toward the eye). Lit as a floor, a card takes the zenith's full sky
irradiance and the sun from above, all of it the same on every pixel, while
the near crown's leaf cards turn to face the eye and take the light of a
standing face, which at night is a fraction of the zenith's. The bake's
brightness was matched to the near crown's mean (design 10.7), so the whole
difference is the normal.

**Fix.** Both impostors take a standing normal: toward the eye across the
ground, tilted up by `0.35` (the near crown's leaves face the eye and lean
a little skyward), so a card is lit as the near crown's leaves are, the
sun-side brighter and the far side darker. Nothing else of the lighting
changes. Verify: the same view at night and by day, near and far trees in
one frame, before and after, with the band's mean against the near crowns'.

### 13.5 Where a plant's colour comes from (owner, 2026-11-16)

The owner: "the tinting is because you are sharing ground tint with
vegetation, really the tint should be like based on texture not
shading/lighting/reflections", and "very pronounced trees tinted by sun".
Checked every place a plant could borrow the ground's colour:

- **`ground_tint`** (the season's dry and frost on the grass) runs only on
  `kind` 0, 1 and 4 (caps, walls, cut walls): no plant takes it.
- **`flora_albedo`, `plant_art`, `tree_albedo`**: a plant's albedo is its
  own texels (the tuft ramp, the flower, fern and fruit palettes, the kit's
  bark and leaf tiles pulled to their means, the bake's pixel matched to the
  near crown's mean) and the season laid on that art. None reads a lit,
  hazed or reflected term.
- **The bake** (`pbd_core::tree_bake::shade`) carries a fixed key: 1.15 on
  every leaf (a scale the shader's mean match takes out) and 0.85 to 1.15 on
  bark by its normal. A shape term in texture space, not the scene's light;
  kept.
- **What did borrow the ground's look was the wet block** (13.2): the
  puddle mirror of the sky, the glint toward the sun and the rings, all run
  on plants as on a floor. Toward a low sun on wet ground that is the
  bleached, pale silhouettes ("trees tinted by sun"); in rain, the ghosts.
  Gone with 13.2. Measured with the ground soaked under a clear sky and a
  sun 16 degrees up (`before/after-soaked-toward-sun.jpg`).
- **The light** is then applied once, by the plant's own normal: a near
  leaf card's (turned to the eye), the impostors' standing normal (13.4),
  the clutter's surface up (design 9: a piece shades as the cap it stands
  on, so it does not pop against it; its colour is still its own).

### 13.6 As built (2026-11-16, branch `claude/rain-artifacts`)

- 13.2: `plant` in the rain branch of `planet_surface.wgsl`;
  `rain_plant_darken` 0.88 in `weather.ron` (`rain_near.z`).
- 13.3: `TreeFade` in `planet.rs` (two points, a progress, `tree_fade` in
  the uniform, which `planet_visibility.wgsl` and `rain_splash.wgsl` copy);
  `tree_level` and `tree_show` in the shader; the foliage edge and the
  clutter reach by the same two points; `tree_fade_s` 0.5,
  `tree_fade_lag_m` 5 in `scatter.ron`. Test:
  `the_trees_fade_by_time_and_a_still_camera_is_whole`. Instrument:
  `PBD_TREE_ARRIVE=<m>@<s>`; `seq-tree-fade-still-camera.jpg` shows the
  dither at 0.1 and 0.3 s and none at 2 s. While a fade runs, the dropped
  pixels of a tree whose two representations differ in outline show what is
  behind (half a second, then gone).
- 13.4: the impostors' normal `t.x*dir.x + t.z*dir.y + axis*IMPOSTOR_LIFT`
  (0.35). Night, leaf pixels' mean: far band 15.6 to 13.8 against the near
  crowns' 16.5.
- Frame cost not measured (a Mac sitting is owed with the branch; the
  changes remove work from plants in rain and add two distance tests a tree
  vertex).

### Pixel tufts are the only grass (owner, 2026-11-16)

The owner: *"grass should be pixel tufts, remove the other grass"*, then *"no
setting to swap, just the new stuff"*. The blade grass is removed: from the
surface shader, the Pixel grass row on the settings screen, `FloraLook` and
its system. `--grass` now takes only `tufts`, so old capture recipes still
run, and `blades` is refused. A settings file's `pixel_grass` value is still
read, so old files load, but nothing reads it any more.

