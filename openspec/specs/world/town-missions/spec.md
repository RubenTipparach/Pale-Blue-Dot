# Town Missions Specification

## Purpose
Jobs a town offers on its board and through its people, taken by the player,
done, and paid in coins and goodwill (`townsfolk` design 8 and 12;
`pbd_core::economy::jobs`, `assets/config/jobs.ron`).

## Requirements

### Requirement: A town posts jobs from its needs
Each settlement SHALL show three to five jobs on its board, made from its
needs, its trades and the game day's seed. The same day SHALL give the same
board. A job not taken SHALL be replaced after three game days.

#### Scenario: A harbour's board
- **WHEN** the player reads a harbour town's board
- **THEN** at least one job asks for fish

### Requirement: A taken job is kept and paid
A taken job SHALL be saved with its town and its reward, and SHALL NOT fail
or expire in Explorer. Finishing it SHALL pay coins and raise the town's
goodwill, and higher goodwill SHALL give better prices and more jobs.

#### Scenario: Paid on return
- **WHEN** the player finishes a job worth 30 coins and returns to its giver
- **THEN** the purse gains 30 coins and the town's goodwill rises

### Requirement: Town jobs are counted by the starter missions' rule
A town job's objectives SHALL be counted by the same function as the
starter missions, from the same events, and a job naming its town SHALL
count only what happens in that town.

#### Scenario: A sale elsewhere
- **WHEN** the player has a job to sell goods in one town and sells in another
- **THEN** the job does not count the sale
