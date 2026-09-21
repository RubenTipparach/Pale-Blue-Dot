# World: saves

## ADDED Requirements

### Requirement: The game opens on the saves page

A desktop launch that names no world and takes no capture SHALL open on the
saves page, listing every world with LOAD, DELETE and NEW WORLD, with the most
recently played world open behind it and offered as CONTINUE.

#### Scenario: A returning player continues

- **WHEN** the game is launched with no `--world` and worlds exist
- **THEN** the saves page is shown over the most recently played world
- **AND** Escape or CONTINUE enters that world

#### Scenario: A first run makes a world by name

- **WHEN** the game is launched with no worlds saved
- **THEN** the page says there are none and offers NEW WORLD
- **AND** NEW WORLD takes a typed name and enters the world it makes

#### Scenario: A named launch skips the page

- **WHEN** the game is launched with `--world <name>` or a capture
- **THEN** it opens straight into that world
