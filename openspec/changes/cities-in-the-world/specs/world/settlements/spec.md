## ADDED Requirements

### Requirement: A settlement stands at every site

Every site in a world's site list SHALL have its kind's settlement built on
it when it comes within range. The settlement's layout SHALL be charted onto
the real cells around the site's anchor through the neighbour tables, and
every piece SHALL be cut from its own cell's corners.

#### Scenario: Arriving at a site

- **WHEN** the player walks to a village site from the map
- **THEN** the village is there, and every house in it can be entered

#### Scenario: The chart is the sphere's

- **WHEN** a settlement's cells are listed
- **THEN** each is a real level-11 cell, and two cells neighbouring in the
  layout are neighbours on the sphere

### Requirement: A settlement is fitted to its ground

Inside a settlement's footprint, the ground SHALL stand at the layout's
terrace layers. Where the layout has a hollow (a cave town's chamber, tunnel
and rooms, a cliff village's rock-cut rooms), the column SHALL be empty
between the hollow's floor and roof, and solid above and below it. Between the footprint and the natural ground, a margin SHALL
ease the surface so no step up or down is more than one layer from cell to
cell. The trees and ground clutter inside the footprint SHALL be cleared,
except the layout's own plants.

#### Scenario: Walking in from the fields

- **WHEN** the walker crosses the margin into a town from any side
- **THEN** no cell-to-cell step on the way is more than one layer

#### Scenario: No tree in the market

- **WHEN** the cells of a town's streets and plots are checked
- **THEN** none carries a generated tree or a clutter blade

#### Scenario: A cave town's chamber

- **WHEN** the columns under a cave town's chamber are generated
- **THEN** each is solid below the chamber's floor, empty up to its roof, and
  solid from the roof to the mountain's own surface, except the shaft's one
  cell, which is open to the sky
- **AND** the chamber is read from the town's stored record, with no edit
  written for it

### Requirement: Two settlements of a kind are different towns

Each settlement SHALL vary its template by its site's seed:
- a rotation by a multiple of 60°;
- a mirror;
- each building's kit, drawn from its biome's kits;
- some plots left empty.

The variation SHALL keep every layout rule: headroom, roof overlap, stair
landings and furniture clearance.

#### Scenario: Two villages

- **WHEN** two village sites are built
- **THEN** their layouts differ in at least their rotation, mirror or kits,
  and both pass every layout check

### Requirement: A settlement never overwrites the player's work

A site whose footprint or margin holds a player edit made before its
settlement record SHALL NOT be built. The map SHALL mark it as unsettled.

#### Scenario: A player dug there first

- **WHEN** a world has a player edit inside a site's footprint from before its
  settlement record was made
- **THEN** no settlement is built on that site, and the edit is as the player
  left it

### Requirement: A settlement is seen from afar, and at night its lights are

Beyond the range where its pieces are drawn, a settlement SHALL be drawn in a
reduced form that keeps its outline and roofs. At night its lit windows and
lanterns SHALL show as points of light at any distance at which the planet's
surface is drawn, orbit included.

#### Scenario: From the next hill

- **WHEN** a town is a kilometre away by day
- **THEN** its walls and roofs are drawn

#### Scenario: From orbit at night

- **WHEN** the night side is seen from orbit
- **THEN** every settlement on it shows as a cluster of lights, and the coarse
  hexes of ground under it are lit warm by them

### Requirement: A settlement fades in and out

A settlement's pieces SHALL dither-fade between their near and far forms, as
the terrain's detail does. No settlement or piece SHALL appear or vanish in
one frame.

#### Scenario: Flying toward a town

- **WHEN** a ship flies toward a town at cruise speed
- **THEN** no frame shows a piece that was absent in the frame before at full
  opacity

### Requirement: A settlement is a stored record

Every settlement SHALL be generated with its buildings when the world is made,
and written to the save before it is shown. Each building SHALL be stored as
its own definition (plot, walls and openings, stairs, storeys, kit, roof and
state), not as a reference to a template, and its pieces SHALL be derived from
that definition. Every later change to a settlement, by the player or by the
world, SHALL be an authored journal entry. A building whose state is abandoned
SHALL give no light from its windows or hearth.

#### Scenario: Templates change

- **WHEN** the shipped templates are revised after a world was made
- **THEN** that world's towns are exactly as their records were stored, and a
  new world's towns use the revised templates

#### Scenario: A house is abandoned

- **WHEN** an authored entry sets a building's state to abandoned
- **THEN** after dusk its windows and hearth are dark, and a reload keeps it so

### Requirement: Everything in the approved towns mockup is in the game

Every feature listed in `docs/plans/towns-mockup-parity.md` SHALL be built by
the change its row names, or SHALL be marked "mockup only" with the owner's
agreement. The settlements step SHALL NOT be accepted while any row is still
planned or waiting on an answer.

#### Scenario: The settlements gate

- **WHEN** the owner reviews the settlements step
- **THEN** every row of the parity list is built or agreed mockup-only, and each
  built row has a video shot beside the same shot in the prototype
