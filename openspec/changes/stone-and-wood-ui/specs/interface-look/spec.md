# Interface look: rock and stone in play, carved oak out of play

## ADDED Requirements

### Requirement: UI art is pixel art at whole-number scales

Every image the interface draws (panel frames, plaques, slots, bars, the
compass strip, icons) SHALL be pixel art authored at native size, drawn at a
whole-number UI scale and sampled nearest. A UI image SHALL NOT be drawn at a
fractional scale or sampled linearly.

#### Scenario: A panel at the default scale

- **WHEN** a menu panel is drawn in a 1920 x 1080 window
- **THEN** the UI scale is a whole number and the frame's 14-pixel corner is
  drawn exactly 14 times that scale in screen pixels, with no blended texel

### Requirement: The UI scale follows the window height on the desktop

The game runs on the desktop only (owner, 2026-11-14: "phones cant run this
lol"). The interface SHALL take its UI scale from the window height as
max(1, floor(height / 450)), a whole number, and SHALL keep one layout at
every window size: no layout for a narrow or phone-width window.

#### Scenario: A taller window

- **WHEN** the window is resized from 1920 x 1080 to 2560 x 1440
- **THEN** the UI scale changes from 2 to 3, and every piece is redrawn at the
  new whole number with the same layout

### Requirement: Rock in play, carved oak out of play

The menus a player opens out of play (the main menu, pause, settings and the
saves) SHALL be drawn in the carved-oak set. Everything in play (the HUD, the
pack, the map, machines and stations, talk and trade, chests, the loading slab)
SHALL be drawn in rough rock and dressed stone, with carved oak only as the
accents the design names (the title lintel, the open tab's cap, the compass
pointer) and iron and brass only as small fittings. The owner, 2026-11-14:
"main menu I think should use the timber stuff, but the game I like the rock
stuff", "pause menu = timber too".

#### Scenario: Pausing, then opening the pack

- **WHEN** the player pauses
- **THEN** the pause menu's frame, plaques and tabs are carved oak
- **WHEN** the player resumes and opens the pack
- **THEN** its frame is rough rock, its slots dressed stone, and the only oak is
  the lintel carrying its title

### Requirement: A rock frame never shows a repeat

A rough-rock frame SHALL NOT tile its rim or the shadow its lip casts on the
face: each in-play panel's rock is baked at that panel's size, or from a baked
edge longer than the panel that is only cropped. The rock is the approved base
rock (owner, 2026-11-14, "Ok good continue with the mockups"): the first render's
field and light without its strata rings, and without an outline.

#### Scenario: A wide panel

- **WHEN** the chest panel (372 x 168 native px) is drawn
- **THEN** no part of its rim or lip shadow is a copy of another part of it

### Requirement: Key bindings are grouped by context

The settings' Input page SHALL group the bindings into tabs by context (on foot,
hands, flying, vehicles and ships, menus and view), one row per action with its
key and a way to change it, a key already used in the same tab flagged on the
row, and a reset for the tab. The owner, 2026-11-14: "organize the input menu
better with tabs or something instead of big uber list".

#### Scenario: Rebinding a key

- **WHEN** the player picks Change on "Dig / use tool" and presses E
- **THEN** the row shows E and notes that E is also Interact, and Reset tab puts
  the tab's keys back

### Requirement: Labels are drawn in the kit's pixel font

Titles, plaque and tab labels, section labels, compass letters and key caps
SHALL be drawn in the kit's own bitmap font at the integer UI scale, in which C,
G and S are open, E has a flat back, 0 and O differ, and 1 and I differ. The
owner, 2026-11-14, of "PACK" reading as "PAOK": "the c there is kinda weird".

#### Scenario: The pack's title

- **WHEN** the pack opens
- **THEN** its lintel reads PACK, each letter whole pixels at the UI scale

### Requirement: Panels are 9-slice pieces with finished corners

A panel, plaque, bar, tab or frame that changes size SHALL be drawn as a
9-slice image whose corners are fixed and whose sides and centre stretch or
tile, with its margins read from the kit's own data file rather than restated
in code.

#### Scenario: A taller settings page

- **WHEN** a binding is added and the settings panel grows
- **THEN** its corners are unchanged and only its sides and centre grow

### Requirement: One palette module for the interface

The interface's colours SHALL come from one module of tokens. No UI file SHALL
keep its own copy of a panel, edge or ink colour.

#### Scenario: Changing the panel face

- **WHEN** the face colour token changes
- **THEN** every menu, the pack, the guide and the loading screen change with
  it, and no other file is edited

### Requirement: Text on the interface is legible

Text a player reads on the interface SHALL have a contrast of at least 4.5:1
against what it sits on, in every state of the control that carries it.

#### Scenario: A plaque's label

- **WHEN** a button is drawn idle, hovered, pressed, focused or disabled
- **THEN** its label's contrast against the plaque in that state is at least
  4.5:1
