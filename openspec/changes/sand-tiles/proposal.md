# Proposal: redrawn beach sand, a desert sand of its own

## Why

The owner, 2026-11-19: "can you redo sand texture? its got a lot of these
weird blue shells. also desert sand and beach sand should be different", and
"beach would have shells and stuff".

What the game has today (read in the code and the atlas, 2026-11-19):

- **The blue shells are the beach sheet's.** `assets/tilesets/beach.png`,
  baked into `atlas.png` slot 2 by `tools/build_tileset_atlas.py`. Its top
  tile (0,0) "dry shell sand" is pale ivory speckled with teal and turquoise
  flecks (the sheet's `accent` in `biomes.json` is "pale turquoise sea
  glass"). Its side (1,0) "dry over wet sand side" and its wet sand (2,0)
  carry the same teal flecks. Those flecks are what reads as blue shells.
- **The two sands are close.** The desert sheet's top (0,0) "pale sand" is a
  pale ochre (tile mean `#e9c083`) beside the beach's pale ivory (`#dbc596`).
  Lit, at a distance, they read as one sand.
- **There is one sand block.** `Material::Sand` is the only sand. Generated
  ground draws its biome's sheet, so a beach and a desert each show their own
  top, but `pbd_core::column::home_biome` maps `Sand => Beach`: a sand block
  dug anywhere is the beach's sand, its pack icon is the beach tile, and put
  back down in a desert (`placed-blocks-keep-material`) it draws the beach
  tile, teal flecks and all, in the middle of the dunes.

## What changes

1. **The beach sand is redrawn**: warm pale sand with a scatter of small
   shells and shell bits in cream, white and soft pink and coral, a pebble and
   driftwood grit, no teal or blue anywhere, low in contrast and with no
   outline (design 1). The side and the wet sand under it match (design 2).
2. **The desert sand is redrawn to read as another sand**: warmer
   orange-ochre, fine wind-sorted grain, faint ripples, no shells; its side's
   pale band takes the same sand over the sheet's own strata (design 3).
3. **Sand dug in a desert is desert sand** (recommended, design 5): a second
   block, `Material::DesertSand`, given by the dig rule for generated sand in a
   desert cell, drawn from the desert sheet wherever it is placed. Generation
   is untouched, so no new generator version; old saves keep their sand.
4. The far colours, tuft ramps and the beach sheet's manifest follow the new
   tiles (design 4).

The look (1, 2) is shown on M13's new Sand tab, Today against New, by day and
at night, and waits for the owner's approval before anything goes into
`assets/tilesets/`. Nothing in the game changes in this commit.

## Approved (2026-11-20)

The owner, in chat: "approve sand and mountains, and rivers". The Sand tab
(M13 version 4) is approved for the game, with design 5's recommendation (c);
built on `claude/phase-2-1`. Desert sand takes save code 29, the next free
one after the sea's cells and sticks (24 to 28).

## Capabilities

- `planet/rendering`: ADDED "Beach sand and desert sand are drawn apart".
- `world/editing`: ADDED "Sand dug in a desert is desert sand".

## Impact

- Art: `assets/tilesets/beach.png` (0,0) (1,0) (2,0), `desert.png` (0,0)
  (1,0), and the rebaked `atlas.png`; `biomes.json`'s beach accent.
- `pbd_core::flora::TUFT_RAMPS` (beach, desert rows) and the far colours of
  codes 1 and 4 in `planet_surface.wgsl` move with the tiles; the test
  `the_plants_colours_are_the_atlas_s` prints the new rows.
- Design 5 adds one `Material` at the end of the enum and the save's material
  table (forward only), touches `yield_of` in `desktop/digging.rs`, and every
  exhaustive match on `Material` (58 `Material::Sand` sites in 22 files).
- Saved games: unchanged. A save's sand stays sand. No generator version.
- Mockup: `docs/mockups/foliage.html` (M13) version 4, the Sand tab;
  candidate tiles and their script in `docs/mockups/sand/`.
