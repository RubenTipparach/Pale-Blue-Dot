## ADDED Requirements

### Requirement: The showcase bot plans its walks over cells and layers, digging included
The showcase bot SHALL plan a walk as a sequence of cell-and-layer steps over the live ground, each step naming the blocks it digs first, where a step up is at most one block, a drop at most three, stations and lamps are never dug and water too deep to wade is not entered on a dry walk.

#### Scenario: Out of a pit
- **WHEN** the bot stands in a pit two blocks deep and its goal is outside
- **THEN** the plan cuts a stair up the wall and the bot climbs out on it

#### Scenario: Through a wall
- **WHEN** a stone wall stands between the bot and its goal and going round is longer than digging
- **THEN** the plan tunnels through, and the bot digs it with a pickaxe

#### Scenario: Round a station
- **WHEN** a workbench stands in the straight line
- **THEN** the plan goes round it and digs nothing of it

### Requirement: The bot follows the plan and replans when a step fails
The bot SHALL dig each step's blocks with the tool for their material, aiming at each, then walk in, and SHALL plan again from where it stands when a step is not reached in 6 s or a block does not break in 8 s.

#### Scenario: A step not reached
- **WHEN** the walker is held short of a step for 6 s
- **THEN** a new plan is made from its position
