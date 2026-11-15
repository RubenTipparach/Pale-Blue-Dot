# Player: missions

## Purpose

A chain of starter missions that walks a new player through the first game
loop. Proposed; nothing here is built.

## ADDED Requirements

### Requirement: Missions count what the player does

A mission's objectives SHALL be counted from the game's own events: crafting,
placing, catching, cooking, planting, harvesting, taming and smelting. A
mission SHALL open when the missions it needs are done.

#### Scenario: Catching fish

- **WHEN** "Catch fish ×3" is the current mission and the player lands three fish
- **THEN** the mission is done and the next one opens

### Requirement: Progress is saved, and old worlds join the chain

Mission progress SHALL be a durable player record of the world. A world saved
before missions existed SHALL open with the chain started, and with every
mission whose result the world already holds marked done.

#### Scenario: An old world with tools

- **WHEN** a world that owns a stone pickaxe and shovel is opened by a build with missions
- **THEN** "Make mining tools" is done
