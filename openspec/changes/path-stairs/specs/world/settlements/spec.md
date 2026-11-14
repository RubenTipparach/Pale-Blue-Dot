## ADDED Requirements

### Requirement: A step on a town's path has a stair
Wherever two of a town's path cells (paved lanes, streets, squares, the quay,
a gate's passage) meet with one standing more than 0.35 m and at most 1.25 m
over the other, the town SHALL stand a stair against that riser, so that a
walker climbs from one to the other with no rise or drop of the eye over
0.1 m in any 0.1 m of its way and never leaves the ground. A cell where the
mockup cut street steps SHALL take street steps across the whole cell, and
any other riser a straight flight in the lower cell against it. Where a
pier's planks step up by such a riser, the pier SHALL stand a timber stair on
its lower planks against it.

#### Scenario: Every template's risers
- **WHEN** each template (village, walled town, harbour, desert, tundra
  camp, jungle village) is laid and cut
- **THEN** every riser between two of its path cells has a stair
- **AND** a walker climbs each one and comes down it with no eye jump over
  0.1 m and no tick in the air

#### Scenario: Just inside the north gate
- **WHEN** a walker comes through a walled town's north gate
- **THEN** stone street steps take it up the metre to the gate street

#### Scenario: Marenstrand's bank
- **WHEN** a walker goes down Marenstrand's main pier
- **THEN** it climbs onto the planks over the sand bank by a timber stair and
  comes down off them by another

### Requirement: Stairs are derived and yield to the player
A path stair SHALL be cut from the stored town and its template each time
the town is cut and SHALL never be saved; it SHALL change no record, level,
ground height or layout version. A stair SHALL NOT be placed on a riser
whose cells, or a pier stair's cells, hold a player's edit.

#### Scenario: A saved town
- **WHEN** a world saved before stairs is opened
- **THEN** its towns' records and ground are as they were
- **AND** each town is the same pieces with its stairs added

#### Scenario: The player has dug a step cell
- **WHEN** a town is cut and a riser's lower cell holds an edit
- **THEN** no stair stands on that riser
