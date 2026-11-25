# Build mode's palette icons (`player-building` design B5)

`wall`, `door`, `window`, `floor`, `roof`, `stairs`, `post` and `lantern`
are M6's own icons, copied unchanged from `docs/mockups/building/icon_*.png`
(the Hex House Builder, `docs/mockups/building.html`). `remove` and
`blueprint` are drawn for the game by a short PIL script in the
`player-building` commit (an X; a house outline on a sheet), 32 x 32, in
the same size as M6's. They are UI icons and keep their 1 px edge.

`erase` is M6 version 3's rubber (`building-planner`), copied unchanged from
`docs/mockups/building/icon_eraser.png` by `tools/art/planner_icons.py`: the
planner's Erase tool. The planner's other tools use the icons above.
