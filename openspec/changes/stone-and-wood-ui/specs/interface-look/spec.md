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
