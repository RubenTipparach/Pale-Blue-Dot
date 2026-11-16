## ADDED Requirements

### Requirement: Mountain flats are grassed or snowed in version 11

In generator version 11, a Mountains cell whose six neighbours stand within
one metre of it SHALL take snow above the ragged snow line and alpine grass
or dirt below it, and a steeper Mountains cell SHALL take stone. Worlds made
on an earlier version SHALL keep their ground.

#### Scenario: A flat bench on a range

- **WHEN** a version 11 world is generated and a Mountains cell below the
  snow line has every neighbour within one metre of its height
- **THEN** its top is grass or dirt, with soil under it

#### Scenario: An old world

- **WHEN** a world made on version 10 is opened
- **THEN** its ground is the same, cell for cell, as before
