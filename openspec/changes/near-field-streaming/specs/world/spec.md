# World: the near field

## ADDED Requirements

### Requirement: The column tier follows the player continuously

The column tier SHALL be resident for every finest cell within `reach_m` of
the player, generated nearest first under a per-frame time budget, and SHALL
be a complete tier on every frame: every resident column is either carved
with all its neighbours resident, or generated solid as the rim.

#### Scenario: Walking never reaches the rim

- **WHEN** the player walks at sprint speed across fresh ground
- **THEN** on every frame every cell within `reach_m` of the player has a
  resident column
- **AND** no rim column lies within `reach_m` of the player

#### Scenario: A column is generated once

- **WHEN** the player walks out and back over the same ground within the
  unload distance
- **THEN** no column is generated a second time

### Requirement: An edit never waits for streaming

Digging or placing at a cell whose column is not resident SHALL generate that
column on the same frame and apply the edit to it.

#### Scenario: An edit past the tier's edge lands

- **WHEN** the player aims at a cell outside the resident tier and digs
- **THEN** the layer is taken, the material is given, and the column is
  resident afterwards
