# Cell Addresses Specification

## ADDED Requirements

### Requirement: A cell's key is its region's address and its place in it
The game SHALL key every cell by the pair of its region, a triangle 64 cells on
a side of its face's lattice taken after the face is made canonical, and its
place in that region, two 6-bit coordinates. A region SHALL be the same 64
cells on a side on every body.

#### Scenario: The same region size on two bodies
- **WHEN** a region is taken on a 600 m body and on the 4,800 m home world
- **THEN** each spans 64 cells on a side, about 180 m across

### Requirement: Every rung from 300 m to 307 km has keys
The region address SHALL carry levels 7 to 17 at least, so that an edit on a
body of any radius on the ladder from 300 m to 307,200 m has a key.

#### Scenario: An edit on a 300 m body
- **WHEN** a player digs a cell on a 300 m body (L7) and the world is saved and reopened
- **THEN** the dug cell is still dug

### Requirement: Old keys move to the new form without loss
Reading a save written with the 32-bit key SHALL turn each key into its region
and place exactly, and SHALL keep the old reader so such a save always opens.

#### Scenario: A save from before the change
- **WHEN** a home-world save written with 32-bit keys is opened
- **THEN** every edit is at the same cell as before, and the save is written back in the new form
