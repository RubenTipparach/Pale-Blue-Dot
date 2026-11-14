# World: opening another world from the saves page

## ADDED Requirements

### Requirement: Opening a world from the saves page shows the loading screen

Opening a world from inside the game (NEW WORLD or LOAD) SHALL write the
leaving world's pose, clock, weather and craft and drain its writer, then
show the loading screen while the new world's save is opened, its sites and
towns read or made and stored and its planet built, and keep it up until the
world has warmed and `READY` is logged. The game SHALL NOT relaunch.

#### Scenario: A new world is made while one is open

- **WHEN** NEW WORLD is pressed while a world is open
- **THEN** the loading screen is shown, the old world's files hold its last
  pose, and `READY` is logged before play resumes in the new world

### Requirement: A new world starts at the new-world spawn

A world with no saved pose SHALL start where a launch starts a new world, on
the new-world clock, with no edits and the starting kit, whatever world was
open before it. A world with a saved pose SHALL resume there.

#### Scenario: A new world made from far away

- **WHEN** a new world is made while the player stands elsewhere in another
  world
- **THEN** the walker stands at the new-world spawn, not where it stood

### Requirement: The world's name is shown on arrival

When the loading screen lifts on a saved world, the world's name SHALL be
shown with whether it is new or resumed.

#### Scenario: Arriving in a new world

- **WHEN** a new world finishes loading
- **THEN** its name and "A NEW WORLD" are shown for a few seconds
