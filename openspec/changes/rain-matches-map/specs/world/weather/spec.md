# Weather: the rain on the map is the rain you stand in

## ADDED Requirements

### Requirement: The map shows the rain the player stands in

The world map's rain (its clouds-and-rain layer and the RAIN overlay, on the
map and on the globe) SHALL draw the same field the player's rain reads: the
rain seen (`Atmosphere::rain_seen`), negative where it snows, through one
function in `pbd_core::weather`. Neither SHALL draw the simulation's rain
rate.

#### Scenario: Between two bursts of rain

- **WHEN** a place's rain rate falls under the raining threshold between two
  bursts and the rain seen there is still letting go
- **THEN** the player there stands in rain and the map shows rain there

#### Scenario: The same place on the map and on the ground

- **WHEN** the map's rain and the player's rain are read at the same place
  from the same weather state
- **THEN** they agree within a byte of the map's texel
