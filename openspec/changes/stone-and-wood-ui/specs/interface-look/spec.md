# Interface look: stone and carved wood

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
against what it sits on, and every control a pointer or a finger presses
SHALL be at least 44 x 44 screen pixels at the default scale.

#### Scenario: A plaque's label

- **WHEN** a button is drawn idle, hovered or pressed
- **THEN** its label's contrast against the plaque is at least 4.5:1 and the
  plaque is at least 44 pixels tall
