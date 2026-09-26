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

### Requirement: Decks, walkways and outdoor stairs collide like floors
A raised deck, a walkway between two points (a rope bridge, a boardwalk, a
jetty) and an outdoor stair SHALL answer the stand query with the top of their
planks, and their rails SHALL block the walker.

#### Scenario: Across a rope bridge
- **WHEN** the walker crosses a rope bridge with 0.8 m of sag between two
  platforms at 9 m
- **THEN** its feet follow the planks down to 8.2 m and up again
- **AND** it is grounded on every tick

#### Scenario: Up a porch stair
- **WHEN** the walker climbs a stilt house's porch stair from the boardwalk
- **THEN** it reaches the deck with no eye jump over 0.1 m

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
