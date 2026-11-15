# The game's own pictures used by `docs/mockups/ui.html`

Byte-identical copies:

| File | Source |
| --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` (block icons are its tiles) |
| `hand.png`, `pickaxe.png`, `axe.png`, `shovel.png`, `rod.png` | `assets/items/tools/` |
| `torch.png`, `lantern_post.png`, `lantern_wall.png`, `candle.png`, `brazier.png` | `assets/items/lights/` (provenance in its `PROVENANCE.md`) |
| `perch.png`, `minnow.png`, `eel.png`, `silverfin.png` | `assets/items/fish/` (provenance in its `PROVENANCE.md`) |
| `night-dunmouth.jpg` | `docs/screenshots/cities-in-the-world/game-4a-dunmouth-above-night.jpg` |
| `pack-before.jpg` | `docs/screenshots/inventory-grid/pack-after-dig.jpg` |
| `map-before.jpg` | `docs/screenshots/world-map/game-open.jpg` |

Game captures re-encoded from PNG to JPEG (quality 88, same 1440 x 900 pixels)
to keep the page small:

| File | Source |
| --- | --- |
| `day-desert.jpg` | `docs/screenshots/compass-bar/compass-noon-north.png` |
| `day-meadow.jpg` | `docs/screenshots/hud-after.png` |
| `pause-before.jpg` | `docs/screenshots/menu-pause.png` |
| `settings-before.jpg` | `docs/screenshots/menu-settings.png` |
| `loading-before.jpg` | `docs/screenshots/loading-screen/building-the-planet.png` |

A still of the mockup's own Play demo (not a game capture):

| File | Source |
| --- | --- |
| `title-backdrop.jpg` | the Play tab (`ui/play.js`) at golden hour, the camera still, no UI, captured headless (SwiftShader) by the page's checks; the Main menu tab's backdrop |

Everything else in `docs/mockups/ui/` is mockup-only art: `tools/art/ui_art.py`
draws the kit's PNGs, `slices.json`, the kit font (`font/`) and four item icons
(`items/`); `tools/art/ui_panels_blend.py` renders the rendered and smooth kit
(`rendered/`, `rendered/hires/`) and the first two renders `blender-*.png`. The
Play demo also loads `building/scene-kit.js` and its textures in
`building/game/` (provenance in that folder's README).
