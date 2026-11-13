# World: construction

## Purpose

How things too big for a pack are made and put in place: the assembly bay's
crated items, the hex printer that prints a building layer by layer, and the
cranes that lift them. Proposed with the M22 mockup; nothing here is built.

## ADDED Requirements

### Requirement: Large items are crated records in the world

The assembly bay SHALL deliver an item too large for a pack as a crate in the
world, not as a pack stack. A crate SHALL be a stored record with a stable ID,
a mass and a footprint in whole cells. A crate SHALL be saved when it is made
and every time it moves, through the durable path.

#### Scenario: A mast is crated, not stacked

- **WHEN** the assembly bay finishes a mast
- **THEN** a crate holding the mast stands in the bay's out-cell with its
  mass and footprint, and the player's pack is unchanged

#### Scenario: A crate survives a reload where it was set down

- **WHEN** a crane sets a crate down and the world is closed and opened
- **THEN** the crate is in the same cells with the same ID

### Requirement: The hex printer prints a building one layer at a time

The hex printer SHALL print a blueprint's pieces onto the plot's cells: floors
in cells, walls on cell edges, and solid footing cells. It SHALL print one
1 m layer per pass, from the lowest layer up. Each finished layer SHALL be one
world edit through the durable path that names the printer as its author. A
print that stops part-way SHALL leave every finished layer in place.

#### Scenario: A storey is three passes

- **WHEN** the printer prints a one-storey blueprint
- **THEN** it makes three passes, and after each one the layer it finished is
  saved as an edit authored by the printer

#### Scenario: A stopped print keeps its finished layers

- **WHEN** the printer runs out of printstone after four layers
- **THEN** the four layers stand, and the fifth has no partial pieces

#### Scenario: A printed wall is a town piece

- **WHEN** a walker walks into a printed building
- **THEN** its walls stand on cell edges and its floors in cells, exactly as a
  town building's do

### Requirement: A crane lifts within its load chart

A tower crane SHALL lift at most its rated moment divided by the hook's
radius, and never more than its hook limit. A lift past the chart SHALL be
refused with the load, the radius and the limit. The cab SHALL be a seat with
slew, trolley and hoist controls, and the camera's mouse look SHALL stay raw.

#### Scenario: A load at the tip is refused

- **WHEN** a crane rated 120 t m with an 8 t hook limit is asked to lift 4 t
  at 40 m
- **THEN** the lift is refused with "4.0 t at 40 m: the chart allows 3.0 t"

#### Scenario: Close in, the hook limit holds

- **WHEN** the same crane lifts 8 t at 10 m
- **THEN** the lift is allowed, and 9 t at 10 m is refused by the hook limit
