# Design: rock and stone in play, carved oak out of play

The owner, 2026-11-14: **"pull in UI skills from my fleet-command repo, and
develop ui for this game, do something like with rock and carved wood"**;
then **"I like the stone kit more, can you make the stone more blue?"**; then
**"phones cant run this lol, mostly stone with some occasion timber for
accent"**.

Then, the same day: **"show me the game with that 3d rendered look"**, **"yea
the 3d rendered look as a whole kit, also make the menus interactive please"**,
**"make a 3js demo"**, **"main menu I think should use the timber stuff, but the
game I like the rock stuff"** and **"pause menu = timber too"** (all recorded in
"The owner's answers" below).

Mockup: `docs/mockups/ui.html` ("Stone and Timber"), tabs Play, Main menu, Kit,
HUD, Pack, Pause, Map, Station, Talk, Loading, with a page-wide look switch
Drawn | Rendered (pixelated) | Smooth (Cycles). Its shots are in
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

## 3. Materials: rock in play, timber out of play

The owner, 2026-11-14: **"main menu I think should use the timber stuff, but
the game I like the rock stuff, just need to determine which rock thing to
use"**, then **"pause menu = timber too"**. So the kit has two sets:

| Where | Set | Pieces |
| --- | --- | --- |
| **Out of play**: the main menu (title), pause, settings, saves | **Timber**: the carved-oak set (`ui_art.py --oak`, `docs/mockups/ui/oak/`) | oak frame round a stone face, oak plaques in five states, oak tabs (top level and Input's sub-tabs), oak scroll thumb and slider knob, oak-rimmed tooltip used as the key cap, the oak lintel for the title |
| **In play**: HUD (hotbar, compass, crosshair), pack, map, machines and stations, talk and trade, chests, the loading slab | **Rock and stone**: rough rock frames, slate-blue dressed stone inside | rock slab, stone plaques, slots, tabs, bars, scroll, slider, checkboxes, the compass strip (oak pointer), map frame, portrait, tooltip; the oak lintel carries a panel's title |

| Material | Where | Ramp (Sweaft-64, dark to light) |
| --- | --- | --- |
| Rough rock | In-play panel frames (pack, stations, chest, talk, trade), the loading slab, the map frame's corner blocks, the emblem | `#010e1a #1f3b52 #3e586e #5d778d #92a3b2` |
| Dressed stone | Faces text sits on; in-play plaques, tabs, compass strip, slots, scroll, slider, checkboxes, tooltips, bar frames, the map's moulding, the portrait frame, the sub-panel frame | `#011c34 #1f3b52 #3e586e #92a3b2 #bfcad7`, plus `STONE_MID` `#5d778d` |
| Carved oak | The timber menus; in play only the title lintel, the open tab's cap and the compass pointer | `#2f1525 #41322c #725244 #ab8465 #cfaa81` |
| Iron, brass | Fittings: studs, brackets, bands, pegs; brass on hover and town studs | `#010000 #161612 #2c2c25 #434137 #a09f91`; `#733126 #c56d25 #fb902f #faaf5c` |
| Rune glow | Focus and selection only | `#204f63 #53998b #84e3b5 #c1f1d8` |

Rules from the art direction kept: lit from the top left, a dark seam between
two materials, alpha 0 or 255 in the pixel looks, every colour from the
palette. The drawn kit keeps its 1 px `#010000` outline; the rendered and
smooth looks have **no outline** (owner: "the original didn't have a black
outline").

### 3a. Timber in play

In play, oak stays three small accents (the title lintel pegged over a rock
frame, the open tab's cap, the compass pointer), as decided earlier ("mostly
stone with some occasion timber for accent"). Out of play, the whole menu is
oak.

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

- **The kit's own pixel font** for every title, plaque and tab label, section
  label, compass letter and key cap. The owner, 2026-11-14, of a "PACK" lintel
  that read as "PAOK" in Pixelify Sans: **"the c there is kinda weird, which
  you'd fix it"**. `ui_art.py` draws it: 5 x 7 capitals, digits and
  punctuation, a 1 px stroke, open C and G, a flat-backed E, an open S, a
  slashed 0 against a round O, a flagged 1 against a serifed I
  (`docs/mockups/ui/font/kit-font.png` + `kit-font.json`, each glyph's x, width
  and rows). The page draws it as whole pixels (an inline SVG of the glyph runs,
  filled with the element's colour, a 1 px shadow under it), so its scale is
  always the UI scale. Glyph sheet: `docs/screenshots/mockup-ui/ui-font-glyphs.png`;
  the lintel before and after: `pack-title-before-after.png`.
- **VT323** (Google Fonts, OFL) stays for body text: lines of talk, settings
  rows, recipe names.
- In the game the kit font is a Bevy bitmap font: the PNG as an image atlas,
  the JSON's metrics as the glyph table, drawn at the integer UI scale with
  nearest sampling (section 8).

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

## 7. Three looks of one kit: drawn, rendered, smooth

The owner, 2026-11-14: **"show me the game with that 3d rendered look"**, **"yea
the 3d rendered look as a whole kit"**, then of the three rock frames on the Kit
tab (drawn, rendered and pixelated, the smooth Cycles render): **"do all
three"**. Every piece exists three ways and the mockup's switch swaps the whole
page, the Play demo included:

| Look | Made by | Files | Sampling |
| --- | --- | --- | --- |
| Drawn | `tools/art/ui_art.py`, pixel by pixel | `docs/mockups/ui/*.png`, `ui/oak/*.png` | nearest, whole-number scale |
| Rendered | `tools/art/ui_panels_blend.py`: modelled in Blender, rendered, reduced to the drawn piece's pixels and snapped to its material's Sweaft-64 ramp, no outline | `ui/rendered/*.png`, `ui/rendered/oak/*.png`, the rock per panel in `ui/rendered/rock/` | nearest, whole-number scale |
| Smooth | the same renders, not reduced | `ui/rendered/hires/` (8x, so every 9-slice margin is the drawn one times 8), the rock per panel in `ui/rendered/hires/rock/` (4x) | linear |

**Smooth breaks the art direction's rule** that UI art is pixel art sampled
nearest. It is offered because the owner asked to see it, and it stays a
question (Q1), not a decision.

### 7a. The base rock (approved 2026-11-14)

The owner liked the first smooth rock render (`rock_field`, the frame in Q1's
first comparison): **"I liked this version, please keep this version, but I want
one without the lines!"**, **"also shadow tiling can be better"**, **"I want to
make sure you make the base one good before committing"**, **"i wanted to see
that one without the lines, prioritize this render"**, **"get me a good sample
first before you make the kit"**, and of an attempt that added one: **"that is
wrong — the original didn't have a black outline"**. The parent session made the
sample from the original script with two changes only, and the owner approved
it: **"Ok good continue with the mockups"** (2026-11-14;
`docs/screenshots/mockup-ui/base-rock-approved.png`). The base rock is that
sample, exactly:

- **The first render's field, light, camera and colours, unchanged**:
  `rock_field()` (fbm relief at 0.12 and 0.5 a pixel, the rim's rounded fall to
  a chipped silhouette, the step onto the lip, the slate-blue rock blend), the
  sun at Euler (-40, -28, 0) degrees at 3.2, a 0.35 sky, Blender's default
  specular, roughness 0.7, an orthographic camera face-on, Cycles on the CPU (no
  GPU here), 24 samples.
- **Change 1: the strata rings are removed.** `sin(d * 3.1) > 0.92` drew rings
  at fixed distances from the edge: straight streaks along the rim.
- **Change 2: each panel is rendered at its own size**
  (`rock_field(W, H)`, `--w/--h` in the sample), not as a 48 px tile. The old
  frame was a 9-slice whose 20 px sides repeated, so the rim's bumps and the
  shadow the lip casts on the face repeated every tile, and the tiled centre
  carried the top of that shadow down the face as a faint dashed line. Rendered
  at its size, nothing repeats.
- **No outline in any look.** The smooth rock is the render's own alpha edge;
  the pixelated rock is the render reduced, alpha cut at one half, no INK ring.

The in-play rock panels and their sizes (native px, borders included; the
script's `ROCK_PANELS`, written to `ui/rendered/slices.json` as `rock_panels`;
the page sizes each panel exactly):

| Panel | Size | | Panel | Size |
| --- | --- | --- | --- | --- |
| pack | 268 x 152 | | trade | 308 x 148 |
| station (forge, hearth, bench, mill, press) | 230 x 156 | | loading slab | 150 x 148 |
| chest | 372 x 168 | | the approved sample | 192 x 96 |
| talk | 270 x 124 | | kit demos | 48 x 48, 128 x 64, 200 x 100 |

The tooltip, the map's drawer frame and the map frame are dressed-stone
9-slices whose sides vary only across themselves, so they stretch with nothing
to repeat. The drawn look keeps the drawn 48 px slab as a 9-slice (its 20 px
sides now carry irregular cracks and speckle, not dashes; below).

**In the game**, a rock panel is either baked at its size (most panels have one
size at a given UI scale), or, for one that must grow at run time, baked once
with a long edge (longer than the panel can get) that is only ever cropped,
never tiled. No visible repeat is the rule.

### 7b. Every other piece carries the same material

Plaques, slots, tabs, bars, scroll, slider, checkboxes, the compass strip, the
dressed-stone frame, the map frame, the portrait, the emblem, the icons and the
oak set are each modelled as one height field with material masks (the
`blender-ui-panels` / `blender-organic-panels` method: rolled edges, sunk wells,
cut grooves, forged studs, an oak grain that runs along the board), at the drawn
piece's size and layout, and rendered under the base rock's light. Their stone
carries rock_field's surface: the same two fbm layers and the same light-dark
blend, full in the corners and varying only across a stretching band, so a
stretched side stays plain. The relief is exaggerated 1.7x so a one-pixel bevel
still turns a tone after the snap. The faces text sits on are flat.

### 7c. The drawn rock's lines

The owner: **"how come the rock one has lines in it?"**. The drawn slab had
evenly spaced dashes on rows 5 and 42 (and the matching columns), repeating
every 20 px with the tile, which read as a stitched line. `rock_panel()` now
lays out short cracks at varied angles and lengths (1 to 4 px), pits and
light and dark speckle per side, at varied depths across the rim, still once
per 20 px so the sides tile.

### 7d. Contrast in every look

`ui_panels_blend.py` measures each text pair on every rendered piece: in the
pixelated look against the commonest colour of the text area and any colour
covering a tenth of it, in the smooth look against the 10th, 50th and 90th
luminance percentile. A pair under 4.5:1 is fixed by changing the face's value
(the albedo under the text, or one step along its ramp in the pixel look),
never by dropping the rule. Two pieces needed it: the stone plaque's disabled
face (darkened to 0.444 of its albedo) and the compass strip (lightened to
1.15). The oak disabled plaque is WOOD[0], a step darker than the drawn one.
The worst pairs: smooth TEXT_DIM on the rock face 4.99 to 5.0, LABEL on the
hovered plaque 4.57 to 4.65. Every number is in `ui/rendered/slices.json`.

### 7e. Re-running the bake, and what it costs

    python3.11 -m venv <dir> && <dir>/bin/pip install "bpy==4.5.*" pillow numpy
    <dir>/bin/python tools/art/ui_panels_blend.py [--samples 24]          # the whole kit and every rock panel
    <dir>/bin/python tools/art/ui_panels_blend.py --only rock --rocks pack # one rock panel
    <dir>/bin/python tools/art/ui_panels_blend.py --only button-idle,oak/frame

The whole bake is about 4 minutes on this container's 4 CPU cores (no GPU; on
the owner's desktop it runs on the GPU): 0.1 to 25 s a piece, the largest rock
panel (372 x 168) 25 s. **At run time it costs nothing**: the rendered and
smooth looks are static textures like the drawn ones, loaded once; a smooth
texture is larger (8x or 4x a side), which is memory, not frame time.

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
| Fonts | `assets/ui/kit-font.png` + `.json`, `assets/fonts/VT323` | The kit font as a Bevy bitmap font: the PNG an image atlas sampled nearest, the JSON's glyph rows and widths its metrics, drawn at the integer UI scale; VT323 (OFL) as a `TextFont` for body text |
| Rock panels | `ui_kit.rs`, the rendered look | One baked rock texture per in-play panel size (`rock_panels`), or a long baked edge only ever cropped; never a tiled rock edge |
| Timber menus | `menu.rs` (pause, settings, saves), the title screen | The oak set: `nine(OakFrame)`, oak plaques and tabs; Input split into sub-tabs (On foot, Hands, Flying, Vehicles, Menus), each row action, key cap, Change |
| Station, talk, trade | the crafting, cooking, townsfolk and market changes | Built from the same kit when those screens are built; nothing new here |

## 9. Decisions

1. **Rock in play, timber out of play.** The owner, 2026-11-14: "main menu I
   think should use the timber stuff, but the game I like the rock stuff",
   "pause menu = timber too".
2. **Slate-blue stone and rock.** The owner's answer to Q2 ("I like the stone
   kit more, can you make the stone more blue?").
3. **Desktop only; no phone layouts.** "phones cant run this lol".
4. **The base rock is the approved sample** (section 7a): the first render
   minus the strata rings, rendered at each panel's size, no outline. The
   owner, "Ok good continue with the mockups".
5. **The kit's own pixel font** for titles, labels, tabs, compass letters and
   key caps; VT323 for body text. "the c there is kinda weird, which you'd fix
   it".
6. **Settings, Input in sub-tabs.** "organize the input menu better with tabs
   or something instead of big uber list": On foot, Hands, Flying, Vehicles,
   Menus; one row per action (action, key cap, Change; a clash noted on the
   row); Reset tab. Rows and keys are the game's (`controls.rs` BINDINGS); two
   proposed rows are marked proposed (crouch / sink, drop stack).
7. **One glowing accent, the rune.** Recommendation taken (ask only with
   screenshots).
8. **Labels inlaid light in stone, cut dark in oak.** Measured (section 4).
9. **Integer UI scale from the window height, nearest sampling** for the drawn
   and rendered looks. The smooth look is the exception the owner asked to see;
   it is Q1, not decided.
10. **No night dimming of the UI.** Recommendation taken.
11. **The quiet screen holds.** In play: crosshair, hotbar, compass.

## 10. Owner questions (each with screenshots)

| Id | Question | Screenshots | Status |
| --- | --- | --- | --- |
| Q1 | **Which rock for the game: drawn rock, rendered rock (pixelated), or smooth rock (Cycles)?** (the owner: "just need to determine which rock thing to use") | `docs/screenshots/mockup-ui/q1-which-rock-hud.jpg`, `q1-which-rock-pack.jpg` (in-play screens only, the three side by side, after both rock-line fixes); the Kit tab | **Open.** Recommendation: rendered rock (pixelated), the approved base rock in the game's pixel style. Smooth breaks the nearest-sampling rule |
| Q2 | Menus framed in oak round stone, or in rough rock? | `q2-oak-vs-rock.png` | **Answered: stone** (2026-11-14), then rock in play and timber out of play |

Q1 goes in the survey (`docs/design/survey.html`) beside its screenshots when the
page is published; Q2 moves to "Already decided".

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

### The whole kit rendered, interactive, and a demo (2026-11-14)

The owner's words, in order, all 2026-11-14:

1. **"show me the game with that 3d rendered look"**: every tab of the mockup
   can be shown in the rendered look (section 7).
2. **"yea the 3d rendered look as a whole kit, also make the menus interactive
   please"**: every piece rendered; every menu works on the page (pause: Resume,
   Saves, Settings, Quit with an in-page confirm; settings: tabs, sliders that
   drag, checkboxes, Rain shadow off by default, Back; the pack: pick up, put
   down, right click for half, hover for name and count, 1-0 and the wheel; the
   map's drawer and overlay rows; stations: a recipe, its ingredients from the
   pack's real counts, a Make button that runs the bar and drops the result into
   the pack; talk choices; trade with a running total; the loading bar). Keys act
   only while a game screen has focus, so the page still scrolls.
3. **"make a 3js demo"**, then **"a simple world scene that is beautifully
   rendered that I can walk around in, interact with machines and stuff, open my
   inventory"**: the Play tab (`docs/mockups/ui/play.js`, three.js r128, one
   renderer made when the tab opens and stopped on other tabs). A hex hillside at
   the game's cell (2.833 m, 1 m layers) with a stream and pond, trees, a stone
   workshop, a straw hut, a lookout, lamp posts and a campfire; a sky with a sun
   and clouds, one shadow-casting light (the sun, or a dim moon at night), a
   hemisphere fill, fog in the sky's colour, ACES and sRGB, animated water with
   the sky in it, vertex ambient occlusion on the columns, instanced grass and
   flowers, sparks, smoke and motes; golden hour by default, Day, Dusk, Night and
   a time slider, and at night every lamp, window and fire lights what is near
   it. First-person walking (WASD, Space, mouse with pointer lock, drag to look
   where the lock is refused), gravity, no walking through columns; left click
   digs into the pack, right click places the held block; E uses what is in
   front: the bench, the forge and bellows (it glows hotter and throws sparks
   while it smelts), the hearth (smoke while it cooks), the water mill (the
   stream turns its wheel), the brick press feeding a chest along a conveyor, the
   chest, and Wren the smith. The frame rate was not measured (no GPU here).
4. **"do all three"**: the look switch is Drawn | Rendered (pixelated) | Smooth.
5. **"how come the rock one has lines in it?"**, then **"the 3d rendered one"**:
   section 7c (the drawn rock) and 7a (the render).
6. **"main menu I think should use the timber stuff, but the game I like the rock
   stuff, just need to determine which rock thing to use"**: timber out of play,
   rock in play (section 3); Q1 is now only "which rock".
7. **"pause menu = timber too, and organize the input menu better with tabs or
   something instead of big uber list"**: pause and settings in oak; Input in
   sub-tabs (decision 6).
8. **"the c there is kinda weird, which you'd fix it"**: the kit font
   (Typography).
9. **"I liked this version, please keep this version, but I want one without the
   lines!"**, **"also shadow tiling can be better"**, **"I want to make sure you
   make the base one good before committing"**, **"i wanted to see that one
   without the lines, prioritize this render"**, **"get me a good sample first
   before you make the kit"**, **"that is wrong — the original didn't have a
   black outline"**: section 7a.
10. **"Ok good continue with the mockups"**: the base rock approved, from the
    parent session's sample (`base-rock-approved.png`); the kit carried its
    material after that, and not before.

## No close X in the game (owner, 2026-11-14)

The owner: "We don't need an x button in the actual game this is for just mobile. In desktop esc
will do just fine." The game's panels shut with Esc (and E for a machine, I for the pack, M for the
map); none gets a close X. The X in `docs/mockups/ui.html` is the mockup page's own, shown only on a
touch screen so the page can be used from a phone, and is not part of the design.

## Decided: smooth rock, rendered smooth timber (owner, 2026-11-14)

The owner, after the three looks side by side (`docs/screenshots/mockup-ui/compare/`): "Smooth is
the best. Write it down as the decision and start working the current UI as the wood + rendered
smooth rock". Q1 is answered: the game's UI uses the **Smooth** look, the Blender renders sampled
linear, not reduced to pixels:
- in play, the approved base rock (`docs/mockups/ui/rendered/hires/rock/`, baked per panel size)
  and the smooth stone pieces (`rendered/hires/`);
- out of play (main menu, pause, settings, saves), the carved oak in the same smooth render
  (`rendered/hires/oak/`).
This is the owner's own exception to nearest sampling, for the UI only: the UI's kit textures are
sampled linear. Terrain, blocks, items and every world texture stay nearest (CLAUDE.md, "Terrain
textures use nearest point sampling"). Item icons inside slots stay pixel art, nearest. The kit font
stays the pixel bitmap font, nearest, at whole-number scale. The drawn and pixelated looks stay in
the mockup as the record, not in the game. The owner's words also count as the go-ahead to start
section 1 of `tasks.md` (the kit in the game).

## Section 1 as built: the Bevy 0.18.1 names, and what differs from section 8 (2026-11-14)

Read off the crate sources (`bevy_ui` 0.18.1 `widget/image.rs` and `lib.rs`,
`bevy_sprite` 0.18.1 `texture_slice/`, `bevy_ui_render` 0.18.1
`ui_texture_slice_pipeline.rs`, `bevy_image` 0.18.1, `bevy_text` 0.18.1):

| Name | As it is in 0.18.1 |
| --- | --- |
| `ImageNode` | `bevy::ui::widget::ImageNode { color, image, texture_atlas: Option<TextureAtlas>, flip_x, flip_y, rect, image_mode }`; builders `new`, `from_atlas_image`, `with_color`, `with_mode` |
| `NodeImageMode` | `Auto`, `Stretch`, `Sliced(TextureSlicer)`, `Tiled { tile_x, tile_y, stretch_value }` |
| `TextureSlicer` | `bevy::sprite::TextureSlicer { border: BorderRect, center_scale_mode, sides_scale_mode, max_corner_scale }` |
| `BorderRect` | **differs from older Bevy**: `{ min_inset: Vec2, max_inset: Vec2 }` (min is left and top, max right and bottom), with `all` and `axes`; there are no `left/right/top/bottom` fields |
| `SliceScaleMode` | `Stretch`, `Tile { stretch_value }` |
| `UiScale` | `bevy::ui::UiScale(pub f32)`; it scales `Val::Px` and text, and the slicer's target size is taken after it |
| `ImageSampler` | `bevy::image::ImageSampler::{linear(), nearest()}`, per image (`Image::sampler`) |
| Atlas | `TextureAtlasLayout::new_empty(UVec2)` + `add_texture(URect)`, `TextureAtlas { layout, index }` |
| Default font | `TextPlugin` inserts Fira Mono at `AssetId::<Font>::default()`; inserting another `Font` there replaces the face of every `Text` |

**How a slice corner scales (the risk in section 12, measured in the source).**
`compute_texture_slices` draws a corner at `border * min(target / image,
max_corner_scale)` UI units, `target` being the node's size in UI units. So a
corner GROWS with the node until the texture's own size unless
`max_corner_scale` caps it. The game bakes each piece at the screen's pixels a
native pixel (`s`), gives the slicer the margins times `s`, and sets
`max_corner_scale = 1/s`: a corner is then its native size in UI units, and
`margin * UiScale` screen pixels, at every node size
(`ui_kit::slicer_at`, test `the_kit_bakes_linear_and_the_font_nearest`).

What differs from section 8's plan, and why:

- **The kit is `crates/pbd-app/src/ui_kit.rs`, in the library, not
  `desktop/ui_kit.rs`.** The loading screen (`loading.rs`) is in the library
  and draws the kit's slab; the binary's `desktop/` cannot be reached from
  there.
- **Smooth, linear, resampled per scale.** Section 8 was written for the pixel
  looks (nearest). The owner chose Smooth, so the kit's pieces are sampled
  linear; drawn straight from the 8x renders they would be minified four to one
  with no mip chain. Each piece is resampled once on the CPU (premultiplied,
  Catmull-Rom) to exactly `s` texels a native pixel when the kit is installed
  and again when the window crosses a step of the scale, into the same handles.
  Item icons and the pixel font stay nearest.
- **Decoded before the first frame, not through the asset server**, in the
  plugin's `finish`, so the loading slab is on the first frame the window shows
  (CLAUDE.md, "A loading screen on boot").
- **The map frame is hollow**: a Bevy slicer always fills the centre, so the
  bake clears the face inside the map frame's 22 px margins.
- **The kit font is drawn as glyph image nodes** (`KitText`): one atlas
  image node per glyph, nearest, whole native pixels, a one-pixel shadow node
  under it where carved. Bevy's `Text` takes only outline fonts.
- **VT323 is installed as the default face** rather than named in each
  `TextFont`, so every line of body text reads in it.
- **`UiScale` is `s / window scale factor`**, so the screen pixels a native
  pixel is the whole number `s` on a high-DPI display as well.
- Settings got its tabs (Picture, Weather, Movement, Input); Input is the
  binding table under a heading per context with each key in a cap, scrolled
  by the wheel. The sub-tabs and rebinding (decision 6) are new behaviour and
  stay in this change for a later step. Frame limit and render scale are oak
  plaques that step through their values, as the rows did; bloom, rain shadow
  (still off by default) and auto step are checkboxes.
- The pack carries only its slots under the PACK lintel: the old CLOSE (I)
  button and the line of mouse instructions are gone (no close button in the
  game; "All text is excess on the UI"). I and Esc shut it as before.
- The map keeps its legend as a drawer that is always open (the mockup's
  shut-by-default drawer would be a behaviour change).
- **The pause panel is the owner's menu only** (Resume, Saves, Settings,
  Quit; `menus-and-a-quiet-screen`). The weather, the clock and the globe
  overlay that sat on it made it taller than a 900 px window at scale 2 (the
  first capture had its lintel off the top of the screen). They moved to
  Settings: the storm forcing beside the rain shadow on the Weather tab, the
  clock and the globe overlay on a new World tab. Nothing scrolls off screen.
- Body text's shadow is one native pixel in ink (`ui_kit::text_shadow`):
  Bevy's default of 4 px read, at scale 2, as a second copy of the line.

## Section 2: Input in sub-tabs with rebinding, sliders, tooltips (written 2026-11-14, before the code)

Written up first, as CLAUDE.md asks; the code follows it. Decision 6 is the
owner's ("organize the input menu better with tabs or something instead of
big uber list"); everything in this section beyond the owner's words is a
recommendation taken (ask only with screenshots).

### The sub-tabs

The Input page gets one row of oak sub-tabs, the settings tabs' own piece:
**ON FOOT, HANDS, FLYING, VEHICLES, MENUS** (decision 6). Each holds the
groups of `controls::BINDINGS` that belong to it, a heading per group and one
row per action, scrolled by the wheel when it is longer than the page:

| Sub-tab | Groups (headings) | Rows |
| --- | --- | --- |
| ON FOOT | ON FOOT | move, jump / rise in water, sprint |
| HANDS | HANDS | open or shut a door, the tool in hand, place, select a slot, step the slots, the tools (G) |
| FLYING | FLYING, KESTREL | free flight; the Kestrel's pitch, roll, yaw, power, nacelles, assist, wheel brake |
| VEHICLES | VEHICLES, TERN, LOON, ROWBOAT, COG | board, view, anchor; each boat's own |
| MENUS | WORLD, SCREEN | map, walk or fly, spawn, weather, noon or midnight; menu, screenshot, frame graph, the guide, the pack |

`BINDINGS` stays the one table: the old ON FOOT group is split in two (ON
FOOT and HANDS) and the field guide and the pack move to SCREEN, which is a
regrouping of the same rows and keys; `--help` prints the new headings. The
two proposed rows of decision 6 (crouch / sink, drop stack) are not added:
the game has no such actions, and a row for one would be a key that does
nothing (`controls::tests::a_named_key_is_a_key_something_reads`).

### Rebinding: what can be changed

A **slot** is one keyboard key in one row: `W A S D` is four slots, `SPACE /
CTRL` two. A slot is named by its group and its default key
(`"flying:KeyQ"`): within a group no key appears twice (a test), and the
default key is what the game's readers press, so the name says exactly what
a rebind replaces. Mouse buttons, the wheel, the number row's `1 - 0` range
and `ESC` are shown in their caps but are not slots: the mouse is not a key,
the range is ten readers' keys at once, and Esc is the key that cancels a
rebind and always opens the menu.

The flow, as the owner put it (no close X; Esc closes):
- **Click a key cap.** The cap reads `PRESS A KEY`; any other cap or sub-tab
  or leaving the page cancels it.
- **Press a key.** The slot takes it. A key with no label in the kit's font
  (outside `controls::ASSIGNABLE`: letters, digits, F1-F12, the arrows,
  space, tab, enter, backspace, shift, ctrl, alt, the punctuation the font
  has, the numpad digits, home, end, insert, delete, page up and down) is not
  taken and the cap goes on waiting.
- **Esc cancels**, and is consumed, so it does not also step the screen back.
- A slot set back to its default key is removed from the file, so a player
  who never rebinds has an empty list and a file like today's.
- **A clash is flagged on the row**: when a slot's key is a key another slot
  reads in a context that is live at the same time, the row says `also:`
  and the other action. The defaults share keys between contexts on purpose
  (W is move on foot, pitch in the Kestrel and a sheet in the Tern); those
  are never live together, so they are not flagged.
- **RESET TAB** puts the open sub-tab's slots back to their defaults.

### Rebinding: how the game obeys it (one remap, not a rewrite of every reader)

The game reads keys in a dozen systems by their `KeyCode` (`walking.rs`,
`flight_view/input.rs`, `vehicles.rs`, `weather.rs`, `desktop/*.rs`), and a
key means different things in different contexts. Rewriting every reader to
ask an action table would touch every system and its tests. Instead, one
system, `controls::remap_keys`, runs in `PreUpdate` right after Bevy's input
systems and before the menu's, and rewrites `ButtonInput<KeyCode>` for the
keys a rebind touches, so every reader goes on reading its default key:

1. It keeps its own copy of the **physical** keyboard from the
   `KeyboardInput` messages (and lets every key go on `KeyboardFocusLost`),
   exactly as Bevy's `keyboard_input_system` does.
2. It works out the **live context**: aboard a craft of kind K (`Aboard`),
   else on foot (`WalkingReadout::active`), else flying. Each group says
   when it is live (`controls::When`): ON FOOT and HANDS on foot; FLYING in
   free flight; VEHICLES on foot (F boards) and aboard; KESTREL, TERN, LOON,
   ROWBOAT and COG aboard that kind; WORLD and SCREEN always.
3. For every default key `c` of a live slot that has been moved, or whose
   key some live slot now uses, it sets `c`'s pressed, just-pressed and
   just-released state from the physical keys of the live slots whose
   default is `c` (an unmoved slot is its own key). A key that is no live
   slot's default keeps its physical state.
4. **While a menu is open the remap is off** (every key its physical
   self), so the menus, the name field and the rebind capture read the real
   keyboard; and keys it touched last frame but not this one are put back
   to their physical state.

With no rebinds it touches nothing at all, so a player who never rebinds,
every `--capture` and every benchmark (which read no settings file) run
exactly the game of today; the readers' own tests still press the default
keys directly. What the remap cannot do, on purpose: give one key two
meanings in the same context except by the clash the row already flags, or
rebind the mouse.

### Rebinding: the file (old files still load)

`PlayerSettings` stays a small `Copy` value. The rebinds are their own
resource, `controls::KeyBinds` (slot name to key, a `BTreeMap` so the file
is written in one order), kept in the same `settings.ron` through the
existing path (`SettingsFile::keep`, the temporary file and the rename).
The file goes to **version 3**, which adds `keys: { "flying:KeyQ": "KeyZ" }`
with a default of none:
- a file of version 0 to 2 has no `keys` and reads as no rebinds, every
  other field as before (a test reads a version-2 file);
- a slot or a key this build does not know (a later build's) is skipped with
  a warning, not an error, and the rest of the file still loads;
- a version-3 file read by an older build: serde ignores the unknown `keys`,
  and `Loaded::writable` already refuses to write a newer file back, so the
  rebinds survive;
- a capture or a benchmark (`settings::for_launch` isolated) reads no file
  and has no rebinds.

### Sliders for the frame limit and the render scale

The two value plaques become the stone groove with the oak grip
(`menu::slider`, the time and weather bars' own), with the value beside it in
the kit's font. The values and steps are the plaques' exactly: the frame
limit OFF, 60, 120 (`FrameLimit`) and the render scale 50, 67, 75, 85, 100
(`settings::SCALES`, shown low to high). A press or a drag sets the step
nearest the cursor, and the grip and the lit part of the groove sit at that
step; the choice is kept through `SettingsFile::keep` as before.

### Tooltips (in the kit)

`ui_kit::Tooltip(String)` on any node with an `Interaction` is the text the
kit shows while the pointer is over it, in one stone `tooltip` piece
(VT323 body text, the 6 px margin) drawn over every panel. It stands above
the hovered node, centred on it, 3 px clear, so it never covers that slot's
count or its neighbours' (the coordinator's review of the first capture,
where it hung below the pointer over the counts and off the window's
bottom by the tool slot); below the node only when there is no room above,
and clamped 3 px inside the window on every side. It
shows only while a menu holds the pointer, since in play there is no cursor.
The slots keep their own tooltips from the store: an empty slot has none, a
stack its name and count (`Stone x 12`), a tool its name and purpose; the
pack's thirty, the hotbar row (in the pack and on the HUD, which gets an
`Interaction` for it) and the tool slot, which says what is in hand.

### The map's drawer stays open (recommendation)

The map's legend drawer stays open by default, as it is. The mockup's
drawer is shut by default and opened by a plaque; shutting the game's by
default would change what a player sees on opening the map, so it is a
**recommendation, not done**: shut by default, opened by a DRAWER plaque on
the frame and by a key, with the choice remembered for the run. Recorded in
`tasks.md` section 2 so the owner can take it or leave it.

## Crafting in the pack (owner, 2026-11-14; mockup only)

The owner: "the pack should also include some kind of basic crafting system, with a drop down list,
and a button to look at crafting trees/recipes". The M29 mockup now shows it
(`docs/mockups/ui/game-ui.js`, `craftRow()` and `recipes()`; `docs/screenshots/mockup-ui/pack-crafting.jpg`):

- **Below the hotbar, a Craft row:**
  - a stone drop-down of what the hands can make, where a recipe short of something is greyed;
  - what it takes, as wells lit or dark against what is carried;
  - an arrow, the result, and Craft.
- **Recipes** opens a rock panel with every made thing as a grid of wells. Picking one shows its tree:
  each ingredient under it, indented, with where it is made ("by hand", "at the forge, coal") or
  "found, have N", down to what is dug or caught.
  - "Craft in the pack" takes a by-hand recipe back to the drop-down.
  - Back, Esc or E returns to the pack.
- **The rock:** the pack grows to 268 x 204 native px for the row. Its rock is the approved base rock,
  rendered at that size (`pack-craft` in `tools/art/ui_panels_blend.py`).
- **The recipes are mockup data:** planks, torches, shovel, axe and rod by hand, plus the stations'
  own recipes. The game has no crafting yet.
- **Not built:** this is a design shown in the mockup, not a requirement. It joins the game through
  its own change when the owner asks for crafting there.

## No notes in the game's screens (owner, 2026-11-15)

The owner: "notes belong in mockups and chats, not the real game" (CLAUDE.md,
"Notes stay out of the game"). The settings pages carried developer notes as
hints beside their labels. These are removed, and the rows keep their labels
and controls. (Bloom's "costs ~2.4 ms on a Mac" is the owner's to remove:
"dont worry about the mac note ... I'm removing it".)
- Render scale: "the HUD stays sharp".
- Rain shadow: "the rain fog and streaks; off by default".
- The line under it: "The near shower, the wet ground and the drops on the
  lens still draw."
- Auto step: "walk up one layer without a key".

The Input page's "Click a key, then press the new one. Esc cancels." stays: it
is how the control is used. The "also: …" note on a clashing key also stays: it
shows the player's own bindings, not a design note.

The debug readout (column, position, elevation) is a developer overlay. A
recommendation, not done here: show it only behind its debug key and keep it
off by default. That changes behaviour, so it waits for its own go-ahead.

## Interact on F, and the prompt that names it (owner, 2026-11-16; written before the code)

The owner: "interaction with plants, I should be able to harvest plants by
pressing F on them, we should use F for interact since E will be used for
roll in space", and then "F to harvest should popup, or F to open work
bench etc." Everything below beyond those words is a recommendation taken
(ask only with screenshots).

### The key move

- **F is the one interact key on foot**, for everything E did: open a
  station (workbench, forge, campfire), harvest a ripe crop, pick a wild
  plant or a tree's fruit, till with the shovel, feed, tame, lead and
  collect from an animal, take up a fence piece or an incubator, open or
  shut a door. F also still boards a craft, and aboard it steps off.
- **E is freed on foot.** Flying, Q/E roll as before; aboard a boat, Q/E
  stay the boat's own (hiking, the blade or an oar as a rudder).
- The two rows of the old table, `hands:KeyE` ("use: ...") and
  `vehicles:KeyF` ("board, or step off"), become **one row in one slot**,
  `interact:KeyF`, in a new INTERACT group on the HANDS sub-tab, live on
  foot and aboard. One slot is the only honest shape: the remap
  (section 2, "how the game obeys it") makes every reader read a slot's
  default key, so two slots with the default F could never be told apart
  once one was moved; and one key that does one thing per press is what
  the owner asked for. Rebinding INTERACT moves use, board and step-off
  together, and the prompt shows the new key.

### One press, one thing: the priority table

`pbd_app::interact` holds one table, `ACTS`: each row is a thing F can act
on and the verb it is named by, **in priority order**. Every frame, the
systems that know a kind of thing (husbandry, fruit, the farm and its
stations, the town doors, the boats) offer what the eye is on and what F
would do to it; the highest row offered wins and is the frame's one
`Interact::now`. Then:
- **the prompt prints `now`** (its key, its verb, its thing), and
- **an F reader stands aside when `now` names another reader's row**, so
  F does what the prompt said: a crop beside a moored boat is harvested,
  not boarded. With nothing named (no offer was seen yet), each reader does
  what it did before and the first to use the press takes it, as the
  husbandry hands always have; so a press is never lost to a frame's lag.

| Rank | Thing aimed at | Prompt | Reader |
| --- | --- | --- | --- |
| 1 | a kept animal with eggs, milk or wool waiting | `F  Collect eggs` | husbandry |
| 2 | a hen, fertile eggs in hand | `F  Set eggs under hen` | husbandry |
| 3 | an animal, its food in hand (wild or kept) or a wild one to tame | `F  Feed hen` | husbandry |
| 4 | a kept animal following / standing | `F  Let hen stay` / `F  Lead hen` | husbandry |
| 5 | an incubator | `F  Set eggs in incubator` / `F  Take incubator` | husbandry |
| 6 | a fence or gate piece | `F  Take up fence` | husbandry |
| 7 | a fruiting tree with fruit hanging | `F  Pick apples` | fruit |
| 8 | a workbench, forge or campfire | `F  Open workbench` | farm |
| 9 | a ripe crop | `F  Harvest wheat` | farm |
| 10 | a growing crop | `F  Check wheat` (says how long) | farm |
| 11 | a wild plant | `F  Pick wild chili` | farm |
| 12 | a door in reach | `F  Open door` / `F  Shut door` | towns |
| 13 | tillable ground, shovel in hand | `F  Till grass` | farm |
| 14 | a craft in reach (nothing above it aimed) | `F  Board rowboat` | vehicles |

The ranks keep today's order where today had one (the husbandry hands ran
first and took a press they used, then the fruit, then the farm); the door,
which today could open in the same press as a crop, and boarding, which
today could happen in the same press as anything, are placed below them.
Boarding is last: a craft is boarded by walking up to it, so anything aimed
at is the more deliberate choice.

### Aiming at the plant itself

The eye ray stops at the first solid block, and a crop or a wild plant is
not a block: it stands in the air over its cell. Today `E` acts on the
cell the ray lands in, so aiming at a plant's leaves acted on the ground
behind it. A plant gets its own hit test, `farm::plant_on_ray`: an upright
cylinder per sown crop (radius 0.9 m, 1.1 m tall: the three clumps) and per
wild plant (0.55 m, 0.95 m: the one bush) within reach, the nearest hit
taken; a plant hit nearer than the block the ray meets wins over that
block. The ground cell under a plant still finds it, as before.

### The prompt

A small stone tooltip just under the crosshair: the bound key in an oak cap
(the settings page's own cap), then the verb and the thing in the kit's
font, e.g. `F  Harvest wheat`. Nothing else: no reason, no note. It shows
only on foot with the pointer held; it hides while a panel or menu is open,
and when nothing is offered. It does not flicker: a thing stays named for
0.15 s after the aim leaves it (`interact::HOLD_S`), so crossing the seam
between two cells of a field changes the name, never blinks it out. The
same hold applies to `now` itself, so the prompt and F never disagree. The
boats' on-foot line `[F] board the rowboat` is replaced by the prompt, and
aboard, the panel's keys line names the interact key for stepping off.
The farm's note (what the last press did) moves down from 56% to 60% of the
window, below the prompt, so the two never overlap.

### Saved rebinds (old settings files still load)

Settings files go to **version 8**. Reading any version, the rebinds of the
two retired slots migrate to the new one:
- `hands:KeyE = K` (a player who moved "use") becomes `interact:KeyF = K`:
  **a player who rebound use keeps their key**; only the default moved;
- otherwise `vehicles:KeyF = K` becomes `interact:KeyF = K`;
- if both were moved to different keys, use's wins and the board's is kept
  in the file, unread, as any unknown slot is (section 2's `kept`);
- a player who never rebound reads none, and F is interact.
The next write is version 8 with `interact:KeyF`. An older build reading a
version-8 file keeps the unknown slot and, as today, refuses to write a
newer file back.

### Text

Every text that names E for use now names F: the INTERACT row, the starter
missions' help lines (`missions.ron`), the farm and husbandry notes, the
showcase bot's presses, and the doc comments. Two help lines were wrong and
are corrected while there: the bucket fills with the right button, and a
boat is boarded with F. The husbandry line under the hotbar no longer says
which key to press (`E with turf`, `[E]`): the prompt says it, with the
player's own key; it says what the animal eats instead.

### Capture instruments

Two capture-only flags were added for the screenshots, as instruments
(CLAUDE.md, "Write it up before touching code"), and change nothing a player
runs: `--hold-pointer` makes the walker hold the pointer as a player's does
after a click (a still capture's window never has the focus, so without it
there is no aim and no prompt), and `--farm-ripe` sows `--farm`'s field
ripe, so the crop in front of the eye is one F harvests.

### Found while capturing (not changed)

The fruit's hit test (`digging::tree_on_ray`, which the axe and the fruit
have always used) is an upright cylinder on the tree's cell centre. In
`docs/screenshots/interact-f/prompt-pick-fruit.jpg` the prompt names the
plums while the crosshair is on the grass beside the drawn trunk: the hit
cylinder and the drawn trunk do not quite agree. This is not new (E picked
the plums there too); the prompt now shows it. Left for its own change.

## Tab closes what is open; an X on every stone panel (owner, 2026-11-22)

The owner, playtesting: "Tab should by default close out of menu if a menu
is open. If no menu is open then tab opens pack." And: "Crafting menu could
use a little x in the corner too. Maybe you can add a little x to all the
stone menus".

**Tab.**
- In the world, Tab opens the pack, as now.
- With any screen open over the world, Tab closes it to the world: the pack,
  a station (workbench, forge, kitchen), a chest, the missions, the map, the
  guide, breeding and the pause menu.
- On the title, settings and the worlds page Tab does nothing; those are
  left to Esc and their BACK buttons.
- I keeps its own toggle (pack, or pack from missions), and O, M, J and B are
  unchanged.

**The X.**
- Every stone panel's carved lintel (`menu::lintel`) gets a small oak
  plaque with an X in its top-right corner, the kit's plaque at 22 x 22 px.
- Pressing it does what Esc does on that screen (`MenuAction::Back`,
  `Screen::back`): a panel over the world closes to the world, settings and
  the worlds page step back as BACK does.
- One place builds it, so a new panel with a lintel has its X too.
- The X is a key in the locale file (`menu.close`), like every other word.
- On the pack and the craft panel the lintel sits on the panel's top band, so
  the X is at that band's corner.

Check: screenshots of the pack, the workbench, a chest, the forge, the
kitchen, the map, the pause menu and the worlds page, each with its X, in
`docs/screenshots/stone-and-wood-ui/`. The X's place is tuned from them.

## What a recipe takes, against what is carried (owner, 2026-11-22)

The owner: "When showing items for recipe you just show one number. Need to
show x/3 stone for example for making an axe".

An input's well shows how many the recipe takes ("3"), and the carried
count is only in its tooltip ("stone: need 3, have 1"). Change: every input
shows carried over needed, "1/3", on the hands' panel, the workbench, the
forge and the campfire. Outputs keep their plain count, shown only above one.
- The pack and workbench wells are repainted every frame already, so they
  take it there (`craft::paint`, `have_of_need`).
- The forge's and kitchen's lists were written once when spawned. Their input
  counts carry a `NeedCount(item, n)` and are repainted while the station is
  open (`craft::paint_need_counts`), from the same pack-and-hotbar count.
- The number is digits and a slash, no words, so it needs no locale key.
- A two-digit carried count ("12/3") may run past a 16 px well. Seen on the
  screenshots; if it crowds, the carried count is capped at "9+".

Check: screenshots of the stone axe with 0, 1 and 3 stone carried, and of the
forge, in `docs/screenshots/stone-and-wood-ui/`.
