## ADDED Requirements

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
