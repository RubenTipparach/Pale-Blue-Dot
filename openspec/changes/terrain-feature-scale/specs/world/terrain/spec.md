## MODIFIED Requirements

### Requirement: The terrain generator is Tenebris's
The surface altitude SHALL be Tenebris's generator ported term for term, with
each noise field declared as either planet-scale, whose size is an angle and
does not change with the body, or land-scale, whose size is a number of metres
and whose unit-sphere scale is derived from the body's radius. A land-scale
term SHALL carry its amplitude in metres rather than as a share of the relief
budget, so the summit and the roughness underfoot move independently.

#### Scenario: Mountains stand on land
- **WHEN** a cell is above the mountain elevation
- **THEN** no cell within three cells of it is below sea level

#### Scenario: Rivers reach the sea
- **WHEN** a cell is a river channel
- **THEN** a downhill walk from it ends below sea level

#### Scenario: The budget holds
- **WHEN** the sphere is sampled
- **THEN** the summit is between 130 and 180 m, the floor between 60 and
  110 m down, and land is between 40 and 60 percent of it

#### Scenario: The ground is as rough as the reference's
- **WHEN** adjacent land cells are compared over the sphere
- **THEN** at least a third of them differ by a block or more
- **AND** the finest feature in the height field is under twenty metres, which
  is under seven cells
