# Crew Missions Specification

## Purpose
Missions a world's players meet together, kept once a world with each
hand's share (`coop-missions` design 1 to 3; `pbd_core::missions::crew`,
the `crew` section of `assets/config/missions.ron`, counted by
`pbd_server::Server` from the edits it accepts).

## Requirements

### Requirement: Crew progress is the world's and counts each hand
A crew mission's progress SHALL be one record of the world (`crew_missions`),
kept by the server, counted from the changes it accepts by the same counting
function as a player's own missions (`missions::count`), and SHALL record
each author's share of every objective. A world with no such record SHALL
load as a crew with nothing done.

#### Scenario: Two players dig toward one objective
- **WHEN** player 0 digs 3 cells and player 1 digs 4 cells toward a crew
  objective of 7
- **THEN** the objective is met, its shares read 3 and 4, and a reload of
  the world keeps both

#### Scenario: A world from before crew missions
- **WHEN** a world with no `crew_missions` record is opened
- **THEN** its crew has nothing done, and nothing else in it changes

### Requirement: An objective can need more than one pair of hands
An objective with `hands: n` SHALL be met only when at least `n` different
players have counted toward it, never asking more hands than the crew has
had, and one with `each: n` only when every player present past `grace_s`
has counted at least `n`. A player still owed SHALL count toward an
objective whose total is already reached.

#### Scenario: One player digs the whole site
- **WHEN** one player of a crew of two counts all 7 of a `hands: 2`
  objective
- **THEN** the objective is not met and shows that another pair of hands is
  needed, until the second player counts toward it

#### Scenario: A crew of one
- **WHEN** one player plays a world alone
- **THEN** they can finish the house chain by themselves

#### Scenario: A late joiner
- **WHEN** one player brings all 24 logs of an `each: 4` objective while a
  second player has been in the world under `grace_s`
- **THEN** it is met; were the second player in past `grace_s`, it would
  wait for their 4 logs
