# The water life's atlas

`atlas.png` (512 x 512, sRGB, sampled nearest with no mipmaps) and its
manifest `atlas.ron` are written by `tools/water_life_atlas.py`
(`ocean-environments` design B14); `--check` holds the committed files to it.
Nothing here is hand-edited or taken from outside the repository.

- **Fish skins** (64 x 32, four seeds a species) and **fin tiles** (M9's 8 x 8
  tile, four times each way): M9's own `paintSkin` from
  `docs/mockups/fish.html` (Fish Lab and Aquariums), ported to Python and
  checked pixel for pixel against the page's JavaScript under three.js r128.
  Every colour is snapped to Sweaft-64, the balatro-pixel-art kit's palette.
- **Reef tiles** (32 x 32): drawn in the generator as grey multiply masks
  (four greys, 168 to 255): polyps, knobs, brain, plate, pores, lattice,
  stripes, veins, grit, ridges, basalt, ice, mat, and a white tile for
  untextured faces. The piece's vertex colour carries its palette.

The same PNG and the manifest as JSON are copied to `docs/mockups/fish/`
(`water_life_atlas.png`, `water_life_atlas.json`), so M9 and the game wear the
same pixels.
