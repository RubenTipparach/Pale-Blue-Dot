# Player: tool crafting

## Purpose

How the player gets tools: tier 0 by hand, a workbench, and tools by tier.

## ADDED Requirements

### Requirement: Hands gather soft ground

Bare hands SHALL break soil, dirt, grass, sand, snow and tilled soil at a
quarter of the stone shovel's speed. They SHALL break no rock, ore or wood.

#### Scenario: Digging dirt by hand

- **WHEN** a player with nothing in hand holds the dig button on dirt
- **THEN** the dirt breaks in four times the stone shovel's time and goes into the pack

#### Scenario: Rock by hand

- **WHEN** a player with nothing in hand holds the dig button on rock
- **THEN** nothing breaks

### Requirement: Tier 0 comes off the ground by hand

Dug grass SHALL give fibre, and dug dirt, soil and sand SHALL give loose
stone, each in a share of cells chosen by the cell, never from a block a
player placed. A tree SHALL give one branch log by hand, once, and more logs
with an axe up to a limit, recorded per tree so it survives saves.

#### Scenario: A tree by hand

- **WHEN** a player with nothing in hand works a tree they have not taken from
- **THEN** one log goes into the pack, and working it again by hand gives nothing

### Requirement: Tools are crafted, and a new world owns none

A new world SHALL start with no tools owned. Tools SHALL be made at a
workbench from recipe data, and the workbench by hand from the pack. A tool's
tier SHALL set its speed and the ore it can break; a tool of the same or a
lower tier than the one owned SHALL be refused before anything is taken. A
world saved before this change SHALL open owning every tool it owned, at the
stone tier.

#### Scenario: An old world keeps its tools

- **WHEN** a world saved with all four tools is opened by a build with tool crafting
- **THEN** it owns a stone pickaxe, shovel, axe and the rod

#### Scenario: A fresh world reaches a stone pick

- **WHEN** a new world is played using only hand actions on ground within 200 m of its spawn
- **THEN** the player can make a workbench and then a stone pickaxe

#### Scenario: A better tool replaces the old one

- **WHEN** a player owning a stone pickaxe makes an iron pickaxe
- **THEN** they own an iron pickaxe, which breaks blocks 2.4 times as fast
