# Art for the seas, rivers and marshes mockup (`ocean.html`)

- `game/`: the game's own art, byte-identical copies (see `game/README.md`).
- `m7/`: byte-identical copies from the animals mockup (M7, `docs/mockups/animals/`):
  the heron, gull, shore crab, bog toad, mire lurker, firefly and glowmoth
  portraits, and the `drop_*` icons (feather, egg, shell, meat, glow, hide) the
  bestiary shows as what a creature gives. M7's design is the source.
- `m9/`: byte-identical copies from the fish lab mockup (M9,
  `docs/mockups/fish/species-<name>.png`, renamed `<name>.png`): bloatfin,
  lanternjaw, glassback, emberback, sunplate.
- Everything else here is mockup-only pixel art drawn in code by
  `tools/art/ocean_art.py` with the balatro-pixel-art kit (Sweaft-64 palette,
  hard alpha): 43 creature sprites at 24 x 24 (side views head right for
  swimmers and fliers, top views head right for crawlers), nine plant and reef
  cards (`card-*`) and ten 32 x 32 textures (`tex-*`). Run
  `python tools/art/ocean_art.py` to redraw them.
