## ADDED Requirements

### Requirement: A landing costs no hitch

Walking at the player's own speed, a fine set landing SHALL cost no frame
over 16.7 ms on the reference desktop (RTX 3070, release build).

#### Scenario: Walking down the owner's river

- **WHEN** the bot walks 1 km from `19.25, -6.75` with the frame log on
- **THEN** no frame within one second of a landing is over 16.7 ms
