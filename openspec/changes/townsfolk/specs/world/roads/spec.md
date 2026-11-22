# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns. The slope, the network, cut and
fill, the lamps near towns and the player's own road cells are built and in
`openspec/specs/world/roads` (2026-11-22); the road jobs are left here.

## ADDED Requirements

### Requirement: Jobs on the roads

A town's board SHALL be able to post a rockfall on a road to clear and a
trader to walk with to the next town, and each SHALL be counted by the
starter missions' rule.

#### Scenario: A rockfall

- **WHEN** the player takes a rockfall job and digs the road cell clear
- **THEN** the job counts it, and the road is walked again
