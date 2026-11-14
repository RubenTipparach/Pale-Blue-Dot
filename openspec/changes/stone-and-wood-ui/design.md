# Design: the UI in rock, dressed stone and carved oak

The owner, 2026-11-14: **"pull in UI skills from my fleet-command repo, and
develop ui for this game, do something like with rock and carved wood"**.

Mockup: `docs/mockups/ui.html` ("Stone and Timber"), tabs Kit, HUD, Pack,
Pause, Map, Station, Talk, Loading, Phone. Its shots are in
`docs/screenshots/mockup-ui/`. Status: **proposed**. Nothing here is built in
the game; the kit's art, the two scripts and the mockup are.

## 1. Where the game's UI stands (the "before")

Read off `crates/pbd-app/src/desktop/*.rs`, `loading.rs` and the screenshots
named in the proposal.

| Screen | File | What it is today |
| --- | --- | --- |
| Crosshair, instrument line | `hud.rs` | 3 px pale square; one 11 px text line top left (near-field and place readout, an instrument until `near-field-streaming` lands) |
| Hotbar, tool slot | `slots.rs`, `equipment.rs` | 10 squares of 44 px, gap 4, 1 px border, bottom 22 px; the tool slot left of them; counts in the corner; icons are atlas tiles and 16 px item PNGs, nearest |
| Pack | `pack.rs` | 3 x 10 slots + the hotbar row in a `PANEL_FILL` box, a hint line, CLOSE (I) |
| Pause, settings, saves | `menu.rs` | `Screen` enum; panels with `PANEL_FILL` and `EDGE`; rows painted idle/hovered/pressed by three flat fills; the bindings list from `BINDINGS` |
| Time and weather bars | `time_ui.rs`, `weather_ui.rs` | Bars on the pause panel |
| Overlay legend | `overlay_ui.rs` | Legend and a row on the pause panel |
| Map | `map_screen.rs`, `map_sites.rs` | Full-screen map; a text panel of overlay toggles open top left; coordinates bottom right |
| Compass bar | `compass_bar.rs` | 32 px translucent strip, letters, ticks every 15 degrees, town diamonds, the nearest town named |
| Field guide | `guide.rs` | A menu screen of entries |
| Frame graph | `frame_graph.rs` | An instrument, F3, off by default |
| Loading | `loading.rs` | A bordered box on black: name, step, a bar of cells |

What holds across all of them: no image behind any panel; one palette in
`menu.rs` and copies of it in `pack.rs`, `loading.rs` and `compass_bar.rs`;
sizes as raw px literals; Bevy's default font. The look is a debug overlay.
The quiet screen (owner, "All text is excess on the UI",
`menus-and-a-quiet-screen`) is right and stays.

The mockups' panels that will become game screens were read too: the forge
(crafting M3), the hearth (cooking M27), the conversation and job board
(townsfolk M5), the barter (market M12), and the breeding pens (husbandry M26),
the shipyard (M4), the house builder (M6) and the game tables (M11). Each is a
frame with a title, a list or grid of slots, a few plaques and sometimes a
bar; the kit covers all of them with the same pieces.

## 2. How the ported skills were used

- **game-ui-frontend** (OpenAI game-studio, via Fleet Command): visual
  direction first (fantasy: a hand-built world of hex blocks; material
  language: oak, stone, rock, iron; type; palette; motion: almost none), one
  persistent HUD cluster plus transient prompts (already the quiet screen),
  menus that feel native to the world rather than a dashboard, input gated
  while a panel is open (already `MenuOpen`), desktop and phone from the start.
- **ui-ux-pro-max** (NextLevelBuilder, via Fleet Command): the Pixel Art style
  entry (pixelated type, no anti-aliasing, limited palette), the Pixel Retro
  pairing (Press Start 2P + VT323; Press Start 2P is too wide, so the display
  face is Pixelify Sans, confirmed on Google Fonts with `--domain
  google-fonts`), and the accessibility rules: text contrast 4.5:1, 44 x 44 px
  targets, visible focus, reduced motion, no horizontal scroll at 390 px. Its
  `--design-system` generator was tried ("fantasy rpg crafting game") and
  returned a SaaS landing page; it was not used.
- **panel-textures**: the 9-slice rule (corners finished like hero pieces;
  edges and centre plain enough to stretch; margins written beside each piece;
  checked at three sizes; a contact sheet looked at before it is done).
- **blender-ui-panels** and **blender-organic-panels**: model, light from the
  upper left, render face-on on a transparent film, judge composited over the
  game; one height field with material masks, no primitives stuck on. Used for
  the rendered variant (section 7).

## 3. Materials

| Material | Where | Ramp (Sweaft-64, dark to light) |
| --- | --- | --- |
| Carved oak | Frames, plaques, tabs, compass strip, map frame, tooltips, scroll thumb | `#2f1525 #41322c #725244 #ab8465 #cfaa81` |
| Dressed stone | Panel faces and lips, slots, grooves, checkboxes | `#161612 #2c2c25 #434137 #a09f91 #bfcad7` |
| Rough rock | Loading slab, map corner blocks, the emblem | `#161612 #2b1f35 #3e3551 #69608b #92a3b2` |
| Iron | Brackets, rivets, bar bands, the compass pointer | `#010000 #161612 #2c2c25 #434137 #a09f91` |
| Brass | Hovered rivets, town studs, titles | `#733126 #c56d25 #fb902f #faaf5c` |
| Rune glow | Focus and selection only (the mint the game uses today) | `#204f63 #53998b #84e3b5 #c1f1d8` |

Rules from the art direction kept: lit from the top left, a 1 px `#010000`
outline round every piece, a dark seam (`#170916`) between two materials,
alpha 0 or 255, every colour from the palette (the kit's `save()` refuses
anything else). Rough rock is cold and dressed stone warm on purpose, so a
glance tells a menu (oak on warm stone) from a thing in the world (rock).

## 4. Tokens (what replaces `INK`, `MINT`, `PANEL_FILL`, `EDGE`)

| Token | Colour | Use | Contrast measured (`ui_art.py`) |
| --- | --- | --- | --- |
| `TEXT` | `#eab798` | Body text on a stone face | 7.85:1 on `FACE`, 10.1:1 on `WELL` |
| `TEXT_DIM` | `#cfaa81` | Secondary text | 6.51:1 on `FACE` |
| `TITLE` | `#faaf5c` | Carved titles, key names | 7.6:1 on `FACE` |
| `LABEL` | `#010000` | A label cut into oak (plaques, compass letters) | 6.21:1 on idle oak, 9.71:1 hovered |
| `FACE` | `#2c2c25` | A panel's stone face | |
| `WELL` | `#161612` | The floor of a slot, groove or window | |
| `SEAM` | `#170916` | Carved lines, seams | 5.72:1 on the compass oak |
| `INK` | `#010000` | Outlines | |
| `RUNE` | `#84e3b5` | Focus ring, selected slot | 11.8:1 on `WELL` |
| `HEALTH`, `STAMINA`, `BREATH`, `PROGRESS` | ramps above | Bar fills | |

Every pair a player reads clears WCAG AA (4.5:1); the disabled label
(`TEXT_DIM` on dim oak) is 5.66:1. In Bevy the tokens become one module,
`desktop/ui_kit.rs`, and every file imports them from there, so the copies in
`pack.rs`, `loading.rs` and `compass_bar.rs` go (the palette's one-source rule).

## 5. Components

Every piece is in `docs/mockups/ui/` (the game would load the same PNGs from
`assets/ui/`), drawn by `tools/art/ui_art.py`; `slices.json` gives each
piece's size, slice margins and how its sides scale.

| Piece | Native px | Slices | States / variants |
| --- | --- | --- | --- |
| `frame` (menu panel) | 48 x 48 | 14, sides stretch | one |
| `rock` (slab) | 48 x 48 | 14, sides **tile** (20 px period) | one |
| `slot-*` | 24 x 24 (16 x 16 well at 4,4) | none | idle, hover, selected, disabled; uncommon, rare, epic, legendary |
| `button-*` (plaque) | 48 x 22 | 8 | idle, hover, pressed (sinks 1 px), disabled, focus |
| `tab-open`, `tab-shut` | 40 x 18 | 7 | open (taller, no foot), shut |
| `scroll-track`, `scroll-thumb` | 8 x 24, 8 x 16 | 4 | |
| `slider-track`, `slider-knob` | 32 x 8, 9 x 13 | 3, none | |
| `check-off`, `check-on` | 12 x 12 | none | the tick is a glowing rune |
| `tooltip` | 24 x 18 | 6 | |
| `bar` + `fill-*` | 40 x 10 (channel 5..34 x 3..6), fills 2 x 4 | 4 | health, stamina, breath, progress |
| `compass` + `compass-pointer` + `compass-town` | 80 x 18, 7 x 5, 5 x 5 | 10 / 9 | letters and ticks carved dark, N red-inlaid |
| `map-frame` | 64 x 64 | 22 | rock corner blocks with the planet's ring and dot |
| `portrait` | 32 x 32 | 8 | |
| `emblem` | 48 x 48 | none | the pale blue dot set in a rock roundel |
| `icon-*` | 11 x 11 | none | close, back, gear, map, pack, cut into oak |

Rarity frames are offered for the mockups' loot; the game has no rarity yet.

### Typography

- **Pixelify Sans** (Google Fonts, OFL): carved titles and plaque labels, upper
  case, letter-spaced half a pixel, a 1 px dark shadow below for the cut.
- **VT323** (Google Fonts, OFL): counts, lines of talk, settings rows, the
  bindings list.
- Sizes are whole multiples of the UI scale (labels 8 px x scale, body 10 px x
  scale). If Bevy's text rasteriser blurs either face at those sizes, the
  fallback is a bitmap font drawn with the kit (recommendation taken: try the
  TTFs first; ask only with screenshots).

## 6. Scale, sampling, night and phone

- **Integer UI scale.** `UiScale` = max(1, floor(window height / 450)): 2 at
  900 and 1080, 3 at 1440, 4 at 2160. The player may pick a lower whole number
  in settings (`player-settings`); never a fraction. Pieces are authored at 1x
  and every node size is a whole number of native pixels times the scale.
- **Nearest sampling** for every UI image (`ImageSampler::nearest()` through
  `ImageLoaderSettings`, as `planet.rs` loads the atlas). The map's own render
  stays linear (`map_screen.rs`), it is not pixel art.
- **Night.** The UI is unlit: the same pieces by day and by night. Each piece
  carries its own outline and an opaque face, so it reads over noon sand and
  over a dark town alike (HUD tab, both shots). Nothing glows but the rune.
  Recommendation taken (ask only with screenshots): no night dimming.
- **Phone width** (a 390 px window, or the owner's phone viewing a mockup):
  the scale stays 2 (slots 48 px, plaques 44 px tall, both touch targets); the
  hotbar wraps to two rows of five in the lower left; the compass narrows to
  150 native px; a menu fills the width and its grid drops to five columns.
- **Motion**: none of its own. A pressed plaque sinks a pixel; hover lifts a
  tone. No idle animation (CLAUDE.md, "Nothing in a mockup spins on its own").

## 7. Drawn against rendered (the panel-textures route)

`tools/art/ui_panels_blend.py` models the frame and the rock slab as height
fields in numpy (moulding profile by distance to the edge, so the mitres come
free; fbm rock; grain and strata varying only across each edge so a stretched
side is unchanged), renders them in Cycles face-on with a sun from the upper
left (bpy 4.5.16 wheel, Python 3.11, **CPU only: this cloud container has no
GPU**, 24 samples, about 3 s a panel), then area-averages in linear light to
48 x 48 and snaps to the kit's own colours.

They differ: the rendered frame's grain is finer and its face snapped to a
brown (`#41322c`) where the drawn one is grey stone; the rendered rock is
noisy at 2x where the drawn one reads as blocks. **Recommendation: drawn**,
with the renders kept as a shading reference. This is owner question Q1.

## 8. Where each piece lands in the game (after the go-ahead)

API names as of Bevy 0.18.1 (`ImageNode`, `NodeImageMode::Sliced(TextureSlicer
{ border, center_scale_mode, sides_scale_mode, max_corner_scale })`,
`SliceScaleMode::{Stretch, Tile}`, `UiScale`); confirm them against the
crate at the first task, and again if `bevy-0-20` lands first.

| Piece | File | How |
| --- | --- | --- |
| Tokens, image handles, `nine(piece)` helper | new `desktop/ui_kit.rs` | One `UiKit` resource: handles for every PNG loaded with a nearest sampler, the slice margins read from `assets/ui/slices.json` (the art's own file, so Rust never restates a margin), the colour tokens as `Color` constants. `nine(&kit, Piece::Frame)` returns an `ImageNode` with `NodeImageMode::Sliced` |
| UI scale | `desktop/mod` setup, `settings.rs` | Set `UiScale` from the window height (section 6) on resize; all `px()` literals in `desktop/` become native-pixel constants (slot 24, not 44) |
| Palette | `menu.rs`, `pack.rs`, `loading.rs`, `compass_bar.rs`, `map_screen.rs`, `guide.rs`, `overlay_ui.rs`, `time_ui.rs`, `weather_ui.rs` | `INK`/`MINT`/`PANEL_FILL`/`EDGE`/`DIM` and the tuples become `ui_kit` tokens; the copies are deleted |
| Menu panels | `menu.rs` (`panel_node`), `pack.rs`, `guide.rs` | `BackgroundColor` + `BorderColor` become `nine(Frame)`; padding = 14 native px |
| Rows / buttons | `menu.rs` (`idle`/`hovered`/`pressed`, `TouchedRows`) | The same `Changed<Interaction>` query swaps the plaque image (idle, hover, pressed) instead of a fill; disabled and focus are two more images; label colour `LABEL` |
| Slots | `slots.rs` (`cell`, `border_of`), `equipment.rs` | The slot square becomes an `ImageNode` of `slot-idle`/`-selected`/`-hover`; the icon node sits at 4,4 native inside it at 16 x 16 (the item PNGs' own size); counts in VT323 |
| Tabs, slider, checkboxes, scroll | `menu.rs` (settings), `time_ui.rs`, `weather_ui.rs`, `guide.rs` | Settings gets tabs (Input, Picture, Weather); the time and weather bars become `slider-track` + `slider-knob`; `rain_shadows` and `auto_step` become checkboxes; long lists get the scroll track |
| Compass | `compass_bar.rs` | The strip is `nine(Compass)`; letters and ticks in `SEAM`/`LABEL`, N in red inlay; town diamonds are `compass-town`; the pointer image over the middle |
| Map | `map_screen.rs` | The map frame at the window edge (`nine(MapFrame)`, no centre fill); the overlay panel becomes a drawer of plaques, shut by default, opened by a plaque |
| Tooltips | `pack.rs`, `slots.rs` | `nine(Tooltip)` on hover, item name and count |
| Loading | `loading.rs` | `nine(Rock)` slab, emblem, carved name, the step in VT323, the bar as `bar` + `fill-progress`; the UI images and both fonts join the prewarm so the first frame already shows them |
| Fonts | `assets/fonts/` + every `TextFont` | Two TTFs with their OFL texts; `UiKit` holds the handles |
| Station, talk, trade | the crafting, cooking, townsfolk and market changes | Built from the same kit when those screens are built; nothing new here |

## 9. Decisions

1. **Oak frames round stone faces for menus; rough rock for things in the
   world.** Recommendation, asked as Q2 with screenshots.
2. **Drawn pixel art, not rendered-then-pixelated.** Recommendation, asked as
   Q1 with screenshots.
3. **One accent, the rune.** Selection and focus only; the mint stays the
   game's one cold colour against the warm kit. Recommendation taken (ask only
   with screenshots).
4. **Pixelify Sans + VT323.** Recommendation taken (ask only with screenshots).
5. **Integer UI scale from the window height, nearest sampling.**
   Recommendation taken; the art direction's rule, not a choice.
6. **No night dimming of the UI.** Recommendation taken (ask only with
   screenshots); both HUD shots are in the mockup.
7. **The quiet screen holds.** In play: crosshair, hotbar, compass. The
   instrument line stays until `near-field-streaming` takes it.

## 10. Owner questions (each with screenshots)

| Id | Question | Screenshots | Recommendation |
| --- | --- | --- | --- |
| Q1 | Drawn or rendered-then-pixelated panels? | `docs/screenshots/mockup-ui/q1-drawn-vs-rendered.png` (and the Kit tab) | Drawn |
| Q2 | Menus framed in oak round stone, or in rough rock? | `docs/screenshots/mockup-ui/q2-oak-vs-rock.png` | Oak round stone |

They belong in the survey (`docs/design/survey.html`, the `html-survey`
skill) beside those screenshots when the page is published.

## 11. Order of work (after the go-ahead)

1. The kit and the scale: `ui_kit.rs`, assets, nearest sampler, `UiScale`, a
   capture proving a 48 px frame at scale 2 draws its corners at 28 px exactly.
2. Pause, settings and saves (`menu.rs`): the smallest screens, every plaque
   state.
3. Slots, tool slot and pack.
4. Compass bar.
5. Map frame and drawer.
6. Loading screen (with the prewarm).
7. Guide, overlay legend, time and weather bars.
8. Screenshots of each step in `docs/screenshots/stone-and-wood-ui/`, day and
   night; frame time on the owner's desktop.

## 12. Risks and what is not known yet

- **How Bevy draws slice corners under `UiScale`.** If corners do not scale
  with `UiScale`, the kit is loaded pre-scaled (nearest upscale at load by the
  integer scale). Measured in step 1, not assumed.
- **Text crispness** of the two TTFs in Bevy's rasteriser at whole sizes.
- **Frame cost**: a few dozen image quads; not measurable here (no GPU).
- **The mockup scales by halves.** At a page width under 1440 px the scenes
  are shown at half size with the UI at 1x, the game's proportion at a 1440 x
  900 window; under 720 px the scene is a quarter and the UI is not to scale
  (the page says so).
