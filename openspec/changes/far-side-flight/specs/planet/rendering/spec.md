# Rendering: level of detail accounts for the player's height

## MODIFIED Requirements

### Requirement: Level is quantised from distance to the player
A tile's level of detail SHALL be a function of its distance from the player,
quantised into bands of 2,400, 1,200, 600 and 300 m for levels 8 to 11.

- **Distance** is the slant distance, which counts the player's height above
  the ground. A band of radius `B` seen from height `h` SHALL cover the ground
  out to `sqrt(B^2 - h^2)`, and SHALL be empty when `h >= B`.
- **One table:** the band thresholds SHALL be computed once per frame from the
  player's position and published, and the GPU's level selection and the
  fine-set builder SHALL both read that one table.
- **Owner:** the level SHALL be decided by the level below's cell the tile
  belongs to (its owner), against one published player position. Midpoint
  cells with one fine owner SHALL be drawn and split per fragment along the
  owner boundary.
- **Not the camera:** the level SHALL NOT be anchored to the camera, so that
  looking around does not change any tile's level.

#### Scenario: The player stands still and looks around
- **WHEN** the camera turns or pulls back while the player does not move
- **THEN** no tile changes level

#### Scenario: Two adjacent tiles away from a threshold
- **WHEN** two neighbouring tiles are both well inside one band
- **THEN** both are assigned the same level, without consulting each other

#### Scenario: The player climbs
- **WHEN** the player is 300 m or more above the ground
- **THEN** no tile is drawn at level 11
- **AND** when the player is 2,400 m or more above the ground, no fine set is built at all
