# World: town missions

## Purpose

Jobs a town offers on its board and through its people. The board, taking,
counting and paying are built and in `openspec/specs/world/town-missions`
(2026-11-21); the special jobs that need the townsfolk and the world's own
changes in the game are left here.

## ADDED Requirements

### Requirement: Jobs of many kinds

The jobs SHALL include trading, gathering or hunting for the town,
finding a lost villager and guiding them home, delivering a parcel to
another town, cooking a dish, repairing a building, fishing, mining, tending
a herd, bringing in a harvest, clearing a road, escorting a trader and
lighting the town's lamps. They SHALL be counted by the same rule as the
starter missions.

#### Scenario: A delivery

- **WHEN** the player takes a delivery and hands the parcel to the named
  person in the other town
- **THEN** the job is done, and returning to the giver pays its reward

#### Scenario: A lost villager

- **WHEN** the player finds the lost villager and talks to them
- **THEN** they follow the player, wait at deep water, and the job is done
  when they are inside their town's wall

### Requirement: Repairs never touch the player's work

A broken piece a repair job asks for SHALL be a change the world made to a
town building, and SHALL never be a piece the player placed.

#### Scenario: A player's wall in town

- **WHEN** the world chooses a piece to break for a repair job
- **THEN** no piece the player placed is chosen
