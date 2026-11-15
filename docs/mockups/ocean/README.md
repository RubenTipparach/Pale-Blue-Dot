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

- `creatures3d.js`: every animal alive in a scene is a low-poly 3D model built in code here (the owner,
  2026-11-15: "ducks should be 3d, birds in general are 3d", "all fish should also be 3d, unless they are
  caught, and a player drops them as 2d sprites"). Flat shaded, one colour per face, the colours read off each
  creature's sprite above; the fish body follows `fish.html`'s `buildFish` (M9). The sprites stay as the
  bestiary's portraits, and the game's fish icons in `game/` as a caught fish dropped on the ground.
- `anthias.png`, `damsel.png`: the reef anthias and sooty damsel portraits (2026-11-15), 24 x 24 in the same
  framing and Sweaft-64 tones, drawn by `draw_reef_fish.py` (`python draw_reef_fish.py docs/mockups/ocean`).
