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
