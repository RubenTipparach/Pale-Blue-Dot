# World: town missions

## Purpose

Jobs a town offers on its board and through its people, taken by the player,
done, and paid in coins and goodwill.

## ADDED Requirements

### Requirement: A town posts jobs from its needs

Each settlement SHALL show three to five jobs on its board, made from its
needs, its trades and the game day's seed. The same day SHALL give the same
board. A job not taken SHALL be replaced after three game days.

#### Scenario: A harbour's board

- **WHEN** the player reads a harbour town's board
- **THEN** at least one job asks for fish

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

### Requirement: A taken job is kept and paid

A taken job SHALL be saved with its town and its reward, and SHALL NOT fail
or expire in Explorer. Finishing it SHALL pay coins and raise the town's
goodwill, and higher goodwill SHALL give better prices and more jobs.

#### Scenario: Paid on return

- **WHEN** the player finishes a job worth 30 coins and returns to its giver
- **THEN** the purse gains 30 coins and the town's goodwill rises

### Requirement: Repairs never touch the player's work

A broken piece a repair job asks for SHALL be a change the world made to a
town building, and SHALL never be a piece the player placed.

#### Scenario: A player's wall in town

- **WHEN** the world chooses a piece to break for a repair job
- **THEN** no piece the player placed is chosen
