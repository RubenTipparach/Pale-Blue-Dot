# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns. The slope, the network, cut and
fill, the lamps near towns and the player's own road cells are built and in
`openspec/specs/world/roads` (2026-11-22), and so are the roads marked by
waypoints and paved with gravel from a grinder and a mixer and the sparse
network of roads version 3 (design 39 to 41, 43 and 44, 2026-11-23); the
road jobs are left here. The road planner's marking, record and dirt
paving (design 42 to 42l) are built and moved there (2026-11-24); its
bridges, its full bill against the pack and wide roads on one plane on a
slope are left here.

## ADDED Requirements

### Requirement: Jobs on the roads

A town's board SHALL be able to post a rockfall on a road to clear and a
trader to walk with to the next town, and each SHALL be counted by the
starter missions' rule.

#### Scenario: A rockfall

- **WHEN** the player takes a rockfall job and digs the road cell clear
- **THEN** the job counts it, and the road is walked again

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
