# Design: the UI in stone and rough rock, with carved oak as accent

The owner, 2026-11-14: **"pull in UI skills from my fleet-command repo, and
develop ui for this game, do something like with rock and carved wood"**;
then **"I like the stone kit more, can you make the stone more blue?"**; then
**"phones cant run this lol, mostly stone with some occasion timber for
accent"**.

Mockup: `docs/mockups/ui.html` ("Stone and Timber"), tabs Kit, HUD, Pack,
Pause, Map, Station, Talk, Loading. Its shots are in
`docs/screenshots/mockup-ui/`. Status: **proposed**. Nothing here is built in
the game; the kit's art, the two scripts and the mockup are. The owner's
answers of 2026-11-14 are folded in throughout and recorded at the end
("The owner's answers").

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
  while a panel is open (already `MenuOpen`). Its "desktop and phone from the
  start" does **not** apply: the game is desktop only (owner, "phones cant run
  this"), so there are no phone layouts.
- **ui-ux-pro-max** (NextLevelBuilder, via Fleet Command): the Pixel Art style
  entry (pixelated type, no anti-aliasing, limited palette), the Pixel Retro
  pairing (Press Start 2P + VT323; Press Start 2P is too wide, so the display
  face is Pixelify Sans, confirmed on Google Fonts with `--domain
  google-fonts`), and the accessibility rules: text contrast 4.5:1, visible
  focus, reduced motion. Its touch-target and 390 px rules are for phones and
  are not game rules here; the mockup page itself still never scrolls sideways
  at 390 px (an artifact rule for the page, not the game). Its
  `--design-system` generator was tried ("fantasy rpg crafting game") and
  returned a SaaS landing page; it was not used.
- **panel-textures**: the 9-slice rule (corners finished like hero pieces;
  edges and centre plain enough to stretch; margins written beside each piece;
  checked at three sizes; a contact sheet looked at before it is done).
- **blender-ui-panels** and **blender-organic-panels**: model, light from the
  upper left, render face-on on a transparent film, judge composited over the
  game; one height field with material masks, no primitives stuck on. Used for
  the rendered variant (section 7).

## 3. Materials: stone first, timber as accent

The default material of every piece is stone (owner: "mostly stone with some
occasion timber for accent"). Carved oak appears in exactly three small places
(section 3a); iron and brass stay as small fittings.

| Material | Where | Ramp (Sweaft-64, dark to light) |
| --- | --- | --- |
| Rough rock | The menu frame (the slab every menu, the pack, a station, talk and trade sit in), the loading slab, the map frame's corner blocks, the emblem | `#010e1a #1f3b52 #3e586e #5d778d #92a3b2` |
| Dressed stone | Panel faces; button plaques (every state); tabs; the compass strip; slots; scroll track and thumb; slider track and knob; checkboxes; tooltips; bar frames; the map frame's moulding; the portrait frame; the sub-panel frame (`frame`, for drawers); the icons | `#011c34 #1f3b52 #3e586e #92a3b2 #bfcad7`, plus `STONE_MID` `#5d778d` between the middle two (a hovered plaque, a raised thumb) |
| Carved oak (accent only) | The title lintel, the open tab's cap, the compass pointer (section 3a) | `#2f1525 #41322c #725244 #ab8465 #cfaa81` |
| Iron (fittings) | Studs on plaques, frame brackets, bar studs, bands on the scroll thumb and slider knob, the compass end bands, the lintel's pegs, the pointer's tip | `#010000 #161612 #2c2c25 #434137 #a09f91` |
| Brass (fittings) | Hovered studs, town studs on the compass, its end bosses | `#733126 #c56d25 #fb902f #faaf5c` |
| Rune glow | Focus and selection only (the mint the game uses today) | `#204f63 #53998b #84e3b5 #c1f1d8` |

Rules from the art direction kept: lit from the top left, a 1 px `#010000`
outline round every piece, a dark seam between two materials, alpha 0 or 255,
every colour from the palette (the kit's `save()` refuses anything else).
Rough rock (chipped, tiling) is a menu's outer frame; dressed stone (cut,
square) is everything the hand touches inside it.

### 3a. Timber as accent

Three uses, each small, each where wood earns it (a thing a carpenter would
fit to a stone wall). Nothing else is oak.

| Accent | Piece | Size (native) | Why wood here |
| --- | --- | --- | --- |
| 1. The title lintel | `lintel` (9-slice 5/9/5/9, stretches along its grain) | 40 x 16, 32 px tall at 2x | A carved board pegged over the menu's stone frame with the title cut into it: the one warm note on a menu, and it names the screen |
| 2. The open tab's cap | `tab-open` (rows 1..4 of 18) | a 3 px oak strip and its seam | Marks which tab is open without a second glow; shut tabs are plain stone |
| 3. The compass pointer | `compass-pointer` | 7 x 6, oak wedge with an iron tip | The one moving mark in the HUD, warm against the light stone strip |

The first draft's oak pieces (oak frame, plaques, tabs, compass strip, map
frame, tooltip, portrait, thumb, knob, icons, and the iron bar and pointer)
are kept as the `oak_*` and `iron_*` functions in `tools/art/ui_art.py` and
written by `ui_art.py --oak` into `docs/mockups/ui/oak/`; the Kit tab shows the
oak pause menu beside the stone one as the before.

## 4. Tokens (what replaces `INK`, `MINT`, `PANEL_FILL`, `EDGE`)

Every pair a player reads, measured by `ui_art.py` (WCAG 2 contrast; the run
fails if any is under 4.5:1; the numbers are also in `slices.json` and on the
Kit tab):

| Token | Colour | Use | Contrast measured |
| --- | --- | --- | --- |
| `TEXT` | `#eab798` | Body text on a stone face | 6.49:1 on `FACE`, 9.63:1 on `WELL` |
| `TEXT_DIM` | `#cfaa81` | Secondary text | 5.38:1 on `FACE` |
| `TITLE` | `#faaf5c` | Carved titles on a face, key names | 6.29:1 on `FACE` |
| `LABEL` | `#fffefe` | A plaque's label, inlaid chalk; the open tab's label | 7.38:1 idle and focus (`#3e586e`), 4.65:1 hover (`#5d778d`), 11.55:1 pressed (`#1f3b52`), 7.38:1 on the open tab |
| `LABEL_DIM` | `#92a3b2` | A disabled plaque's label | 6.66:1 on `#011c34` |
| `TAB_DIM` | `#bfcad7` | A shut tab's label | 7.00:1 on `#1f3b52` |
| `CUT` | `#011c34` | Letters and ticks cut into the compass strip | 6.66:1 on `#92a3b2` |
| `NORTH` | `#4a051e` | The N on the compass, red inlay | 6.09:1 on `#92a3b2` |
| `OAK_LABEL` | `#010000` | A title cut into the oak lintel | 6.21:1 on `#ab8465` |
| icon inlay | `#bfcad7` | The 11 px icons (close, back, gear, map, pack), inlaid on a face | 7.00:1 on `FACE` |
| `RUNE` | `#84e3b5` | Focus ring, selected slot | 11.20:1 on `WELL` |
| `FACE` | `#1f3b52` | A panel's stone face | |
| `WELL` | `#011c34` | The floor of a slot, groove or window | |
| `INK` | `#010000` | Outlines | |
| `HEALTH`, `STAMINA`, `BREATH`, `PROGRESS` | ramps in `ui_art.py` | Bar fills | |

The plaque labels changed from dark-cut-into-oak to light-inlaid-in-stone
because no dark ink clears 4.5:1 on mid stone (`#010000` on `#3e586e` is
2.82:1); the hovered plaque is the closest pair that passes, 4.65:1. In Bevy
the tokens become one module, `desktop/ui_kit.rs`, and every file imports them
from there, so the copies in `pack.rs`, `loading.rs` and `compass_bar.rs` go
(the palette's one-source rule).

## 5. Components

Every piece is in `docs/mockups/ui/` (the game would load the same PNGs from
`assets/ui/`), drawn by `tools/art/ui_art.py`; `slices.json` gives each
piece's size, slice margins and how its sides scale.

| Piece | Material | Native px | Slices | States / variants |
| --- | --- | --- | --- | --- |
| `rock` (the menu frame, the loading slab) | rough rock | 48 x 48 | 14, sides **tile** (20 px period) | one |
| `frame` (sub-panel: the map's drawer) | dressed-stone moulding; quoin joints and iron brackets in the corners | 48 x 48 | 14, sides stretch | one |
| `lintel` (title board) | **oak** (accent 1), iron pegs | 40 x 16 | 5 / 9 / 5 / 9 | one |
| `slot-*` | dressed stone, well 16 x 16 at 4,4 | 24 x 24 | none | idle, hover, selected (rune), disabled; uncommon, rare, epic, legendary |
| `button-*` (plaque) | raised stone, iron studs | 48 x 22 | 8 | idle; hover (lighter stone, brass studs); pressed (sunk 1 px, its light turned round); disabled (dead dark stone, no studs); focus (a rune cut round it) |
| `tab-open`, `tab-shut` | stone; the open one with the **oak cap** (accent 2) | 40 x 18 | 7 | open (taller, no foot), shut |
| `scroll-track`, `scroll-thumb` | stone groove; stone block with iron bands | 8 x 24, 8 x 16 | 4 | |
| `slider-track`, `slider-knob` | stone groove; stone grip with an iron cap | 32 x 8, 9 x 13 | 3, none | |
| `check-off`, `check-on` | stone well | 12 x 12 | none | the tick is a glowing rune |
| `tooltip` | stone rim round the face | 24 x 18 | 6 | |
| `bar` + `fill-*` | stone band, channel 4..35 x 2..7, iron studs | 40 x 10, fills 2 x 4 | 4 | health, stamina, breath, progress |
| `compass` + `compass-pointer` + `compass-town` | light stone strip; **oak pointer** (accent 3); brass stud | 80 x 18, 7 x 6, 5 x 5 | 9 / 10 / 9 / 10 | letters and ticks cut `CUT`, N inlaid `NORTH` |
| `map-frame` | stone moulding, rough-rock corner blocks with the planet's ring | 64 x 64 | 22 | |
| `portrait` | stone rim round a well | 32 x 32 | 8 | |
| `emblem` | rough-rock roundel, the pale blue dot | 48 x 48 | none | |
| `icon-*` | inlaid light stone | 11 x 11 | none | close, back, gear, map, pack |

Rarity frames are offered for the mockups' loot; the game has no rarity yet.

### Typography

- **Pixelify Sans** (Google Fonts, OFL): carved titles and plaque labels, upper
  case, letter-spaced half a pixel, a 1 px dark shadow below for the cut.
- **VT323** (Google Fonts, OFL): counts, lines of talk, settings rows, the
  bindings list, and the compass letters (Pixelify's capital E has a rounded
  back that reads as a 3 on the strip; VT323's does not).
- Sizes are whole multiples of the UI scale (labels 8 px x scale, body 10 px x
  scale). If Bevy's text rasteriser blurs either face at those sizes, the
  fallback is a bitmap font drawn with the kit (recommendation taken: try the
  TTFs first; ask only with screenshots).

## 6. Scale, sampling, night; desktop only

- **Desktop only.** The owner, 2026-11-14: "phones cant run this lol". The game
  does not run on phones, so it has no phone layout: no wrapped hotbar, no
  full-width menus, no touch-target sizing. The earlier draft's phone section
  (a hotbar of two rows of five, a narrowed compass, a five-column menu grid)
  and its requirement are withdrawn.
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

They differ: the rendered rock is noisy at 2x where the drawn one reads as
blocks; the rendered frame's grain is finer and its face snapped to a brown
(`#41322c`). Since the menus are stone (section 3), the comparison on the page
is the rock slab only, re-rendered in the slate-blue rock ramp
(`--only rock`, about 5 s on the CPU); the rendered oak frame
(`blender-frame*.png`) stays on disk and off the page. **Recommendation:
drawn**, with the renders kept as a shading reference. This is owner question
Q1, still open.

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
| Menu panels | `menu.rs` (`panel_node`), `pack.rs`, `guide.rs` | `BackgroundColor` + `BorderColor` become `nine(Rock)`; padding = 14 native px; the title moves onto a `nine(Lintel)` node set on the frame's top edge (accent 1), text `OAK_LABEL`; sub-panels (the map drawer) use `nine(Frame)` |
| Rows / buttons | `menu.rs` (`idle`/`hovered`/`pressed`, `TouchedRows`) | The same `Changed<Interaction>` query swaps the plaque image (idle, hover, pressed) instead of a fill; disabled and focus are two more images; label colour `LABEL` (disabled `LABEL_DIM`) |
| Slots | `slots.rs` (`cell`, `border_of`), `equipment.rs` | The slot square becomes an `ImageNode` of `slot-idle`/`-selected`/`-hover`; the icon node sits at 4,4 native inside it at 16 x 16 (the item PNGs' own size); counts in VT323 |
| Tabs, slider, checkboxes, scroll | `menu.rs` (settings), `time_ui.rs`, `weather_ui.rs`, `guide.rs` | Settings gets tabs (Input, Picture, Weather; open label `LABEL`, shut `TAB_DIM`); the time and weather bars become `slider-track` + `slider-knob`; `rain_shadows` and `auto_step` become checkboxes; long lists get the scroll track |
| Compass | `compass_bar.rs` | The strip is `nine(Compass)`; letters (VT323) and ticks in `CUT`, N in `NORTH`; town studs are `compass-town`; the oak pointer image over the middle |
| Map | `map_screen.rs` | The map frame at the window edge (`nine(MapFrame)`, no centre fill); the overlay panel becomes a drawer of plaques in `nine(Frame)`, shut by default, opened by a plaque |
| Tooltips | `pack.rs`, `slots.rs` | `nine(Tooltip)` on hover, item name and count |
| Loading | `loading.rs` | `nine(Rock)` slab, emblem, carved name, the step in VT323, the bar as `bar` + `fill-progress`; the UI images and both fonts join the prewarm so the first frame already shows them |
| Fonts | `assets/fonts/` + every `TextFont` | Two TTFs with their OFL texts; `UiKit` holds the handles |
| Station, talk, trade | the crafting, cooking, townsfolk and market changes | Built from the same kit when those screens are built; nothing new here |

## 9. Decisions

1. **Menus framed in rough rock, in slate blue.** The owner's answer to Q2
   (2026-11-14, "I like the stone kit more, can you make the stone more
   blue?").
2. **Mostly stone; carved oak only as three small accents** (section 3a). The
   owner, 2026-11-14: "mostly stone with some occasion timber for accent".
3. **Desktop only; no phone layouts.** The owner, 2026-11-14: "phones cant run
   this lol". The UI scale is a whole number from the window height.
4. **Drawn pixel art, not rendered-then-pixelated.** Recommendation, still open
   as Q1, with screenshots.
5. **One glowing accent, the rune.** Selection and focus only; the mint stays
   the one glowing colour. Recommendation taken (ask only with screenshots).
6. **Labels inlaid light in stone.** No dark ink reaches 4.5:1 on mid stone, so
   plaque labels are chalk (`LABEL`) and only the lintel's title is cut dark.
   Recommendation taken (ask only with screenshots), measured in section 4.
7. **Pixelify Sans + VT323**, VT323 for the compass letters. Recommendation
   taken (ask only with screenshots).
8. **Integer UI scale from the window height, nearest sampling.** The art
   direction's rule, not a choice.
9. **No night dimming of the UI.** Recommendation taken (ask only with
   screenshots); both HUD shots are in the mockup.
10. **The quiet screen holds.** In play: crosshair, hotbar, compass. The
    instrument line stays until `near-field-streaming` takes it.

## 10. Owner questions (each with screenshots)

| Id | Question | Screenshots | Status |
| --- | --- | --- | --- |
| Q1 | Drawn or rendered-then-pixelated panels? | `docs/screenshots/mockup-ui/q1-drawn-vs-rendered.png` (the rock slab both ways, both in the slate-blue rock) and the Kit tab | **Open.** Recommendation: drawn |
| Q2 | Menus framed in oak round stone, or in rough rock? | `docs/screenshots/mockup-ui/q2-oak-vs-rock.png` (kept as the record of the question) | **Answered: stone** (owner, 2026-11-14), then "mostly stone with some occasion timber for accent" |

Q1 belongs in the survey (`docs/design/survey.html`, the `html-survey` skill)
beside its screenshot when the page is published; Q2 moves to its "Already
decided".

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
  900 window. Under 720 px a scene stays 720 px wide and scrolls inside its own
  box: the game has no narrow layout to show, and the page itself never
  scrolls sideways.

## The owner's answers

### Stone, and bluer (2026-11-14)

The owner: "I like the stone kit more, can you make the stone more blue?". Taken as the answer to
Q2: menus are framed in the rough-rock slab, not the carved-oak moulding. Oak stays for the
button plaques, tabs and the compass strip, and the oak frame stays in the kit for comparison.
The stone moves from warm grey to Sweaft-64's slate blues: dressed stone `#011c34 #1f3b52
#3e586e #92a3b2 #bfcad7` (FACE `#1f3b52`, WELL `#011c34`), rough rock `#010e1a #1f3b52 #3e586e
#5d778d #92a3b2`. Text on the new face measures TEXT 6.49:1, TEXT_DIM 5.38:1, TITLE 6.29:1, the
rune 7.55:1, all above 4.5:1.

(Superseded in part by the next answer: oak no longer makes the plaques, the
tabs or the compass strip.)

### No phones; mostly stone, timber as accent (2026-11-14)

The owner: **"phones cant run this lol, mostly stone with some occasion timber
for accent"**. Folded in as:

- The phone layouts are withdrawn: the mockup's Phone tab and phone frames,
  the wrapped hotbar, the full-width menus, the phone-width requirement and
  scenario in the spec delta, and the phone task. The game is desktop only;
  its UI scale stays a whole number from the window height (section 6). The
  mockup page still does not scroll sideways at 390 px, an artifact rule for
  the page, not a game requirement.
- Every piece is redrawn stone-first (sections 3 and 5): the plaques, tabs,
  compass strip, scroll thumb, slider knob, tooltip, bar frame, map frame,
  portrait and icons move from oak or iron to stone, and `frame` becomes a
  dressed-stone moulding. Oak stays in three accents (section 3a). Labels move
  from dark-on-oak to chalk-inlaid-on-stone, and every pair is re-measured
  (section 4, all at 4.65:1 or above).
- Q1 stays open. The rendered rock was re-rendered in the slate-blue rock ramp
  (`ui_panels_blend.py --only rock`, Cycles on the CPU) so the comparison is
  fair; the rendered oak frame is no longer shown.
