## MODIFIED Requirements

### Requirement: The terrain generator is Tenebris's
The surface altitude SHALL be Tenebris's generator ported term for term, with
each noise field declared as either planet-scale, whose size is an angle and
does not change with the body, or land-scale, whose size is a number of metres
and whose unit-sphere scale is derived from the body's radius. A land-scale
term SHALL carry its amplitude in metres rather than as a share of the relief
budget, so the summit and the roughness underfoot move independently. The
mountain ranges SHALL rise in regions of their own, so raising them does not
raise the lowland.

#### Scenario: Mountains stand on land
- **WHEN** a cell is above the mountain elevation
- **THEN** no cell within three cells of it is below sea level

#### Scenario: Rivers reach the sea
- **WHEN** a cell is a river channel
- **THEN** a downhill walk from it ends below sea level

#### Scenario: The budget holds
- **WHEN** the sphere is sampled
- **THEN** the summit is in the band the owner chose (survey H1; provisionally
  260 to 340 m), the floor between 60 and 145 m down, and land is between 30
  and 45 percent of it

#### Scenario: The lowland stays where it was
- **WHEN** the land is sampled on the version that raises the ranges and on
  the version before it
- **THEN** the median land altitude differs by less than 5 m

#### Scenario: The ground is as rough as the reference's
- **WHEN** adjacent land cells are compared over the sphere
- **THEN** at least a third of them differ by a block or more
- **AND** the finest feature in the height field is under twenty metres, which
  is under seven cells

## ADDED Requirements

### Requirement: Tall ground can be walked
Ground above the lowland SHALL rise in steps a walker can climb, not in
walls. The share of steep steps is the owner's choice (survey H3).

#### Scenario: A mountainside is a slope, not a cliff
- **WHEN** adjacent cells are compared over land above 120 m
- **THEN** no more than 1% of them differ by 3 m or more (provisional, survey
  H3)
