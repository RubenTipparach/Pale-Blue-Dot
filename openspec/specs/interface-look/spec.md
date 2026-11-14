# Interface Look Specification

## Purpose
The game's interface is one kit, drawn the way the owner chose on 2026-11-14:
the smooth Blender renders of rough rock and dressed stone in play and of
carved oak out of play ("Smooth is the best ... start working the current UI
as the wood + rendered smooth rock"), the kit's own pixel font for labels,
VT323 for body text, one palette module, and a whole-number UI scale from the
window height. Built by `openspec/changes/stone-and-wood-ui` (section 1);
`crates/pbd-app/src/ui_kit.rs` holds the kit.

## Requirements

### Requirement: The UI scale follows the window height on the desktop
The game runs on the desktop only (owner, 2026-11-14: "phones cant run this
lol"). The interface SHALL take its UI scale from the window's physical
height as max(1, floor(height / 450)), a whole number of screen pixels for
each native pixel, and SHALL keep one layout at every window size.

#### Scenario: A taller window
- **WHEN** the window is resized from 1920 x 1080 to 2560 x 1440
- **THEN** the UI scale changes from 2 to 3, and every piece is baked again
  for the new whole number with the same layout

Pinned by `ui_kit::tests::the_ui_scale_is_a_whole_number_from_the_window_height`.

### Requirement: The kit is the smooth render, drawn at the whole-number scale
The kit's pieces SHALL be the smooth Cycles renders sampled linear (the
owner's exception, for the interface's kit only), each resampled to exactly
the UI scale's screen pixels a native pixel so it is drawn about one texel a
pixel. The item icons inside a slot and the kit's pixel font SHALL stay pixel
art sampled nearest, and no world texture SHALL change its sampling.

#### Scenario: A panel at the default scale
- **WHEN** a menu panel is drawn in a 1920 x 1080 window
- **THEN** the UI scale is 2 and the frame's 14-pixel corner is drawn 28
  screen pixels, sampled linear from a texture baked at 2 texels a native
  pixel

Pinned by `ui_kit::tests::the_kit_bakes_linear_and_the_font_nearest`.

### Requirement: Panels are 9-slice pieces with finished corners
A panel, plaque, bar, tab or frame that changes size SHALL be drawn as a
9-slice image whose corners stay their native size at every node size and
whose sides and centre stretch, with its margins read from the kit's own data
file (`assets/ui/slices.json`) rather than restated in code. No two opposite
margins SHALL add up to more than the piece.

#### Scenario: A taller settings page
- **WHEN** a binding is added and the settings panel grows
- **THEN** its corners are unchanged and only its sides and centre grow

Pinned by `ui_kit::tests::slice_margins_never_exceed_a_piece` and
`ui_kit::tests::the_kit_bakes_linear_and_the_font_nearest`.

### Requirement: A rock frame never shows a repeat
A rough-rock frame SHALL NOT tile its rim or the shadow its lip casts on the
face: each in-play rock panel is the approved base rock baked at that panel's
own size and drawn stretched to it.

#### Scenario: The pack
- **WHEN** the pack (268 x 152 native px) opens
- **THEN** its rock is the render made at 268 x 152, drawn whole, and no part
  of its rim or lip shadow is a copy of another part of it

Pinned by `ui_kit::tests::slice_margins_never_exceed_a_piece` (each rock
panel's file is the render at its panel's size) and
`pack::tests::the_pack_is_framed_in_its_own_rock`.

### Requirement: Rock in play, carved oak out of play
The menus a player opens out of play (pause, settings and the saves) SHALL be
drawn in the carved-oak set; the pack, the map, the HUD and the loading slab
SHALL be drawn in rough rock and dressed stone, with carved oak in play only as
the title lintel, the open tab's cap and the compass pointer. The owner,
2026-11-14: "main menu I think should use the timber stuff, but the game I
like the rock stuff", "pause menu = timber too".

#### Scenario: Pausing, then opening the pack
- **WHEN** the player pauses
- **THEN** the pause menu's frame and plaques are carved oak
- **WHEN** the player resumes and opens the pack
- **THEN** its frame is rough rock, its slots dressed stone, and the only oak
  is the lintel carrying its title

Pinned by `menu::tests::the_timber_menus_are_oak` and
`pack::tests::the_pack_is_framed_in_its_own_rock`.

### Requirement: Labels are drawn in the kit's pixel font
Titles, plaque and tab labels, section labels, compass letters and key caps
SHALL be drawn in the kit's own bitmap font at the UI scale, in which C, G and
S are open, E has a flat back, 0 and O differ, and 1 and I differ. Every
letter and digit SHALL have a glyph.

#### Scenario: The pack's title
- **WHEN** the pack opens
- **THEN** its lintel reads PACK, each letter whole pixels at the UI scale

Pinned by `ui_kit::tests::the_kit_font_has_every_label_glyph`.

### Requirement: One palette module for the interface
The interface's colours SHALL come from one module of tokens (`ui_kit`). No UI
file SHALL keep its own copy of a panel, edge or ink colour.

#### Scenario: Changing the panel face
- **WHEN** a token changes
- **THEN** every menu, the pack, the guide and the loading screen change with
  it, and no other file is edited

Pinned by `ui_kit::tests::no_ui_file_keeps_its_own_palette`.

### Requirement: Text on the interface is legible
Every text and label token SHALL have a contrast of at least 4.5:1 against
the face it is drawn on, in every state of the control that carries it.

#### Scenario: A plaque's label
- **WHEN** a stone plaque is drawn idle, hovered, pressed or disabled
- **THEN** its label's contrast against the plaque in that state is at least
  4.5:1

Pinned by `ui_kit::tests::tokens_clear_four_and_a_half_to_one`.
