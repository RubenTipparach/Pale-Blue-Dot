# World: oil

## Purpose

Oil fields under the ground, the pumpjack, the refinery and the oil plant.
Proposed with the mockups; nothing here is built.

## ADDED Requirements

### Requirement: Oil fields are derived from the seed and change no cell
Oil fields SHALL be a field derived from the world's seed and never saved.
Adding them SHALL change no cell of any world, so every world, old saves
included, has the same fields without a new generator version.

#### Scenario: An old world has oil
- **WHEN** a world saved before oil existed is opened
- **THEN** its ground is cell for cell what it was, and its oil fields are
  the ones a new world with the same seed has

### Requirement: A well is a stored record that draws a field down
A pumpjack SHALL be a stored record naming its builder, and its daily draw
SHALL fall as its share of the field's pressure is drawn down.

#### Scenario: A well slows
- **WHEN** a pumpjack has drawn half its share of a field
- **THEN** its daily draw is below what it drew on its first day
