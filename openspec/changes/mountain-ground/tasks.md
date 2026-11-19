# Tasks

## 1. Before anything moves

- [ ] 1.1 A capture of a mountain flank from the air and from the ground
      (version 10), to confirm what the tan rows are.
- [ ] 1.2 V11's ground digest pinned (both targets). Version 11 is
      `swamps` (2026-11-18); this change is version 13.

## 2. Version 13

- [ ] 2.1 `TENEBRIS_V13` and `GENERATOR_VERSION = 12`; flat Mountains cells
      take snow, alpine grass or dirt (decision 1); tests: the tiers agree,
      flats get grass or snow, steep cells stay stone.
- [ ] 2.2 Climate files for g12.

## 3. The draw

- [ ] 3.1 Alpine soil for grass and dirt on Mountains (decision 2).
- [ ] 3.2 Rock tile per cell by banded altitude and noise (decision 3).
- [ ] 3.3 Risers' far colour (decision 4).

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
- [ ] 5.3 The owner's look at the Mountains tab.

## 6. Mountain flora in the game (after approval and a request)

- [ ] 6.1 Alpine turf into `mountains.png` (3,2), the atlas rebaked;
      `TUFT_RAMPS` row 6 from the test's printout.
- [ ] 6.2 Clutter on Mountains cells (design 8): the sprites, the flower bits,
      `BIOME_FLORA` row 6; a test that the rolls and the gather rule are
      unchanged.
- [ ] 6.3 Lichen, scree and moss in the rock (design 10); snow patches and ice
      (design 11).
- [ ] 6.4 `TreeTable::V13` with the tree line, and the scoured ridges in
      `top_material`; Fir and Krummholz in `tree_kit` with their bakes
      (design 12); tests: the CPU and the pass agree, no tree above the line.
- [ ] 6.5 Before-and-after captures of a range, in the air and on foot, into
      `docs/screenshots/mountain-flora/`; the frame cost on the owner's
      hardware (a mountain walk, before and after).
- [ ] 6.6 Proposals for the owner when asked: tarns, cairns, bilberry as
      forage, boulders taken only by pickaxe.
