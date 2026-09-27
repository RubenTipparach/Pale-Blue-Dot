# Settlements Specification

## ADDED Requirements

### Requirement: Buildings are cut to the cell
A building SHALL be made of pieces placed on the cell grid at the gold-standard
cell size: walls on cell edges, floors in cells, and storeys three layers tall,
except a hut, which is one storey of two layers with nothing over it but its
roof.
A shared edge SHALL carry at most one wall, owned by one of its two cells.

#### Scenario: A one-cell room
- **WHEN** a cell has a timber wall on each of its six edges
- **THEN** the room inside is 2.5 m across between the wall faces
- **AND** its floor-to-ceiling height under the next storey's boards is 2.8 m

#### Scenario: Two houses that touch
- **WHEN** two buildings occupy neighbouring cells
- **THEN** the edge between them carries one party wall, not two

### Requirement: Every stair lands on a layer line
A straight flight, a newel stair and street steps SHALL each start and finish
at whole layers, so the floor at either end of a stair is a height the grid
can hold.

#### Scenario: A straight flight
- **WHEN** a straight flight is placed over two cells in a row
- **THEN** it climbs one storey, 3 m, in 16 risers of 0.1875 m on 0.354 m treads
- **AND** the cells before its foot and after its landing are floor at its two ends

#### Scenario: Street steps
- **WHEN** a street crosses a 1 m terrace
- **THEN** one cell of steps climbs it in 6 risers of 0.167 m

### Requirement: A newel stair's doorways sit on edges
A newel stair SHALL turn one full turn per storey in one cell, so each layer of
rise turns it two edges, and every doorway at a whole layer SHALL be centred on
an edge given by `(entry + 2 * layers) mod 6`.

#### Scenario: A wall tower
- **WHEN** a tower's stair climbs 7 layers from its street door to the wall walk
- **THEN** the wall-walk door is on the edge two round from the street door

#### Scenario: A layout that cannot work
- **WHEN** a stair would need its exit on the same edge as a wall it abuts
- **THEN** the layout is rejected rather than built with a door into masonry

### Requirement: Headroom holds at every walkable point
Every point a walker can stand on inside a settlement SHALL have at least a
body height and the contact skin of clear space above it, including under
roof overhangs, stair turns and landings.

#### Scenario: An eave beside a tower
- **WHEN** a roof overhang would reach over a neighbouring cell whose walkable
  surface is within a body height of the eave
- **THEN** the layout is rejected or the overhang is cut back

#### Scenario: The top of a newel stair
- **WHEN** a walker climbs the last turn under the top landing
- **THEN** the headroom check passes at every footprint point

### Requirement: A kit changes the materials, not the cut
A building's kit SHALL set its wall faces per storey, corner posts, roof, gable
and floor, and SHALL NOT change where its walls, floors, doors or stairs go.

#### Scenario: The same house in two kits
- **WHEN** one footprint is built as red brick and as timber
- **THEN** both have the same walls on the same edges, the same doors and the
  same stair
- **AND** only their faces, posts and roof differ

#### Scenario: A hut
- **WHEN** a straw hut is built on one cell
- **THEN** its door is at least 1.9 m tall and a walker can walk in through it

### Requirement: Roofs never overlap
No roof's plan, eaves included, SHALL overlap another roof's or a tower's or
the keep's.

#### Scenario: Two houses in touching columns
- **WHEN** two two-row houses would stand in neighbouring columns
- **THEN** the layout is rejected, because their plans interlock by half a cell

#### Scenario: A town laid out
- **WHEN** a town is laid out
- **THEN** every pair of roofs is checked and none overlap

### Requirement: A settlement is built from its biome's materials
A settlement's kits and plants SHALL come from its biome's catalogue entry, and
SHALL NOT borrow another biome's.

#### Scenario: A desert town
- **WHEN** a settlement is laid out in the desert biome
- **THEN** its buildings are of sandstone, mud brick and salt plaster
- **AND** its plants are the desert's cactus

#### Scenario: A harbour where the beach meets the fields
- **WHEN** a settlement is laid out across the beach and the fields behind it
- **THEN** its buildings are of driftwood and shell-lime whitewash, and of the
  fields' fieldstone and timber
- **AND** its plants are beach grass on the dunes and the fields' trees, and
  nothing comes from a third biome

### Requirement: Boats are obstacles and a moored ship is boarded like a building
A moored boat SHALL be a solid hull up to its gunwale that the walker goes
round, and its collision SHALL NOT follow the few centimetres it bobs on the
swell. A moored ship's deck, castles and stairs SHALL answer the stand query as
floors, and its rail SHALL block the walker everywhere but a gangway, where a
gangplank from the pier meets the deck with no step over 0.1 m.

#### Scenario: Up the gangplank
- **WHEN** the walker walks up the gangplank from a pier at 1.0 m
- **THEN** it stands on the ship's deck at 1.9 m
- **AND** no tick is airborne and no eye jump is over 0.1 m

#### Scenario: Wading at a moored rowboat
- **WHEN** the walker wades into a moored rowboat from the sea
- **THEN** it stops at the boat's side

### Requirement: Decks, walkways and outdoor stairs collide like floors
A raised deck, a walkway between two points (a rope bridge, a boardwalk, a
jetty, a pier, a gangplank) and an outdoor stair SHALL answer the stand query
with the top of their planks, and their rails SHALL block the walker.

#### Scenario: Across a rope bridge
- **WHEN** the walker crosses a rope bridge with 0.8 m of sag between two
  platforms at 9 m
- **THEN** its feet follow the planks down to 8.2 m and up again
- **AND** it is grounded on every tick

#### Scenario: Up a porch stair
- **WHEN** the walker climbs a stilt house's porch stair from the boardwalk
- **THEN** it reaches the deck with no eye jump over 0.1 m

### Requirement: A settlement is lit at night
Every settlement SHALL carry its own lights: hearths, and the lamps, torches or
braziers of its biome. At night every light SHALL light the surfaces within its
reach wherever the camera is, not only the lights nearest the camera, and a
light SHALL NOT light through a wall or a floor into another room.

#### Scenario: A town from above at night
- **WHEN** a settlement is seen from above at 22:30
- **THEN** every lamp lights the street round it, however far it is from the
  camera

#### Scenario: A town's streets at night
- **WHEN** night falls on a town
- **THEN** its streets are lit along their length, the more densely the
  bigger the town
- **AND** some of its windows glow and the rest are dark, and the room behind
  a lit window is lit

#### Scenario: A rope bridge at night
- **WHEN** the walker crosses a jungle rope bridge at night
- **THEN** small lanterns along its rails light the planks

#### Scenario: A hearth behind a wall
- **WHEN** a house's hearth burns
- **THEN** its light reaches the room's floor, walls and ceiling
- **AND** not the outside of the house's walls or the storey above

### Requirement: No two pieces draw in one plane
Two visible faces of different pieces SHALL NOT lie in one plane over the same
area. Trim, such as a sill, a threshold, or a floor over a wall top, SHALL stand
proud of the faces it meets.

#### Scenario: A window sill
- **WHEN** a window is cut in a wall
- **THEN** its sill's top stands above the wall top under the window, and the
  two never flicker against each other

### Requirement: Furniture stands clear of the walls
No piece of furniture SHALL overlap a wall, a corner post, a door or a hearth.
A piece placed where it would SHALL be moved clear by the least move that
frees it.

#### Scenario: A chest in a corner
- **WHEN** a chest is placed near a corner of a hex room, where the angled wall
  would cut through it
- **THEN** it stands against the wall, not in it

### Requirement: Doors are world state
Opening or closing a door SHALL be a world mutation that enters the durable
transaction path when it happens. A closed door SHALL block the walker and an
open one SHALL NOT.

#### Scenario: Reloading a town
- **WHEN** the player opens a door and the world is reloaded
- **THEN** the door is open

### Requirement: No building on a pentagon
A building SHALL NOT occupy a pentagonal cell or a cell with a pentagonal
neighbour, and its pieces SHALL be built from each cell's own corners.

#### Scenario: Placing a town near a pentagon
- **WHEN** a town layout covers one of the twelve pentagons
- **THEN** no building stands on that cell or its neighbours
