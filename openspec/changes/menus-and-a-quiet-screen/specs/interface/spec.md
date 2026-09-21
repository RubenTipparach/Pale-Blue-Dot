# Interface: what is on screen, and the menus behind it

## ADDED Requirements

### Requirement: The playing screen carries only controls

While the player is in the world with no menu open, the interface SHALL draw
only the aiming crosshair and the slot row. It SHALL NOT draw letterboxing
scrims, flight or walk readouts, a frame-rate counter, or a binding list.

#### Scenario: Nothing is dimmed to make text legible

- **WHEN** the player is walking with no menu open
- **THEN** no full-width band dims the top or the bottom of the window

#### Scenario: A slot still says how many

- **WHEN** a slot holds more than one of an item
- **THEN** its count is drawn in the slot's corner, because a count is part of
  the control rather than a caption on the screen

### Requirement: Escape opens a pause menu

The player SHALL reach a pause menu with `Escape`, offering resume, settings
and quit, and SHALL step back out of it one screen at a time.

#### Scenario: Escape from the world opens the pause menu

- **WHEN** `Escape` is pressed with no menu open
- **THEN** the pause menu is shown with a resume, a settings and a quit control

#### Scenario: Escape from settings returns to the pause menu

- **WHEN** `Escape` is pressed with the settings page open
- **THEN** the pause menu is shown and the world is not resumed

#### Scenario: Escape from the pause menu resumes

- **WHEN** `Escape` is pressed with the pause menu open
- **THEN** no menu is shown and the world is listening again

### Requirement: The bindings are shown under Input in settings

The settings page SHALL show what every control does, under a section named
Input, and that list SHALL come from one table shared with any other place the
bindings are printed.

#### Scenario: The list names a key that something reads

- **WHEN** the binding table names a key
- **THEN** that key is read by one of the systems that read the keyboard

### Requirement: A menu holds the pointer and the world stops listening

While any menu is open the world SHALL NOT read the player's controls, and the
pointer SHALL be free so the menu can be used. On closing, the world SHALL take
the pointer back without the player having to click for it.

#### Scenario: A click on a menu button does not reach the world

- **WHEN** the player clicks a menu button
- **THEN** no block is dug or placed and the pointer is not captured

#### Scenario: Resuming returns to mouse look

- **WHEN** the last menu closes
- **THEN** the pointer is captured again on that frame

#### Scenario: The world does not move under a menu

- **WHEN** a menu is open and movement keys are held
- **THEN** the walker reads no movement and no look
