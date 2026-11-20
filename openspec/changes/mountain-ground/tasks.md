# Tasks

## 1. Before anything moves

- [ ] 1.1 A capture of a mountain flank from the air and from the ground
      (version 10), to confirm what the tan rows are.
- [x] 1.2 V11's ground digest pinned (both targets). Version 11 is
      `swamps` (2026-11-18); this change is version 13.

## 2. Version 13

- [x] 2.1 `TENEBRIS_V13` and `GENERATOR_VERSION = 12`; flat Mountains cells
      take snow, alpine grass or dirt (decision 1); tests: the tiers agree,
      flats get grass or snow, steep cells stay stone.
- [x] 2.2 Climate files for g12.

## 3. The draw

- [x] 3.1 Alpine soil for grass and dirt on Mountains (decision 2).
- [x] 3.2 Rock tile per cell by banded altitude and noise (decision 3).
- [x] 3.3 Risers' far colour (decision 4).

## 4. Proof

- [ ] 4.1 Before-and-after captures of the same flank, in the air and on
      foot, in `docs/screenshots/mountain-ground/`; the owner's look on 3.2.

## 5. Mountain flora: write-up and mockup (2026-11-19)

- [x] 5.1 Design 5 to 14: the bands, generation against draw, the clutter
      slots, alpine turf, lichen, scree, snow patches, `TreeTable::V13`.
- [x] 5.2 Mockup: M13's Mountains tab (`docs/mockups/foliage.html#mountains`,
      version 5), Today against New, day and night with a campfire and
      lanterns, layer toggles and a snow-line slider; the turf and plant art
      drawn by `docs/mockups/foliage/mountain_art.py`; screenshots in
      `docs/screenshots/mountain-flora/` (SwiftShader, no GPU).
- [x] 5.2a Version 6, on the owner's "idk what this texture is supposed to be
      but its overlapping, also you should use the foliage kit instead of
      making new plants": the scree's streaks fixed (the tile id was not
      rounded before its sheet and column were taken, so texels came from the
      next sheet pixel by pixel; the lichen hashed a per-pixel position), snow
      patches, lichen and moss on their tile's own texel grid, stones
      projected square; the plants are the Playground's kit (tufts, flowers,
      bushes, the pine, rocks, the dead shrub, the blueberry plant) and
      krummholz the pine's builder stunted and swept; the turf alone is drawn
      by `docs/mockups/foliage/alpine_turf.py`; shots
      `docs/screenshots/mountain-flora/v6-*.png`.
- [x] 5.3 The owner's look at the Mountains tab: approved for the game (owner,
      2026-11-20, in chat: "approve sand and mountains, and rivers"; version 6).

## 6. Mountain flora in the game (after approval and a request)

- [x] 6.1 Alpine turf into `mountains.png` (3,2), the atlas rebaked;
      `TUFT_RAMPS` row 6 from the test's printout.
- [x] 6.2 Clutter on Mountains cells (design 8): the sprites, the flower bits,
      `BIOME_FLORA` row 6; a test that the rolls and the gather rule are
      unchanged.
- [x] 6.3 Lichen, scree and moss in the rock (design 10); snow patches and ice
      (design 11).
- [x] 6.4 `TreeTable::V13` with the tree line, and the scoured ridges in
      `top_material`; Fir and Krummholz in `tree_kit` with their bakes
      (design 12); tests: the CPU and the pass agree, no tree above the line.
- [ ] 6.5 Before-and-after captures of a range, in the air and on foot, into
      `docs/screenshots/mountain-flora/`; the frame cost on the owner's
      hardware (a mountain walk, before and after).
- [ ] 6.6 Proposals for the owner when asked: tarns, cairns, bilberry as
      forage, boulders taken only by pickaxe.

## 7. Built as version 15 (2026-11-20, `claude/phase-2-1`)

Approved by the owner in chat ("approve sand and mountains, and rivers").
Version 14 went to `ocean-environments`, so everything above that says 11,
12 or 13 for this change was built as **generator version 15**
(`TerrainConfig::TENEBRIS_V15`, `GENERATOR_VERSION = 15`) on version 14's
config; 13 stays unused.

- 1.2 / 2.x: version 14's top digest pinned before 15 landed
  (`version_14_keeps_its_top_blocks`; its ground is version 12's pinned
  digest). `pbd_core::mountain` holds the relief rule, the ragged snow line,
  the flats, the scoured crests and the tree line. Climate g15 is g14's
  (`climate_ground` strips the mountain ground); the city survey reads
  version 14's ground. Measured on 2,000,000 points: a version 15 range is
  66% turf, 4% soil, 12% snow and 19% stone (`examples/mountain_places.rs`),
  since its steps are mostly under a metre.
- 3.x / 6.1 / 6.3: draw-time, every world. The turf is in `mountains.png`
  (3,2) (its "worn stone path", read by nothing, renamed "alpine turf"),
  written by `tools/paint_sheet_tile.py`; grass draws the turf, earth the
  alpine soil (2,3), a grass riser the fields' sod side; rock by tilted 12 m
  strata (slate, granite, dark shale, fractured slate), lichen stone in
  patches, snow-dusted rock from 170 to 190 m, scree on a cell with a
  neighbour 3 m above (the record's `SCREE_BIT`), lichen in the texels, moss
  in the lowest fifth of each metre of a rock wall under 156 m, snow patches
  on flats (`FLAT_BIT`) in the 14 m under the line, ice on snow over 205 m,
  and the risers' far colours. The draw-time snow line is the mean, 170 m
  (`mountain::DRAW_SNOW_LINE_M`).
- 6.2: the slots as design 8, with these left: bilberry is not drawn (a bush
  is juniper or heather), the hardy flower is not drawn, and the two new
  flower bits wait on the owner (the kit's blue and red stand in).
  `BIOME_FLORA` row 6 is white, blue, yellow and red at fern 0.6 and
  mushroom 1.4. A boulder is still taken by hand like a pebble.
- 6.4: `TreeTable::V15` (mountains 60 in 256 on turf), the tree line in the
  record's word (`TREE_LINE_SHIFT`, `KRUMMHOLZ_BIT`), firs are the pine at
  0.55 to 0.85 of its size, and `Species::Krummholz` is a ninth near draw
  built by the pine's builder (variant 0 a flag tree) with its impostor
  bake. A mountain's trees bear pine cones.
- Not built (proposed only): tarns, cairns, bilberry as food, pickaxe-only
  boulders. Frame cost not measured (cloud session, no GPU).
