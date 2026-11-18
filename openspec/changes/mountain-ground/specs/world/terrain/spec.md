## ADDED Requirements

### Requirement: Mountain flats are grassed or snowed in version 13

In generator version 13, a Mountains cell whose six neighbours stand within
one metre of it SHALL take snow above the ragged snow line and alpine grass
or dirt below it, and a steeper Mountains cell SHALL take stone. Worlds made
on an earlier version SHALL keep their ground.

#### Scenario: A flat bench on a range

- **WHEN** a version 13 world is generated and a Mountains cell below the
  snow line has every neighbour within one metre of its height
- **THEN** its top is grass or dirt, with soil under it

#### Scenario: An old world

- **WHEN** a world made on version 11 is opened
- **THEN** its ground is the same, cell for cell, as before
