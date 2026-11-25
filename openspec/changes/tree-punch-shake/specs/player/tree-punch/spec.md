## ADDED Requirements

### Requirement: Bare hands take logs from a tree

A player with bare hands SHALL take logs from a tree as an axe does, up to the
logs the tree gives, each taking longer than with an axe.

#### Scenario: Punching a tree

- **WHEN** a player with no tool holds the use button on a tree
- **THEN** a log comes every few seconds until the tree has given all its wood and falls

### Requirement: A tree shakes when it is hit

A tree being chopped or punched SHALL sway from each hit and settle when the
hits stop.

#### Scenario: Chopping

- **WHEN** a player works a tree with an axe or bare hands
- **THEN** the tree's crown sways with each hit and comes to rest after the last
