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

### Requirement: A mission naming what the build lacks waits

A mission whose objective names an item, recipe or action that the build does
not have SHALL be pending: logged at load, shown as not yet in the build, and
treated as met by the missions that need it. Any other unknown name in the
chain SHALL be refused at load.

#### Scenario: Before tool crafting

- **WHEN** the chain names `planks` and the build has no such good
- **THEN** "Make planks and rope" is pending, and the missions after it still open

### Requirement: The HUD line can be hidden

The current mission SHALL show as one line on the HUD with its count, and a
player setting SHALL hide it without stopping the chain counting.

#### Scenario: Hidden

- **WHEN** the player turns the mission line off
- **THEN** no mission text is drawn, and a fish landed still counts

### Requirement: Every mission in the tree can be played

Every mission in the shipped tree SHALL be live in the build: no mission
waits on a good, recipe or action the game lacks, and each objective is
counted where the game already acts.

#### Scenario: Dig for bait

- **WHEN** the player digs wet ground at the shore, then lands three fish on a line cast with bait in the pack
- **THEN** "Dig for bait" is done, and each of those fish used one bait

#### Scenario: Three kinds of fish

- **WHEN** the player lands a minnow twice and a perch once while "Three kinds of fish" is open
- **THEN** it counts two kinds

### Requirement: The map marks up to four waypoints

A right click on the map SHALL add a waypoint there, and with four already
set the oldest SHALL be removed. A click on a waypoint SHALL remove it. The
waypoints SHALL be a saved record of the world. "Read the map" SHALL count
the walker reaching within 8 m of a waypoint that was set at least 200 m
from where the player stood.

#### Scenario: A fifth waypoint

- **WHEN** the player right-clicks five places on the map
- **THEN** the last four are waypoints, and the first is gone

#### Scenario: A near mark

- **WHEN** the player marks a place 150 m away and walks to it
- **THEN** "Read the map" does not count it

### Requirement: A new world earns its fences and torches

A new world's starting kit SHALL NOT hold fences, gates or torches; a saved
world SHALL keep what any kit dealt it, and a world from before the pen's
grant SHALL still be dealt its fences once.

#### Scenario: A new world

- **WHEN** a new world opens
- **THEN** its hotbar holds no fence, gate or torch, and nine of its ten slots are full
