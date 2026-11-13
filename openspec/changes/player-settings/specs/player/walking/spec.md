# Walking

## ADDED Requirements

### Requirement: Auto step is the player's choice

With auto step on, which is the default, a grounded walker SHALL walk up a
rise of one terrain layer without a key. With it off, a grounded walker
SHALL walk up only rises no taller than a stair's (0.35 m), and a terrain
layer SHALL stop it until it jumps.

#### Scenario: Auto step on

- **WHEN** auto step is on and the walker walks into a one-layer rise
- **THEN** it ends up standing on top of the rise

#### Scenario: Auto step off

- **WHEN** auto step is off and the walker walks into a one-layer rise
- **THEN** it stays below the rise

#### Scenario: Jumping up with auto step off

- **WHEN** auto step is off and the walker jumps while walking into a
  one-layer rise
- **THEN** it ends up standing on top of the rise
