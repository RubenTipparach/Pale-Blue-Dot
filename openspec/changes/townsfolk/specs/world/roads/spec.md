# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns. The slope, the network, cut and
fill, the lamps near towns and the player's own road cells are built and in
`openspec/specs/world/roads` (2026-11-22); the road jobs are left here, and
the roads marked by waypoints and paved with gravel from a grinder and a
mixer (design 39 to 41) and planned in first person with a live ghost, a
bill and edits mid-build (design 42) are proposed here.

## ADDED Requirements

### Requirement: Jobs on the roads

A town's board SHALL be able to post a rockfall on a road to clear and a
trader to walk with to the next town, and each SHALL be counted by the
starter missions' rule.

#### Scenario: A rockfall

- **WHEN** the player takes a rockfall job and digs the road cell clear
- **THEN** the job counts it, and the road is walked again

### Requirement: A road is marked before it is paved

The player SHALL mark a road for paving by driving survey stakes as waypoints,
walking from one to the next in first person. The cells between two stakes
SHALL be the line of cells between them, each checked by the road rule (the
slope, fords and bridges, still water, towns, trees and the player's edits). Every cell of an allowed
segment SHALL be marked. A stake whose segment breaks the rule SHALL be
refused with nothing taken.

#### Scenario: Two stakes on a hillside

- **WHEN** the player drives a stake, then another 12 cells away up a slope
  that rises a layer a cell
- **THEN** the cells between are marked, and no two marked neighbours differ
  by more than a layer

#### Scenario: Too steep

- **WHEN** the line to the aimed cell climbs two layers between two cells
- **THEN** the stake is refused, it stays in the pack, and nothing is marked

### Requirement: The road planner shows the road before it is staked

While survey stakes are in hand and a plan is open, the game SHALL draw a
ghost of the finished road from the plan's last waypoint to the aimed cell,
cut by the road rule (sloped cells, wide roads on one plane, fords and
bridges), recut whenever the aimed cell changes. A cell that breaks the rule
SHALL be drawn red, and only that cell, with the first clash's reason shown.

#### Scenario: A tree on one cell

- **WHEN** the player aims past a tree standing on one cell of the line
- **THEN** that cell alone is red with "A tree in the way", the cells either
  side of it are in the ghost tint, and the stake cannot be driven until the
  tree is felled

#### Scenario: A river

- **WHEN** the aimed line crosses three cells of water deeper than a ford
- **THEN** the ghost shows a plank bridge over them at the bank's height, and
  the bill adds the bridge's planks and logs

### Requirement: The planner's menu and bill

The width (1, 2 or 3), the surface (dirt or gravel) and lamps on or off SHALL
be chosen in a menu, and the bill of what is still to build (stakes, gravel,
planks, logs, lanterns) against the pack, and the plan's turns, SHALL be shown.

#### Scenario: Wider costs more

- **WHEN** the player changes an unpaved gravel plan of 10 cells from width 1
  to width 2
- **THEN** the ghost shows the second lane, and the bill's gravel rises from
  10 to the lane cells' count added

### Requirement: A plan is edited mid-build like a site

The player SHALL grab a waypoint and move it, insert one on a segment, and
take one back, each recutting its segments live and refused where a segment
would break the rule. Paved cells the new route drops SHALL be marked for
removal and give back their gravel when taken up with the shovel, or, with
the removal setting "return", go back to their ground at once with their
gravel to the pack, by the share of gravel returned.

#### Scenario: Moving a waypoint over paved cells

- **WHEN** a waypoint is moved so that 4 paved gravel cells are no longer on
  the route, with the removal setting "mark" and a share of 1
- **THEN** the 4 cells are marked for removal, and taking each up with the
  shovel puts one gravel in the pack

### Requirement: Marks are saved

A plan's waypoints, width, surface, lamps and paved cells SHALL be a durable record of the player's, written
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
gravel than on dirt. A plan whose surface is dirt SHALL pave its cells as dirt
for no material. The place button SHALL NOT lay a road. Every road cell a save
already holds, gravel or dirt, SHALL be kept and walked as it was.

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
