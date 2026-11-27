# The UI kit (`stone-and-wood-ui`)

The game's interface art: the **Smooth** look the owner chose on 2026-11-14
("Smooth is the best. Write it down as the decision and start working the
current UI as the wood + rendered smooth rock"). Every file here is a copy,
byte for byte, of the mockup's art at commit `d69ee33` (the last commit that
touched it before this copy); the scripts that make it are in `tools/art/`.
Only the pieces the game draws are copied. `crates/pbd-app/src/ui_kit.rs`
loads them.

| Here | Copied from | Made by |
| --- | --- | --- |
| `stone/<piece>.png` | `docs/mockups/ui/rendered/hires/<piece>.png` | `tools/art/ui_panels_blend.py` (Blender Cycles, 8 texels a native pixel) |
| `oak/<piece>.png` | `docs/mockups/ui/rendered/hires/oak/<piece>.png` | the same, the carved-oak set |
| `rock/rock-268x152.png` (the pack), `rock/rock-150x148.png` (the loading slab), `rock/rock-464x200.png` (the wide pack, Craft at its side, `tool-crafting` decision 12, 2026-11-16; the 268 x 204 Craft-row rock it replaced is still in the mockup's folder), `rock/rock-372x168.png` (the chest rock, the workbench panel's) | `docs/mockups/ui/rendered/hires/rock/` | the same, the approved base rock rendered at each panel's size (4 texels a native pixel) |
| `slices.json` | `docs/mockups/ui/rendered/slices.json` | the same script: each piece's native size and 9-slice margins, the rock panels' sizes, the tokens and the contrast measured |

Stone pieces copied: `frame`, `lintel`, `slot-idle`, `slot-hover`,
`slot-selected`, `slot-disabled`, `button-idle`, `button-hover`,
`button-pressed`, `button-disabled`, `button-focus`, `slider-track`,
`check-off`, `check-on`, `tooltip`, `bar`, `fill-progress`, `compass`,
`compass-pointer` (oak, filed with the stone as the mockup files it),
`compass-town`, `map-frame`, `emblem`. Oak: `frame`, `button-idle`,
`button-hover`, `button-pressed`, `button-disabled`, `button-focus`,
`tab-open`, `tab-shut`, `slider-knob`, `tooltip`.

**Sampling.** These renders are sampled LINEAR: the owner's exception, for
the UI's kit only. The item icons in a slot, the kit's pixel font
(`assets/fonts/kit-font.png`) and every world texture stay nearest
(CLAUDE.md). The game resamples each piece once to the screen's pixels a
native pixel (the whole-number UI scale), so it is drawn about one texel a
pixel.

To change a piece, change `tools/art/ui_panels_blend.py`, re-render into
`docs/mockups/ui/rendered/`, and copy it here again with this table's
commit updated.

`heart.png` (11 x 10, `taming-hearts`) is drawn by hand as a pixel grid, not
copied from the mockup: the hearts that rise over an animal fed from the hand,
an emote with the 1 px outline of an inventory icon.

`survival/` (`survival-mode` task 4.6) holds the survival HUD's 16 x 16
icons - `heart`, `stamina`, `roast-meat`, `water-drop`, `breath`, `sun`,
`snowflake`, `campfire` - copied unchanged from the approved HUD mockup's
set (`docs/mockups/mission-tree/items/`, drawn by
`tools/mockup_survival_icons.py`), so the game's left stack shows exactly
the icons the owner approved (survey G7).
