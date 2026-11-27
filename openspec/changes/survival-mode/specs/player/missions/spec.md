# Spec Delta

## Purpose

Survival's missions: the starter tree up to the fire, then paths the player
picks, in Survival worlds only.

## ADDED Requirements

### Requirement: A Survival world's tree is the starter trunk, then paths after the fire
The mission tree of a Survival world SHALL be the starter tree as the game
has it, unchanged up to and including "Make a fire", whose ancestors SHALL
include every basic tool mission (the stone axe, the stone pickaxe and the
stone shovel). Every branch that opens from "Make a fire" SHALL be a path:
the survival paths Eat and drink, Hunting, Shelter and weather, and In the
water, and the starter tree's Fishing, Farming, Herding, Mining and
smelting, and Light and building. No survival mission SHALL hang off "Open
the pack". The survival missions SHALL keep the tree's rules (one parent
each, hidden gates for extra needs, unlocks as rewards). An Explorer world
SHALL NOT show or count any survival mission, and its tree SHALL be as
before.

#### Scenario: The pack leads to the axe in both modes
- **WHEN** a player in a new Survival world finishes the pack's mission
- **THEN** the trunk's "Craft an axe" is the current goal, as in an Explorer
  world, and no survival mission is open

#### Scenario: The paths open with the fire
- **WHEN** a Survival player finishes "Make a fire"
- **THEN** "Drink water", "Make a spear", "The first night", "A long swim",
  "Make a fishing rod", "Find seeds", "Make fences", "Dig into the hill" and
  "Build a shelter" open, and the stone axe, pickaxe and shovel have all
  been made

#### Scenario: Explorer never sees them
- **WHEN** a player opens the Missions screen in an Explorer world
- **THEN** no survival mission is shown, none is counted as done or open,
  and Fishing and Farming are drawn as their own branches, as before

#### Scenario: The paths' payoff
- **WHEN** a Survival player finishes "The first night"
- **THEN** the bedroll recipe is unlocked, and a laid bedroll sets where they
  wake

### Requirement: The player picks a path after the fire
When "Make a fire" completes in a Survival world, the game SHALL present the
mission tree with a path picker: a card for each path that has a mission
open, each with the path's icon, name, one plain line and its first
mission. Picking a card SHALL show that path in the tree and SHALL make the
path's first open mission the current goal, which the HUD's mission line
SHALL follow. The paths not picked SHALL stay open, and their missions SHALL
count as every open mission does. The player SHALL be able to change path
from the picker at any time. The chosen path SHALL be saved with the
player's missions record, and a record without one SHALL read as no path.

#### Scenario: The picker after the fire
- **WHEN** a Survival player finishes "Make a fire"
- **THEN** the Missions screen opens with the path picker over the tree

#### Scenario: A path becomes the goal
- **WHEN** the player picks Hunting in the picker
- **THEN** the tree shows the Hunting path, "Make a spear" is the current
  goal and the HUD's mission line shows it

#### Scenario: The others still count
- **WHEN** a player on the Hunting path catches a fish with "Go fishing" open
- **THEN** the catch counts for "Go fishing"

#### Scenario: Later
- **WHEN** the player closes the picker without picking
- **THEN** the HUD's mission line reads "Choose a path" and the picker shows
  the next time the Missions screen opens

### Requirement: The chosen path runs on, then the picker is offered again
While a path is chosen the current goal SHALL be the path's first open
mission in file order. When the chosen path has no open mission, because it
is done or because its next mission waits on another path's, the picker
SHALL be shown again with every path that has a mission open; a started
path whose next mission waits SHALL keep its card and name what it waits
on, and a finished path SHALL be left off. When no path has a mission open,
the current goal SHALL be the first open mission in file order.

#### Scenario: The next mission on the path
- **WHEN** a player on the Eat and drink path finishes "Drink water"
- **THEN** "Something to eat" is the current goal

#### Scenario: A finished path
- **WHEN** a player finishes the last mission of the Eat and drink path
- **THEN** the picker shows again, without Eat and drink

#### Scenario: A path that waits
- **WHEN** a player on the Hunting path finishes "A long shot" before
  "Smelt bronze" is done
- **THEN** the picker shows again, and Hunting's card says it waits on
  Smelt bronze

### Requirement: Drink missions count fresh water only
A drink mission SHALL count sips of fresh water only (a river, a lake, a
swamp pool, rain, a still, a cactus through a straw), never sea water or a
bucket filled at the sea. A sip of salt water SHALL give no water and
SHALL show the note "Salt water", and cost nothing else.

#### Scenario: A sip from the sea
- **WHEN** a Survival player with "Drink water" open drinks from the sea
- **THEN** "Salt water" shows, the water bar does not move and the mission's
  count does not move

#### Scenario: A sip from a river
- **WHEN** the same player drinks from a river
- **THEN** the water bar fills by a sip and the mission counts it
