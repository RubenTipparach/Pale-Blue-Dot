# Player settings

## ADDED Requirements

### Requirement: The player chooses the reflections' quality

The settings SHALL offer REFLECTIONS on the PICTURE tab with four levels,
OFF, LOW, MEDIUM and HIGH. HIGH SHALL draw the reflections as before the
setting existed. MEDIUM SHALL halve wet ground's reflection reach. LOW SHALL
NOT walk the reflected ray through the block grid (the voxel walk), and SHALL
keep the screen traces. OFF SHALL run no reflection pass: no capture, no
half-size trace or compose, and no screen trace; water and wet ground SHALL
still reflect the sky. A settings file without the field SHALL read HIGH.
A change SHALL apply without a restart and SHALL NOT compile a pipeline
after the loading screen has lifted.

#### Scenario: An older settings file

- **WHEN** a settings file of version 7 is read
- **THEN** REFLECTIONS is HIGH and every other field it holds is kept

#### Scenario: Turned off in play

- **WHEN** the player sets REFLECTIONS to OFF in the menu
- **THEN** the next frame runs no capture and no half-size reflection pass
- **AND** the water and wet ground still show the sky

#### Scenario: A launch flag

- **WHEN** the game is launched with `--reflections low`
- **THEN** the voxel walk does not run and the screen traces do

### Requirement: RAIN SHADOW sits on the PICTURE tab

The RAIN SHADOW row SHALL be on the PICTURE tab beside REFLECTIONS, with the
value and the flag it had on the WEATHER tab.

#### Scenario: Opening the settings

- **WHEN** the player opens SETTINGS, PICTURE
- **THEN** Reflections and Rain shadow are both on that tab
