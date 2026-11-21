# World: roads

## Purpose

Road cells that slope instead of step, in dirt and gravel, built by the
player and laid by the world between towns.

## ADDED Requirements

### Requirement: A road slopes, never steps

A road cell SHALL be a hex whose top may tilt from one layer to the next
across the cell, rising at most one layer (1 m) per cell. Its slope SHALL be
its collision, so the walker moves up and down it without a step.

#### Scenario: Up a hill

- **WHEN** the walker walks a road climbing ten layers
- **THEN** its height changes smoothly and it never steps up

### Requirement: Dirt and gravel

The player SHALL be able to build a dirt road with a shovel and a gravel road
with gravel, and walking SHALL be faster on gravel than on dirt.

#### Scenario: Building a road

- **WHEN** the player places a gravel road cell
- **THEN** it is saved as the player's edit, and the walker moves faster on it

### Requirement: Roads join the towns

The world SHALL plan roads once from the settlements and the seed, joining
each settlement to its two nearest neighbours within 2 km and the largest
towns to each other, and SHALL save them as records it authored. A road cell
SHALL be laid only where the player has made no edit. A world saved before
roads SHALL gain them when next opened, without a restart.

#### Scenario: A player's house on the route

- **WHEN** a planned road would cross a cell the player built on
- **THEN** the road goes round it and the player's cell is unchanged

#### Scenario: An old save

- **WHEN** a world saved before roads is opened
- **THEN** its towns are joined by roads, and nothing the player made has
  changed
