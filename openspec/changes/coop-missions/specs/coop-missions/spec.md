## ADDED Requirements

### Requirement: Crew progress is the world's and counts each hand
A crew mission's progress SHALL be one record of the world, kept by the
server, counted from the changes it accepts by the same counting function
as a player's own missions, and SHALL record each author's share of every
objective.

#### Scenario: Two players dig toward one objective
- **WHEN** player 0 digs 3 cells and player 1 digs 4 cells toward a crew
  objective of 7
- **THEN** the objective is met, its shares read 3 and 4, and a reload of
  the world keeps both

### Requirement: An objective can need more than one pair of hands
An objective with `hands: n` SHALL be met only when at least `n` different
players have counted toward it, and one with `each: n` only when every
player present past the grace time has counted at least `n`.

#### Scenario: One player raises every wall
- **WHEN** one player places all 8 pieces of a `hands: 2` objective
- **THEN** the objective is not met and shows that another pair of hands is
  needed, until a second player places a piece of it

### Requirement: A step for two is one change with two authors
A `Together` objective SHALL be met only by two players' matching requests
on one target within `together_s`, settled by the server as one change
naming both; a lone request SHALL change nothing and be refused as `Alone`.
In single player it SHALL count as met.

#### Scenario: The ridge beam
- **WHEN** two players ask to set the same beam 1.2 s apart
- **THEN** the beam is set once, its record names both, and H5 is met

#### Scenario: Alone
- **WHEN** one player asks to set the beam and no second request follows in
  3 s
- **THEN** nothing is placed and the player is told it needs two
