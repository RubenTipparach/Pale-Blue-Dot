## ADDED Requirements

### Requirement: F picks up stones and shrubs

Pressing F on a pebble, bush, dead shrub or fern within reach SHALL give its
goods into the player's hands. A pebble SHALL give loose stone; a bush, dead
shrub or fern SHALL give fibre. No tool SHALL be needed.

#### Scenario: A stone at the feet

- **WHEN** the player aims at a pebble 2 m away and presses F
- **THEN** one loose stone is in their pack and the pebble dissolves out

### Requirement: A taken piece stays taken, then grows back

A taken piece SHALL be a saved record of the world, and SHALL NOT be drawn
until it grows back three game days later. It SHALL dissolve out and in,
never vanishing or appearing in one frame.

#### Scenario: Reopened

- **WHEN** the player takes a shrub, saves and reopens the world the same day
- **THEN** the shrub is not there

### Requirement: Pieces sit where the draw puts them

The CPU's placement of pickable clutter SHALL match the vertex shader's to
within a centimetre, pinned by a test of the shader's own output.

#### Scenario: Pinned

- **WHEN** a roll or salt in the shader's clutter changes without the port
- **THEN** the placement test fails
