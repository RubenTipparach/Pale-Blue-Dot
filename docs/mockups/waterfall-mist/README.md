# waterfall-mist (the Waterfall Mist mockup's files)

- `mist.js`: the page's scene. Built from `../waterfall/waterfall.js` (M31):
  its world, its port of the game's water fragment (`water.wgsl` at
  `dc0c5cb`) and its lights. Changed here: a pit 12 deep with 3.35 m of water,
  the pool's real depth under each cap, and the view from under the water.
  Added here: the splash ported from the game's `animate_splash`
  (`water_flow.rs`), drawn in the game's order today and in the fixed order;
  the new puffs, the bubbles and the aerated colour (mockup only).
- `mist_puffs.png`: byte-identical copy of
  `assets/textures/fx/mist_puffs/albedo.png` (see its `SOURCE.md`).
- `game-{down,side,under}-{today,fixed}.jpg`: the game's own captures, copied
  from `docs/screenshots/fall-splash-order/` (`*-before.jpg` is today,
  `*-after.jpg` the order fix).
