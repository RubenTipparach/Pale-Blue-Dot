## ADDED Requirements

### Requirement: A plain launch opens on the title screen

A plain desktop launch SHALL open, after the loading screen, on the title
screen: the game's logo over the open world, with CONTINUE, NEW WORLD, LOAD
WORLD, SETTINGS and QUIT.

#### Scenario: Launching the game

- **WHEN** the game is launched with no world named and no capture
- **THEN** the title screen is open once loading is done

#### Scenario: Stepping back to the title

- **WHEN** the saves page or settings is opened from the title and Escape is pressed
- **THEN** the title screen is open again
