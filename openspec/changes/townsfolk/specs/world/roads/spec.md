# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns. The slope, the network, cut and
fill, the lamps near towns and the player's own road cells are built and in
`openspec/specs/world/roads` (2026-11-22); the road jobs are left here, and
the roads marked by waypoints and paved with gravel from a grinder and a
mixer (design 39 to 41) are proposed here.

## ADDED Requirements

### Requirement: Jobs on the roads

A town's board SHALL be able to post a rockfall on a road to clear and a
trader to walk with to the next town, and each SHALL be counted by the
starter missions' rule.

#### Scenario: A rockfall

- **WHEN** the player takes a rockfall job and digs the road cell clear
- **THEN** the job counts it, and the road is walked again

### Requirement: A road is marked before it is paved

The player SHALL mark a road for paving by driving survey stakes as waypoints. The
cells between two stakes SHALL be found by the world's road search, at most
a layer apart, off the sea, out of towns and off any cell the player has
edited unless it is a road cell, and every one SHALL be marked. A stake with
no such way SHALL be refused with nothing taken.

#### Scenario: Two stakes on a hillside

- **WHEN** the player drives a stake, then another 12 cells away up a slope
- **THEN** the cells between are marked, and no two marked neighbours differ
  by more than a layer

#### Scenario: Too steep

- **WHEN** the only way to the aimed cell climbs two layers between cells
- **THEN** the stake is refused, it stays in the pack, and nothing is marked

### Requirement: Marks are saved

A plan's stakes and cells SHALL be a durable record of the player's, written
with the pack in the same line, and SHALL read back cell for cell whatever
the road search's version.

#### Scenario: A plan after a reload

- **WHEN** the player marks a plan, closes the game and opens the world
- **THEN** the same cells are marked and the same stakes stand

### Requirement: Gravel comes from a grinder and a mixer

Gravel SHALL be made at a mixer from crushed stone and sand, and crushed
stone at a grinder from stone; neither SHALL be made by hand. Gravel already
held SHALL stay usable.

#### Scenario: From stone to gravel

- **WHEN** the player crushes 2 stone at the grinder, then mixes 3 crushed
  stone with a block of sand at the mixer
- **THEN** the pack holds 4 gravel, and the pack's craft list has no gravel
  recipe

## MODIFIED Requirements

### Requirement: Dirt and gravel

The player SHALL pave a marked cell with gravel by the interact key with the
shovel in hand and gravel in the pack, one gravel a cell, the cell's slope
cut by the world road's own rule from the plan's order, saved as the
player's with the pack in the same line, and walking SHALL be faster on
gravel than on dirt. The place button SHALL NOT lay a road. Dirt tracks
SHALL be laid by the world only, and every road cell a save already holds,
gravel or dirt, SHALL be kept and walked as it was.

#### Scenario: Building a road

- **WHEN** the player, shovel in hand, presses the interact key on a marked
  cell with gravel in the pack
- **THEN** one gravel is taken, the cell is a gravel road cell saved as the
  player's, and the walker moves faster on it

#### Scenario: No gravel

- **WHEN** the player presses the interact key on a marked cell with no
  gravel
- **THEN** nothing is taken and the cell stays marked

#### Scenario: An old player road

- **WHEN** a world holding road cells the player laid by the place button is
  opened
- **THEN** every one is drawn and walked as before
