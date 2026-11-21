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

- **WHEN** the player presses N on foot
- **THEN** the hotbar shows the current kit's pieces, and the held tool is put
  away

#### Scenario: Leaving build mode

- **WHEN** the player presses N again
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

### Requirement: A blueprint places a whole building

The player SHALL be able to choose any building from the settlement templates
and place it as a ghost, rotated by a multiple of 60°. The ghost SHALL be
checked as a whole. Its pieces SHALL be filled in one at a time, or all at
once where building is free.

#### Scenario: Placing a cottage

- **WHEN** the player places a cottage blueprint on flat ground and fills it
- **THEN** the cottage stands with its door, stair and roof, and can be walked
  into like a town's

