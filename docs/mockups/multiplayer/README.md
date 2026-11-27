# Files of `docs/mockups/multiplayer.html`

The page for `openspec/changes/multiplayer` tasks 5.1 (host and join screens)
and 5.2 (another player in the world).

## The game's own, copied byte for byte

| Here | Source |
| --- | --- |
| `ui/oak-{frame,button-*,tab-open,tab-shut,tooltip,slider-knob}.png` | `docs/mockups/ui/rendered/hires/oak/<piece>.png` (= `assets/ui/oak/`) |
| `ui/stone-{lintel,tooltip,slot-idle,check-on,check-off,bar,fill-progress,frame}.png` | `docs/mockups/ui/rendered/hires/<piece>.png` (= `assets/ui/stone/`) |
| `game/atlas.png` | `assets/tilesets/atlas.png` |
| `game/timber.png`, `game/iron.png` | `assets/textures/settlement/<name>.png` |
| `game/day-meadow.jpg` | `docs/mockups/ui/game/day-meadow.jpg` (a game capture) |

## Copied and changed

| Here | Source | Change |
| --- | --- | --- |
| `font/kit-font.js` | `assets/fonts/kit-font.json` | the same JSON as `window.KIT_FONT`, so the page needs no fetch |
| `game/logo.png` | `assets/ui/logo.png` | resized from 1024 x 682 to 512 x 341 |
| `game/night-dunmouth-play.jpg` | `docs/mockups/ui/game/night-dunmouth.jpg` | the capture's debug line cut off the top |
| `game/night-dunmouth-world.jpg` | the same | the debug line and the hotbar cut off |
| `game/title-backdrop.jpg` | `docs/mockups/ui/game/title-backdrop.jpg` (a still of the UI mockup's demo) | cropped to 1392 x 870 to drop the demo's corner button |
| `scene-kit.js` | `docs/mockups/building/scene-kit.js` | none; a copy, so its textures load from this folder |

## Mockup-only

| Here | Source |
| --- | --- |
| `figure-kit.js` | `docs/mockups/npcs.html`, the `window.FK` block (M5's townsperson kit) and its Fields folk palette, copied verbatim (version 2: the other player is the game's NPC figure) |
| `player.js` | the PLAYER tab: the patch, the lamps and the other player's body, whose textures it paints from the `fig/` tiles at load |
