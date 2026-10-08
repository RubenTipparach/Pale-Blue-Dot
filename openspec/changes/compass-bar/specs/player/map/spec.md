# Player: map

## ADDED Requirements

### Requirement: The map is the planet the right way round

The map SHALL be drawn with compass north up and compass east to the right,
so that a coast on the map has the same shape and handedness as the same
coast seen from orbit with north up. The map's cursor readout SHALL give
compass latitude, and the player's arrow SHALL point along their compass
heading.

#### Scenario: A place east of the player

- **WHEN** a place lies a little compass-east of the player
- **THEN** it is drawn to the right of the player's arrow on the map

#### Scenario: A place north of the player

- **WHEN** a place lies a little compass-north of the player
- **THEN** it is drawn above the player's arrow on the map
