# drop-lighting

| File | Command | Shows |
| --- | --- | --- |
| `before-pit.jpg` | `PBD_NO_CLOUDS=1 PBD_WINDOW=1600x1000 pbd-app --view meadow --time 11 --tool shovel --break 2.5 --pitch -55 --frames 300 --capture ...` on main before `drop-lighting` | Three dug sand blocks floating in their pit, lit by Bevy's unshadowed sun: near white, glowing. |
| `after-pit.jpg` | the same command, after | The same drops lit as the terrain is: no glow, the texture reads. Still paler than the pit's sand: their faces are the inventory icon's art, not the world's sand tile (task 1.3). |
| `before-after.jpg` | the two side by side | |
