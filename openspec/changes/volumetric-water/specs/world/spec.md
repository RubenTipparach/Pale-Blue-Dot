# World: water

## ADDED Requirements

### Requirement: Water is read from the column where a column exists

Where a cell has a column, whether a point is under water SHALL be answered
by that column's layer at the point, and only outside the column tier by the
height field.

#### Scenario: A dry cave below sea level is dry

- **WHEN** a cave is carved under land below sea level and nothing floods it
- **THEN** its faces draw without the submerged term
- **AND** a camera inside it is not submerged
- **AND** a walker inside it does not swim

#### Scenario: The sea is the sea

- **WHEN** a point is in a water layer of a column under the sea
- **THEN** its faces, the camera and the walker read it as under water,
  exactly as the height field reads it today
