# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns.

## ADDED Requirements

### Requirement: Dirt and gravel

The player SHALL be able to build a dirt road with a shovel and a gravel road
with gravel, and walking SHALL be faster on gravel than on dirt.

#### Scenario: Building a road

- **WHEN** the player places a gravel road cell
- **THEN** it is saved as the player's edit, and the walker moves faster on it
