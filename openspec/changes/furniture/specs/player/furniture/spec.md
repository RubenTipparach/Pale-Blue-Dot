# Player: furniture

## Purpose

Furniture the player makes at a carpenter's bench, places in a room, sits or
lies on, and keeps things in.

## ADDED Requirements

### Requirement: Furniture is made at the carpenter's bench only

Every piece of furniture SHALL be made at the carpenter's bench, a station
block made at the workbench, and its recipes SHALL be shown as their own
Furniture branch on the bench's panel. No piece SHALL be made by hand or at
the workbench.

#### Scenario: A chair at the bench

- **WHEN** the player opens a carpenter's bench carrying three planks
- **THEN** the Furniture branch lists the chair, and making it takes the
  planks and gives a chair

### Requirement: A piece is placed by the building rules

A piece of furniture SHALL be placed in build mode in a cell at a layer,
against one of the cell's sides or in its middle, and SHALL be refused, with
the rule named, when it would stand off the plot, on a pentagon, held up by
nothing, into a wall, across another piece, in a door's clear space, under
too low a floor, or on a stair.

#### Scenario: Not in a doorway

- **WHEN** the player aims a chest against the side of a cell inside a door
- **THEN** the ghost is red and names the doorway

### Requirement: The player sits and lies down

F on a chair, stool or bench SHALL sit the player at the seat's height, F on
a bed SHALL lie them down at the pillow, and a movement key, a jump or F
SHALL get them up. A place another body holds SHALL refuse them.

#### Scenario: An occupied seat

- **WHEN** a townsperson sits on a stool and the player presses F on it
- **THEN** the player stays standing and the note says someone sits there

### Requirement: A container keeps what is put in it

F on drawers, a closet or a chest SHALL open a panel of its slots beside the
carried slots, and a stack moved either way SHALL be saved with the slots
after it in one durable line, and SHALL be there after a quit and reload.

#### Scenario: Kept across a reload

- **WHEN** the player puts five planks in their chest, quits and loads
- **THEN** the chest holds five planks and the pack five fewer
