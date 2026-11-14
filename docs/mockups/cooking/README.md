# Art for the cooking mockup (`cooking.html`, "Hearth and Kitchen")

- `game/`: the game's own art, byte-identical copies (see `game/README.md`).
- `m3/`: byte-identical copies from the crafting mockup (M3, `docs/mockups/crafting/`):
  `log`, `coal`, `clay`, `brick`, `stone`, `planks`, `iron-ingot`, `glass`,
  `copper-ingot`, the materials a kitchen station is built from and burns.
- `m7/`: byte-identical copies from the animals mockup (M7, `docs/mockups/animals/`):
  the `drop_meat`, `drop_milk` and `drop_egg` icons and the portraits of the
  animals the ranch and sea tables name (hen, plodder, woolback, ridge goat,
  sand strider, tusk boar, meadow and snow hare, shagback, mire lurker, shore
  crab, gull). M7's design is the source.
- `m8/`: byte-identical copies from the farming mockup (M8, `docs/mockups/farming/`):
  `produce_<crop>.png` and the ripe stage `<crop>_3.png` of its seven crops
  (wheat, rice, chili, turnip, sun melon, frost kale, glowcap), and
  `compost.png`, a copy of M8's `tool_fert.png` (compost, what spoiled food
  becomes). M8's design is the source.
- `m25/`: byte-identical copies from the seas, rivers and marshes mockup (M25,
  `docs/mockups/ocean/`): squid, shrimp, octopus, clam, lobster, crab,
  crayfish, abalone, urchin, vent mussel, sea cucumber, redrun, trout, krill
  and reed dabbler (hen). M25's design is the source.
- Everything else here is mockup-only pixel art drawn in code by
  `tools/art/cooking_art.py` with the balatro-pixel-art kit (Sweaft-64 palette,
  1 px outline, hard alpha):
  - item icons at 24 x 24: ingredients (`flour`, `dough`, `butter`, `cheese`,
    `oil`, `soy_sauce`, ...), meat cuts by state
    (`meat_<poultry|pork|beef|mutton|game>_<raw|cooked|smoked|cured>`), fish
    fillets (`fillet_<raw|cooked|smoked|salted>`), every dish and drink, and the
    three proposed catchables (`sand_prawn`, `sand_clam`, `rock_mussel`), sea
    water and a cut kelp frond (`kelp_frond`; M25 has the plant, not the item);
  - kitchen station icons at 32 x 32 (`st_*`);
  - the twelve new crops: `crop_<id>_0..3` at 32 x 32 in M8's layout (stage 0
    the seeded mound) and `produce_<id>` at 24 x 24;
  - textures (`tex_embers`, `tex_crust`, `tex_dough`, `tex_steel`, `tex_panel`)
    and cut-out cards (`card_flame`, `card_smoke`, `card_herbs`, `card_fish`,
    `card_noodles`) for the 3D kitchens, 32 x 32.

Run `python tools/art/cooking_art.py` to redraw them.
