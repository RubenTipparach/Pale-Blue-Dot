# Player: missions

## ADDED Requirements

### Requirement: Every objective is counted where the game acts

Every objective in the shipped mission tree SHALL be raised by the system
that carries out the act in play, not only by a test's hand. A good that
comes out of a dug block SHALL count as gathered when the block is cut.

#### Scenario: Coal from an ore seam

- **WHEN** "Find ore" is open and the player digs an ore cell that gives coal
- **THEN** the coal objective counts one

### Requirement: A place reached can be reached again

A place objective (a reef, a town) SHALL count each time the player comes
onto the place, so a place crossed before its mission opened can still meet
it.

#### Scenario: A reef swum over early

- **WHEN** the player swims over a reef while "Into the water" is open, and again after "See a reef" opens
- **THEN** "See a reef" is done
