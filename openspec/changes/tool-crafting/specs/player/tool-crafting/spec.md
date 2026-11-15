# Player: tool crafting

## Purpose

How the player gets tools: tier 0 by hand, a workbench, and tools by tier.
Proposed for the first game loop; nothing here is built.

## ADDED Requirements

### Requirement: Hands gather soft ground

Bare hands SHALL break soil, dirt, grass, sand, snow and clay at a quarter of
the stone shovel's speed. They SHALL break no rock, ore or wood.

#### Scenario: Digging dirt by hand

- **WHEN** a player with nothing in hand holds the dig button on dirt
- **THEN** the dirt breaks in four times the stone shovel's time and goes into the pack

#### Scenario: Rock by hand

- **WHEN** a player with nothing in hand holds the dig button on rock
- **THEN** nothing breaks

### Requirement: Tools are crafted, and a new world owns none

A new world SHALL start with no tools owned. Tools SHALL be made at a
workbench from recipe data. A tool's tier SHALL set its speed and the ore it
can break. A world saved before this change SHALL open owning every tool it
owned, at the stone tier.

#### Scenario: An old world keeps its tools

- **WHEN** a world saved with all four tools is opened by a build with tool crafting
- **THEN** it owns a stone pickaxe, shovel, axe and the rod

#### Scenario: A fresh world reaches a stone pick

- **WHEN** a new world is played using only hand actions on ground within 200 m of its spawn
- **THEN** the player can make a workbench and then a stone pickaxe
