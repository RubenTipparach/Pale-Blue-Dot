# Antialiasing Specification

## ADDED Requirements

### Requirement: Antialiasing runs after every pass that reads depth
When antialiasing is on, the game SHALL apply it to the finished frame, after
the water, fog, cloud and lens passes, and SHALL NOT multisample the main pass
at the same time.

#### Scenario: SMAA chosen
- **WHEN** the player toggles ANTIALIASING on
- **THEN** the frame is smoothed after the water's passes, and the held tool under water has no bright edge

### Requirement: No antialiasing by default
A new player's settings and a capture's SHALL have antialiasing off, and a
settings file from before the setting SHALL read as off.

#### Scenario: An older settings file
- **WHEN** a settings file without an antialiasing field is read
- **THEN** antialiasing is off and every other field keeps its value

### Requirement: Switching it on compiles nothing in play
The loading screen SHALL compile the antialiasing pipelines, so that switching
it on after READY compiles no pipeline.

#### Scenario: Turned on mid-game
- **WHEN** the player toggles ANTIALIASING on after the loading screen
- **THEN** no `LATE_PIPELINE` is logged
