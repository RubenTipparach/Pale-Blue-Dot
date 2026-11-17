# World Water Specification

## Purpose
Water is a cell like any other: the column says where it is, the sea floods
what is opened onto it, and water a player pours or uncovers moves by
Luanti's liquid rule on the hex grid. Generated water never moves by itself,
so a saved world opens as it was.

## Requirements

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
Proven by `pbd_core::liquid`'s tests and `tests/water_flow_presets.rs`,
which holds the rule to the mockup's own results, cell for cell.

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
edit beside it, so opening a world never changes its water. The rule's own
changes SHALL be saved as edits authored by the water, a flowing cell with
its level, and a save from before flowing water SHALL read as it was.
Proven by `a_poured_source_runs_by_the_rule_and_an_old_world_queues_nothing`
and `flowing_water_saves_its_level_and_an_old_line_reads_as_before`.

#### Scenario: Opening an old world

- **WHEN** a world saved before flowing water is opened
- **THEN** every water cell is where it was, and none is queued

### Requirement: The sea floods what is opened onto it

Air below sea level joined to the sea's water across the sides or downward
SHALL fill as a source, at the tier's build and on the edit that opens it,
within the tier; a pocket not joined to it SHALL stay dry, and no sea sheet
SHALL be drawn over it. Proven by `water_tier_tests`.

#### Scenario: A tunnel into the sea

- **WHEN** a tunnel under land reaches the sea below sea level
- **THEN** it is water to sea level, and a sealed pocket beside it is air

### Requirement: A world's drainage map does not depend on the core count

The drainage map a version 8 world's rivers are read from SHALL be built
with its ground sampled on every core, and SHALL be the map a one-thread
build makes, node for node, so a core count never moves a river. Proven by
`rivers::tests::the_map_built_on_every_core_is_the_one_thread_map`
(`river-cache` design 2).

#### Scenario: Opening a world on a machine with more cores

- **WHEN** the same world's drainage map is built on one thread and on
  several
- **THEN** every lattice cell drains to the same cell, with the same river
  flag, level, drop and width
