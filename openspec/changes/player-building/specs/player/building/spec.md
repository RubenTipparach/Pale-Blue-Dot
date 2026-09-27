# Player: building

## Purpose

How a player puts buildings together from the same pieces the planet's towns
are made of: a build mode, pieces that snap to the cut, placement checked by
the towns' own rules, fitted roofs, blueprints, and every change kept as a
world edit.

## ADDED Requirements

### Requirement: Build mode swaps the hotbar for the kit's pieces

On foot, one key SHALL open and close build mode. In build mode the hotbar
SHALL hold the chosen kit's pieces (walls with their openings, floors, stairs,
posts, roofs, decks and walkways, doors, furniture and lights), each with its
own icon. Leaving build mode SHALL restore the hotbar as it was.

#### Scenario: Entering build mode

- **WHEN** the player presses B on foot
- **THEN** the hotbar shows the current kit's pieces, and the held tool is put
  away

#### Scenario: Leaving build mode

- **WHEN** the player presses B again
- **THEN** the hotbar and the selected slot are as they were before

### Requirement: A piece snaps to the cut

A piece SHALL be placed only where the town's cut puts that kind of piece:
- a wall on a cell edge, between two layer lines;
- a floor in a cell, on a layer line;
- a stair on its own cells, from one layer line to another;
- a post at a cell corner;
- a door or window in a wall's opening.

The ghost SHALL show where the piece would go before it is placed.

#### Scenario: Aiming at a wall's place

- **WHEN** the player aims near a cell edge with a wall selected
- **THEN** the ghost stands on that edge, from the floor's layer line up one
  storey

#### Scenario: A storey up

- **WHEN** a floor is placed three layers above the ground floor
- **THEN** it is the next storey's floor, and walls placed on it stand on it

### Requirement: A placement is checked by the towns' own rules

A placement SHALL be refused when it would break any layout rule a town is
held to:
- headroom at every walkable point;
- roofs never overlap;
- every stair lands on a layer line;
- no building on a pentagon or a pentagon's neighbour;
- furniture stands clear of walls;
- no two faces in one plane.

A refused ghost SHALL be shown in red with the rule it breaks. The check SHALL
be the same function the town layouts are checked with.

#### Scenario: A low floor over a stair

- **WHEN** the player places a floor that would leave less than the walker's
  height over a stair below
- **THEN** the ghost is red and says there is no headroom over the stair, and
  nothing is placed

#### Scenario: One rule, two callers

- **WHEN** a player-built house and a town's house are checked
- **THEN** both are checked by the same function, with the same result for
  the same pieces

### Requirement: A roof is fitted over what it covers

The roof tool SHALL fit the kit's roof over an enclosed footprint the player
picks, the way a town's roofs are fitted. A roof SHALL NOT be placed tile by
tile.

#### Scenario: Roofing a house

- **WHEN** the player uses the roof tool on a house of two rows of four cells
  with walls all round
- **THEN** a gable roof is fitted over its rows, with its eaves, and it passes
  the roof-overlap check

### Requirement: A blueprint places a whole building

The player SHALL be able to choose any building from the settlement templates
and place it as a ghost, rotated by a multiple of 60°. The ghost SHALL be
checked as a whole. Its pieces SHALL be filled in one at a time, or all at
once where building is free.

#### Scenario: Placing a cottage

- **WHEN** the player places a cottage blueprint on flat ground and fills it
- **THEN** the cottage stands with its door, stair and roof, and can be walked
  into like a town's

### Requirement: What is built is kept, and can be taken back

Placing or removing a piece SHALL be a world edit that enters the durable
transaction path at once. A player SHALL be able to remove a piece they
placed. Removing a piece that something else stands on or hangs from SHALL
be refused, with the reason.

#### Scenario: Reloading

- **WHEN** the player builds a room and the world is reloaded
- **THEN** the room is there, piece for piece

#### Scenario: Taking out a floor under a table

- **WHEN** the player removes a floor a table stands on
- **THEN** the removal is refused and names the table
