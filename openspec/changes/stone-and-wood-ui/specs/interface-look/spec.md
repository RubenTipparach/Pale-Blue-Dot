# Interface look: stone, with carved wood as accent

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

### Requirement: Stone is the interface's material, with timber as accent

The interface's frames, plaques, tabs, slots, bars, the compass strip and every
other control SHALL be drawn in stone (dressed stone or rough rock). Carved oak
SHALL appear only as the accents the design names (the title lintel over a
menu, the open tab's cap, the compass pointer); iron and brass only as small
fittings.

#### Scenario: The pause menu

- **WHEN** the pause menu opens
- **THEN** its frame is the rough-rock slab, its buttons are stone plaques, and
  the only oak on screen is the lintel carrying its title

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
