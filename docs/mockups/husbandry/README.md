# Art for the husbandry mockup (`husbandry.html`, "Pens, Nests and Pedigrees")

Every file here is loaded by the page. Copies are byte-identical; never edit
them: if a source changes, copy it again.

## `game/`: the game's own art

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Hex tops and sides, read the way `planet_surface.wgsl` (`pixel_tile`) does: the fields sheet (slot 4 in `planet_terrain.rs` `TILESETS`), grass (0,0) on top, (1,0) the first metre of a side, earth (2,0) below and on paths, tilled soil (2,3) under the wheat |
| `timber.png`, `poles.png`, `boards.png`, `plank.png`, `plaster.png`, `whitewash.png`, `thatch.png`, `shingle.png`, `iron.png`, `bark.png`, `leaves.png`, `linen.png`, `wool.png` | `assets/textures/settlement/<name>.png` | Fence posts and rails, gates, coop, barn, stable, sty, hives, duck house, kennel, troughs, the farmhouse, the incubator, trees, the player's clothes; each at its `repeat_m` from `assets/textures/settlement/manifest.ron`, sampled nearest |
| `fish-<name>.png` (8) | `assets/items/fish/<name>.png` | The game's eight fish in the Kingdom |

## Other mockups' art (byte-identical; their designs are the source)

- `m7/`: from the animals mockup (M7, `docs/mockups/animals/`, drawn by
  `tools/art/animals_art.py`): the 24 land species' portraits; the hides of
  the woolback, plodder, ridge goat, red fox and meadow hen (`hide_*.png`,
  for the 3D bodies); the drops (`drop_*.png`: egg, milk, wool, meat, hide,
  horn, feather, glow, shell).
- `m25/`: from the seas, rivers and marshes mockup (M25,
  `docs/mockups/ocean/`, `tools/art/ocean_art.py`): 39 water creatures'
  sprites and the anemone card (`card-anemone.png`).
- `m9/`: from the fish lab (M9, `docs/mockups/fish/species-<name>.png`,
  renamed `<name>.png`): bloatfin, emberback, glassback, lanternjaw, sunplate.
- `m1/`: from the planets mockup (M1, `docs/mockups/planets/species/` and
  `planets/res_fleece.png`): Sequoia's, Wick's and Fallow's 14 species and
  Fallow's drover fleece.
- `m8/`: from the farming mockup (M8, `docs/mockups/farming/`): turnip and
  frost kale (favourite foods), the fertiliser icon (compost), and the ripe
  wheat stage (`wheat_3.png`, crossed cards on the tilled cells).

## Mockup-only, new here

Pixel art drawn in code by `tools/art/husbandry_art.py` with the
balatro-pixel-art kit (Sweaft-64, hard alpha), reusing M7's body-plan
functions. Run `python tools/art/husbandry_art.py` to redraw them.

- Portraits, 36 x 36: `yard_hen`, `cockerel`, `chick`, `rootling`,
  `fellrunner`, `herd_hound`, `hedge_cat`, `yard_dabbler`, `clover_bee`,
  `field_mouse`.
- Hides, 256 x 32 (eight 32 px tiles, M7's layout): `hide_yard_hen`,
  `hide_cockerel`, `hide_chick`, `hide_rootling`, `hide_fellrunner`,
  `hide_herd_hound`, `hide_hedge_cat`, `hide_yard_dabbler`.
- Products and feed, 24 x 24: `duck_egg`, `fertile_egg`, `honey`, `wax`,
  `manure`, `tallow`, `cut_steak`, `cut_chop`, `cut_ham`, `cut_drumstick`,
  `feed_grain`, `feed_hay`.
- Tools, 24 x 24: `tool_shears`, `tool_pail`, `tool_lead`, `tool_brush`,
  `tool_smoker`, `tool_incubator`, `tool_nest`.
- Needs, 16 x 16: `need_food`, `need_water`, `need_shelter`, `need_space`,
  `need_mood`, `need_health`.
- Floors, 32 x 32: `tex_straw` (nests, the broody hen's nest, feed troughs),
  `tex_mud` (the sty, the compost heap).

Water, glass and the sky are flat colour, as the game draws them.
