# World: water

## ADDED Requirements

### Requirement: Water flows by Luanti's rule on the hex grid

A water cell SHALL be a source or a flowing cell with a level from 7 to 0.
A queued cell SHALL take its state from its eight neighbours (up, six sides,
down) by Luanti's liquid transform:
- falling first;
- a cell beside a source at level 7;
- otherwise the highest level a neighbour offers: a side neighbour's level
  less one, or a flowing cell above's level plus four, capped at seven;
- liquid only within the range of 8.

Three same-level or upper sources around a cell SHALL make it a source.

#### Scenario: A spring on a slope

- **WHEN** a source is poured on a hillside
- **THEN** water falls to the ground below and spreads, each cell one level
  lower than the cell it came from, and stops seven cells from the last
  source or fall

#### Scenario: Two sources do not flood a plain

- **WHEN** two sources are poured side by side on a flat floor
- **THEN** no new source forms, and the flowing water around them stops at
  the range

### Requirement: Only edits start water moving

Generated water SHALL NOT be queued. Water SHALL start to move only from an
edit beside it, so opening a world never changes its water.

#### Scenario: Opening an old world

- **WHEN** a world saved before flowing water is opened
- **THEN** every water cell is where it was, and none is queued
