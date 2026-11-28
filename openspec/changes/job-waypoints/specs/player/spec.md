## ADDED Requirements

### Requirement: A located job is marked

A taken, unfinished job that sends the player to a place SHALL mark that place on the compass bar and the map: a lost villager, an escort's trader, a parcel's town, a repair's broken wall and a rockfall's road.

#### Scenario: A rockfall job is marked

- **WHEN** the player has taken a rockfall job and not finished it
- **THEN** the compass bar shows a pin at the rockfall's bearing, named with its distance, and the map shows it

#### Scenario: A repair job is marked

- **WHEN** the player has taken a repair job and its town is loaded
- **THEN** the compass bar and the map show a pin at the broken wall
