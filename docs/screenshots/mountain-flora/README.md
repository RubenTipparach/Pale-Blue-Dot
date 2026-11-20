# mountain-flora screenshots

M13's Mountains tab (`docs/mockups/foliage.html#mountains`, version 5;
`openspec/changes/mountain-ground` design 5 to 14), rendered with headless
Chromium on SwiftShader in a cloud container with no GPU (2026-11-19). They
show the look, never a frame time.

| Shot | Shows |
| --- | --- |
| `overview-today-new-day.png` | The whole mountainside, split: today (one slate, snow above 200 m, no trees) left, the plan right |
| `meadow-today-new-day.png` | The alpine meadow at the campfire, split, by day |
| `meadow-closeup-new-day.png` | The meadow close: alpine turf and tufts, avens, gentian, moss campion, cushions, juniper, firs |
| `treeline-new-day.png` | Firs thinning to krummholz at the tree line, lichen on the rock below |
| `snowline-new-day.png` | The snow line: snow patches, dusted rock, lichen, the proposed tarn |
| `night-meadow-new.png` | The meadow at night, lit by the campfire under a dim moon |
| `night-today-new.png` | The same at night, split |
| `phone-new-day.png` | The tab at phone width (390 px) |

## Version 6 (2026-11-20)

The owner on v5: "idk what this texture is supposed to be but its overlapping,
also you should use the foliage kit instead of making new plants". The streaks
were not two surfaces: the shader took a tile's sheet and column from its id
without rounding it, so where interpolation left 127.99997 for the scree (tile
128) the column came out as 3.99997 and the texel came from the next sheet,
pixel by pixel; the lichen hashed a per-pixel position the same way. Both are
rounded now, and snow patches, lichen and moss are whole texels of the face's
own tile. The plants are the Playground's kit. Same renderer (SwiftShader).

| Shot | Shows |
| --- | --- |
| `v6-scree-closeup-new-day.png` | The owner's view: the scree between the cliffs, close, from above, with no streaks |
| `v6-scree-texels-new-day.png` | Closer still: the scree tile, lichen and the kit's rocks, every texel whole |
| `v6-overview-today-new-day.png` | The whole mountainside, split, today left and new right |
| `v6-meadow-closeup-new-day.png` | The meadow close: the kit's tufts from the turf, flowers, the pine |
| `v6-treeline-krummholz-new-day.png` | Krummholz (the kit's pine, stunted and swept east) beside a full fir |
| `v6-snowline-new-day.png` | The snow line: snow patches on the turf's texel grid, lichen, the tarn |
| `v6-night-meadow-new.png` | The meadow at night by the campfire |
| `v6-night-today-new.png` | The same at night, split |
