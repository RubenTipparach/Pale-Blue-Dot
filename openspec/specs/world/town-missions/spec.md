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

### Requirement: A lost villager is walked home
A lost villager job SHALL stand a person of the town out of it until the
player talks to them; from then they SHALL follow the player at the
player's pace, never onto deep water, and the job SHALL count them home
inside their town's wall.

#### Scenario: Home past a river
- **WHEN** the player leads a found villager toward a deep river and then
  back into their town
- **THEN** they wait at the river's edge, and the job counts them at the wall

Pinned by `pbd-app` `economy::tests::a_lost_villager_follows_waits_at_deep_water_and_counts_at_the_wall`.

### Requirement: Repairs never touch the player's work
A broken piece a repair job asks for SHALL be a change the world made to a
town building through the yield check, and SHALL never be in a building the
player has changed or a piece the player placed. Placing the material back
SHALL mend it and count the job.

#### Scenario: A player's wall in town
- **WHEN** the world chooses a wall to break for a repair job in a town
  whose buildings the player has changed
- **THEN** nothing is broken, and the buildings are as the player left them

#### Scenario: Mended with planks
- **WHEN** the player places two planks at a broken wall
- **THEN** the wall stands again and the repair job is counted

Pinned by `pbd-app` `economy::tests::a_repair_breaks_no_players_work_and_placing_planks_counts`.
