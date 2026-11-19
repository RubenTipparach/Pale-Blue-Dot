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

### Requirement: Mountains stand trees below a tree line in version 13

In generator version 13, a Mountains cell of grass or soil SHALL stand a tree
by a density that falls with altitude to none at the tree line: firs below
it and krummholz about it. No Mountains cell SHALL stand a tree in a world of
an earlier version.

#### Scenario: Above the line

- **WHEN** a version 13 Mountains cell stands above the tree line's end
- **THEN** no tree stands on it, on the CPU or in the visibility pass

#### Scenario: An old range

- **WHEN** a world made on version 12 is opened
- **THEN** its mountains stand no trees, as before

### Requirement: Ridges above the snow line are scoured bare in version 13

In generator version 13, a Mountains cell above the snow line that stands
above at least five of its six neighbours SHALL take stone, not snow.

#### Scenario: A crest

- **WHEN** a version 13 cell above the snow line is higher than five
  neighbours
- **THEN** its top is stone
