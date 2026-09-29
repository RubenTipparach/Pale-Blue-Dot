# city-sites

Screenshots for `openspec/changes/city-sites`, tasks 1.2 and 1.3. Taken
headless by `tools/mockup_sites_test.js` (Chromium, 1440 x 900, no GPU) from
`docs/mockups/world-map.html`, with the sites `examples/sites.rs` wrote from
`pbd_core::sites::generate` on generator version 6 with half the desert.

| file | what it shows |
| --- | --- |
| `mockup-world.png` | The spawn's continent and its neighbours at the map's opening zoom: 55 sites on the planet, squares for towns and circles for villages. Holbrook, the village near the spawn, and Linenleigh, a walled town, stand by the player's arrow. |
| `mockup-close.png` | Ashingstead, the capital, at close zoom (50 m scale): its 75 m footprint outlined, on fields at the edge of a desert. The places list shows the capital first, then the home town. |
| `mockup-edit.png` | The site editor after the test dragged Ashingstead east: its outline dashed in amber as edited, and the RON the editor writes, pinning it at its new place and striking its old id. |

## The game's map (task 4.1)

In-game captures, 2026-09-29, in a cloud container on lavapipe at 1440 x 900
with no GPU, from the fast build (`--profile fast`) under `xvfb-run`, on a new
memory-only world at `--time 12` unless named. The world's list is made on its
first frame and read from its save as every world's is (task 3.1). They show
what the map draws, not how smoothly it runs; frame cost was not measured.

| file | flags | what it shows |
| --- | --- | --- |
| `game-map-spawn.png` | `--walk --menu map --map-mpp 12` | The spawn's continent at 12 m a pixel: the 55 sites the owner approved, squares for towns and circles for villages, named. The legend says "55 settlements on the map." The player's arrow is by Holbrook and Linenleigh. |
| `game-map-world.png` | `--map-mpp 40` | Zoomed all the way out: every site, named only the capital (Ashingstead) and the home town (Holbrook). |
| `game-map-footprints.png` | `--map-mpp 1.5 --map-at 29.70 1.36` | Close on Holbrook: each footprint drawn as its ellipse, Holbrook's 35 m, Linenleigh's 75 m, Durnacrag's 50 m. |
| `game-map-night.png` | `--map-mpp 12 --map-night --time 21` | 21:00 with the map's night on: every town in the dark lights a pool round it, the walled and desert towns wider, and its marker and name turn lamp-coloured. |

