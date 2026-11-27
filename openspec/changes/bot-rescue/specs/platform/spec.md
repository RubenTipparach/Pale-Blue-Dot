## ADDED Requirements

### Requirement: The showcase bot is never held in one place

A showcase bot with a point to reach more than 5 m away that has not moved 3 m in 180 s SHALL be put on dry land 20 m toward the point by the game's put-ashore rule, and SHALL log `BOT_RESCUE`.

#### Scenario: A bot in a flooded pit

- **WHEN** the bot has stood within 3 m of one place for 180 s with its goal 400 m away
- **THEN** it is put on dry land toward the goal, a `BOT_RESCUE` line is logged, and the act goes on from there
