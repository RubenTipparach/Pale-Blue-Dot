# city-sites

Screenshots for `openspec/changes/city-sites`, tasks 1.2 and 1.3. Taken
headless by `tools/mockup_sites_test.js` (Chromium, 1440 x 900, no GPU) from
`docs/mockups/world-map.html`, with the sites `examples/sites.rs` wrote from
`pbd_core::sites::generate` on generator version 6 with half the desert.

| file | what it shows |
| --- | --- |
| `mockup-world.png` | The spawn's continent and its neighbours at the map's opening zoom: 55 sites on the planet, squares for towns and circles for villages. Holford, the village near the spawn, and Linenleigh, a walled town, stand by the player's arrow. |
| `mockup-close.png` | Ashingstead, the capital, at close zoom (50 m scale): its 75 m footprint outlined, on fields at the edge of a desert. The places list shows the capital first, then the home town. |
| `mockup-edit.png` | The site editor after the test dragged Ashingstead east: its outline dashed in amber as edited, and the RON the editor writes, pinning it at its new place and striking its old id. |

Nothing here is the game's own map screen yet: that is task 4.1, after the
save's record store (`world-persistence` group 3).
