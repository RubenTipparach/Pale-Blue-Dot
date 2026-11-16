# Interface look: rock and stone in play, carved oak out of play

Sections 1 and 2 of `tasks.md` are built (2026-11-14). The requirements they
made true moved to `openspec/specs/interface-look/spec.md` with the tests
that pin them: the UI scale, the smooth kit, 9-slice panels, the rock frames,
rock in play and oak out of play, the kit's font, one palette module and
legible text (section 1); then key bindings grouped by context and
changeable, the stone sliders and the slots' tooltips (section 2). What
follows is a recommendation not yet taken, written as the requirement it
would be.

## ADDED Requirements

### Requirement: The map's drawer opens on a plaque

The map's legend drawer SHALL be shut when the map opens, and SHALL open
and shut on a DRAWER plaque on the map's frame, the choice held for the
run. A recommendation from the mockup, written down and not built
(the brief: "leave it open by default"), because the game's drawer is open today and shutting it changes what
a player sees on opening the map.

#### Scenario: Opening the map

- **WHEN** the player opens the map the first time in a run
- **THEN** the drawer is shut and the DRAWER plaque is on the frame
- **WHEN** the player presses DRAWER
- **THEN** the drawer opens, and stays open the next time the map opens

### Requirement: F is the interact key and does one thing per press

On foot, one key, F by default, SHALL interact with what the player is
looking at: open a station, harvest a ripe crop, pick a wild plant or a
tree's fruit, till with the shovel, feed, lead and collect from an animal,
take up a piece, open or shut a door; with nothing of those aimed, it SHALL
board a craft in reach, and aboard it SHALL step off. One press SHALL do
exactly one thing, the highest of one priority table. E SHALL do nothing on
foot, and Q/E SHALL still roll in flight. A player's saved rebind of "use"
SHALL be kept as their interact key. The owner, 2026-11-16: "we should use F
for interact since E will be used for roll in space".

#### Scenario: Harvesting with F
- **WHEN** the player looks at a ripe crop's leaves and presses F
- **THEN** the crop is harvested into the pack and the plot is saved bare

#### Scenario: A rebind from before
- **WHEN** a version 7 settings file with `hands:KeyE: "KeyK"` is read
- **THEN** K is the interact key and F is not

### Requirement: A prompt names what F will do

While the player on foot looks at something F acts on, a prompt under the
crosshair SHALL show the bound key and the verb and the thing (`F  Harvest
wheat`), and nothing else; it SHALL name the action the next press performs,
SHALL hide when nothing is offered and while a panel or menu is open, and
SHALL NOT blink out as the aim crosses between cells. The owner, 2026-11-16:
"F to harvest should popup, or F to open work bench etc."

#### Scenario: Looking along a field
- **WHEN** the aim sweeps across a row of ripe wheat
- **THEN** the prompt reads `F  Harvest wheat` throughout, and goes when the
  aim leaves the field
