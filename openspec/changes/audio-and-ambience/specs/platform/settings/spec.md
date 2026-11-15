## ADDED Requirements

### Requirement: The player sets five volumes
The player's settings SHALL hold five volumes, master, effects, ambience,
music and menus, each a whole percent from 0 to 100 set in steps of 10 on the
settings screen, with defaults of 80, 100, 80, 60 and 70. They SHALL be kept
in the player's settings file at version 3. A file of version 2 or older
SHALL read with the default volumes and keep every other setting it has. A
run with `--capture` or `--frame-log`, or with `PBD_TAKE_SOUND=1`, SHALL use
the default volumes whatever the file says.

#### Scenario: An older file
- **WHEN** the game reads a version 2 settings file with a 67 % render scale
- **THEN** the render scale stays 67 % and the volumes take their defaults

#### Scenario: Round trip
- **WHEN** the player sets the ambience volume to 40 % and restarts the game
- **THEN** the ambience volume is 40 %

#### Scenario: A take with sound
- **WHEN** the player has set the master volume to 0 % and a take runs with `PBD_TAKE_SOUND=1`
- **THEN** the take plays at the default volumes
