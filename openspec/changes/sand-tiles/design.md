# Design: sand-tiles

The owner, 2026-11-19: "can you redo sand texture? its got a lot of these
weird blue shells. also desert sand and beach sand should be different", and
"beach would have shells and stuff".

Candidate tiles: `docs/mockups/sand/sand_tiles.py` draws them and writes
`sand_tiles.png` (today and new, beach and desert) and `sheet.png` (each tile
tiled 4 x 4). They are shown on M13's Sand tab (`docs/mockups/foliage.html#sand`,
version 4). Nothing is written into `assets/` until the owner approves.

## 0. Which tiles the game draws

Read off `planet_surface.wgsl` (the material table above `pixel_tile`),
`face_code`, `desktop/slots.rs` and the tests that read the atlas:

| Tile | Drawn where | Redrawn |
| --- | --- | --- |
| beach (0,0) dry shell sand | every face of sand in a beach cell (top, wall, and down to `ground.y`, 4 m: sand "shows its own picture on every face"); a drowned beach top (code 0) under the sea; **every placed sand block anywhere**, and the sand pack icon (`home_biome(Sand) = Beach`); the beach's tuft ramp (`TUFT_RAMPS[1]`) | yes |
| beach (1,0) dry over wet sand side | the first metre of a grassy cap's wall in a beach cell (`GRASS_SIDE_CODE`): rare, but drawn | yes |
| beach (2,0) wet sand | earth (`DIRT_CODE`) under a non-sand cap in a beach cell: rare, but drawn | yes |
| beach (3,0) coastal sandstone | stone under the sand | no (no teal in it) |
| beach (2,1), (0,2) | a beach tree's bark, and `PLANT_MEANS` | no: a beach crown wears the fields' leaves in the dry olive (`plant_art`), and neither tile is sand |
| desert (0,0) pale sand | every face of desert sand (code 4) to 4 m; `TUFT_RAMPS[3]`; placed desert sand (design 5) | yes |
| desert (1,0) stratified sand side | a grassy cap's wall in a desert cell | its pale band only |
| desert (2,0) ochre grit, (3,0) red sandstone | earth and stone under the desert | no: already orange and red, nothing blue |

Other teal in the beach sheet ((1,1) shell bed, (0,3) pebbles, (2,3) sea
glass) is on tiles the game does not sample, and is left alone.

## 1. The beach top: shell sand, no blue

- **Sand**: a five-step ramp off the sheet's own commonest top texel,
  `#e2cb98`: `#cdb383 #d8bf8e #e2cb98 #e9d4a4 #efdcb0`. Most texels sit on the
  middle step; wrapped value noise nudges patches one step lighter or darker
  and single texels fleck a step either way, so it is sand and not paper.
- **Shells**, about eight a tile, each 3 to 5 texels: a scallop, a clam, a
  small spiral and an auger, in four warm ramps (light, mid, dark): cream
  `#fffaeb #f4e6c6 #cdb68c`, white `#ffffff #f1ece2 #cfc1a6`, pink
  `#fbe0d4 #eeb7a2 #cc937c`, coral `#f8cdb4 #e59d80 #c0806a`. Lit from the top
  left as the sheets are. A shell's shadow is one sand step down (`#d3ba88`),
  never an outline.
- **Bits**: shell fragments of one to three texels, a pebble in warm grey
  (`#cbb89c #b3a084 #9c8a70`), and driftwood grit (`#a4865f`), sparse.
- **No blue**: no texel whose blue exceeds its red or whose green leads its
  red; the script asserts it on every redrawn tile.
- Seamless: the noise is periodic in the tile and sprites wrap round its
  edges; the script checks the step across each wrap against the steps inside.

The new top's mean is `#e0c89b` (today `#dbc596`): the same brightness, so
the beach's far colour (code 1's `base`) still agrees with it.

## 2. The beach's side and wet sand

- (2,0) wet sand: a damp ramp off the sheet's own `#9b7b57`
  (`#88694a #927353 #9b7b57 #a58560 #b0916b`) with fewer, duller shell bits
  (the dry colours less 38 per channel) and a pebble. No teal.
- (1,0): the dry top's sand and bits over that wet sand, on a wavy edge that is
  periodic across the tile, with a one-texel dry foot and a damp line under it.

## 3. The desert top: another sand

- A warmer orange-ochre off the desert sheet's own colours (`#d59a58`,
  `#e4c78e` in `biomes.json`): `#c4824a #d39556 #dea464 #e6b373 #edc184`.
- **Fine, wind-sorted grain**: the same grain as the beach but with no shells
  and no pebbles; a few rust grains (`#b9693f`) and quartz glints (`#f2cf98`).
- **Faint ripples**: four crests down the tile and one turn across it (so they
  wrap), warped by periodic noise; a lee face one step darker and a crest one
  step lighter on part of their texels. Low contrast: from a walking height
  they read as texture, not stripes.
- Its mean is `#dca365` (today `#e9c083`), linear `(0.72, 0.37, 0.13)`: code
  4's far colour in the shader is already `(0.64, 0.36, 0.13)`, so the near
  tile now agrees with what the desert fades to at range, where today it is
  paler than its own far colour.
- (1,0): today's strata kept texel for texel; the pale band over them takes the
  new sand.

## 4. What moves with the tiles

- `TUFT_RAMPS` (beach and desert rows) are read off the atlas by
  `the_plants_colours_are_the_atlas_s`, which prints the rows to paste. Off the
  candidates: beach `[0xd8bf8e, 0xe2cb98, 0xe2cb98]`, desert
  `[0xd39556, 0xdea464, 0xe6b373]`. `PLANT_MEANS` read (0,2), (2,1) and (3,0)
  and do not move.
- The far colours of codes 1 and 4 stay (sections 1 and 3).
- `biomes.json`: the beach's `accent` "pale turquoise sea glass" becomes "pink
  and white shell"; its tile names stay.
- `tools/build_tileset_atlas.py --check` holds the rebaked atlas to the sheets.
  The sheets are 1254 px sources sampled at the shader's 32-texel grid, so the
  tiles are written into the sources at that grid and the atlas rebuilt; the
  check then compares.

## 5. Placed sand: one block, or two

The redraw alone makes generated beach and desert different. A placed or
dug sand block is the question: `home_biome(Sand) = Beach`, so sand dug in a
desert and put back down there is beach sand with shells, and the pack calls
both "sand".

**(a) A second block that generation places.** `Material::DesertSand`,
written by the generator into desert columns. A desert column's material is
generation's output, so this is a generator change: a new generator version
for new worlds, the old versions carried and digest-pinned. Every world made
before it keeps one sand. Heavy, and it buys nothing a player sees that (c)
does not.

**(b) One block, only the redraw.** Placed sand is beach sand everywhere. The
desert's own ground looks right, but a filled-in hole in the dunes is a patch
of shell sand, which is the very "desert sand and beach sand should be
different" the owner asked for, at the scale of one block.

**(c) A second block that the DIG gives (recommended).** Generation is
untouched: a desert column still holds `Material::Sand`, drawn from the desert
sheet as today. The difference lives in the rule that turns a dug layer into a
pack item, `desktop::digging::yield_of`, which already asks whether a layer was
placed (for ore):

- generated `Sand` dug in a cell whose biome is the desert gives
  `Item::Block(Material::DesertSand)`;
- placed `Sand` gives sand wherever it lies, and placed `DesertSand` gives
  desert sand;
- `home_biome(DesertSand) = Desert`, so it is drawn from the desert sheet when
  placed (`placed-blocks-keep-material`), and its icon is the desert top.
  Render code 1 (sand) with the desert's placed sheet: no new shader code.
- It is named "desert sand" in the pack; it digs, sounds, bounces and gathers
  bait as sand does (a helper `Material::is_sand` replaces the `Sand` arms
  where both should act alike; recipes that take sand take either).

Saves: `DesertSand` is a new material at the end of the enum and the save's
material table (`saves/format.rs`); nothing existing is renumbered. An old
save's placed sand stays sand and is drawn as the redrawn beach sand. No
generator version; no world changes under the player.

Recommendation: **(c)**. Recommendation taken (ask only with screenshots): the
choice between (a), (b) and (c) has no picture of its own beyond the tiles,
which the owner approves on the Sand tab.

## 6. Mockup

M13's Sand tab (`foliage.html#sand`), its own renderer made when the tab opens
and stopped when it closes, the other tabs' loops stopped while it is open:

- a hex patch: a beach running down into a shallow sea, and dunes of desert
  behind a rise, both drawn as the game draws sand (the top tile on every face
  to 4 m, stone under it, top UVs over 1.5 tile widths, nearest texels);
- **Beach: Today / New** and **Desert: Today / New** switches, and a
  **Compare** view that splits the screen today | new;
- **Day / Night**: at night a dim moon, and torches and lanterns on the sand
  lit with the lamp falloff the game bakes, each lighting what is near it;
- **View**: the beach, the desert, or the line between them; drag to orbit,
  nothing turns on its own;
- a flat 2D panel of every changed tile, today beside new, each tiled 4 x 4;
- the notes in an info modal.

Screenshots in `docs/screenshots/sand-tiles/`, rendered on SwiftShader in a
cloud container (no GPU): they show the look, never a frame time.
