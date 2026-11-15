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
- **Fungal forest** (violet caps, warm stalks, dark violet moss). No game
  biome is a fungal forest; a new biome is a new generator version, its own
  change (Q7). Not built.
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
