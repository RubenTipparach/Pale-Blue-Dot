# Proposal: the UI in rock, dressed stone and carved oak

## Why

**The owner, 2026-11-14: "pull in UI skills from my fleet-command repo, and
develop ui for this game, do something like with rock and carved wood".**

The game's UI today is one look, inherited from the engine's first HUD and
never designed: near-black teal panels (`PANEL_FILL`, 0.02/0.06/0.08 at 94%),
a thin mint-grey border (`EDGE`), mint headings (`MINT`), pale ink (`INK`),
Bevy's default font at 11 px, and three flat fills for a row's idle, hovered
and pressed states. It reads as a debug overlay sitting on a pixel-art world
(`docs/screenshots/menu-pause.png`, `menu-settings.png`,
`inventory-grid/pack-after-dig.jpg`, `world-map/game-open.jpg`,
`loading-screen/building-the-planet.png`). Measured in the source:

| | Today |
| --- | --- |
| Palette | 4 constants in `menu.rs`, copied by hand into `pack.rs` (`EDGE`, `PANEL_FILL`, `DIM`) and `loading.rs` (`INK`, `MINT`, `DIM`, `EDGE`, `NIGHT`); `compass_bar.rs` keeps its own `INK`, `NORTH`, `PLACE` tuples |
| Panels | `BackgroundColor` + `BorderColor`, no image; 46 `BorderColor`/radius uses across `desktop/` |
| Images in UI | slot icons only (atlas tiles, item PNGs), sampled nearest |
| Font | Bevy's default, sizes 11-26 px, no pixel face |
| UI scale | none: every size is a raw px literal (slot 44, gap 4, bar 32) |

The world is pixel art sampled nearest at whole texels; its UI should be too.

## What changes

A UI kit in three materials and one accent, drawn as pixel art in the game's
style and used by every screen:

- **Carved oak** for what the hand touches: menu frames (a moulding with
  mitred corners and iron brackets), button plaques, tabs, the compass strip,
  the map's frame, tooltips.
- **Dressed stone** for what holds things: the face every menu's text sits on,
  slots as stone blocks with a well cut in them, grooves for scrollbars and
  sliders, checkboxes.
- **Rough rock** for what stands in the world: the loading screen's slab, the
  map frame's corner blocks.
- **Iron and brass** fittings in corners only; **one rune glow** (the mint the
  game already uses) for focus and selection.

Every piece is drawn at native size by `tools/art/ui_art.py` with the
`balatro-pixel-art` kit (every colour Sweaft-64), shown at a whole-number UI
scale with nearest sampling, and built as a 9-slice where it stretches. Two
Google Fonts (OFL): Pixelify Sans for carved titles and labels, VT323 for
counts and lines of text.

The quiet screen does not get louder: in play the crosshair, the hotbar and
the compass stay the only things drawn (`menus-and-a-quiet-screen`).

Also in this change, done: five UI skills ported from Fleet Command into
`.claude/skills/` (`game-ui-frontend`, `ui-ux-pro-max`, `blender-ui-panels`,
`blender-organic-panels`, `panel-textures`), with provenance in
`.claude/skills/PROVENANCE.md`; the kit (`tools/art/ui_art.py`,
`tools/art/ui_panels_blend.py`); and the mockup `docs/mockups/ui.html`
("Stone and Timber").

## What does not change

- No game code in this change's first step. The Bevy work (design.md,
  "Where each piece lands") waits for the owner's go-ahead
  (CLAUDE.md, "Write it up before touching code").
- Layout and behaviour of every screen: the same controls, the same keys,
  `MenuOpen` and `Display::None` as they are.
- Saves: the UI holds no saved state.

## Impact

- `crates/pbd-app/src/desktop/` (every UI file), `crates/pbd-app/src/loading.rs`,
  a new `desktop/ui_kit.rs`, `assets/ui/` (the kit's PNGs and `slices.json`),
  `assets/fonts/` (two TTFs and their OFL texts).
- The loading screen's prewarm must include the UI images and fonts so the
  first frame is the new screen (CLAUDE.md, "A loading screen on boot").
- Frame time: a handful of image draws in place of flat quads; to be measured
  on the owner's desktop with `perf_suite.py` (not measurable in a cloud
  container, which renders on the CPU).
