# Spec Delta

## Purpose

The Survival branch of the starter missions: the survival loop taught in
the order its pressures arrive, in Survival worlds only.

## ADDED Requirements

### Requirement: A Survival world's tree has a Survival branch
The starter tree of a Survival world SHALL have a Survival branch that opens
with the pack's mission and teaches drinking, eating, a spear, a hunt, a
cooked meal and a first night, with side branches for hunting and weapons,
heat and cold, the body in water, and fire. Its missions SHALL keep the
tree's rules (one parent each, hidden gates for extra needs, unlocks as
rewards) and SHALL unlock the survival recipes in the order the player needs
them. In a Survival world, the branch's first mission SHALL be the current
goal as soon as the pack's mission is done, ahead of the trunk's next. An
Explorer world SHALL NOT show or count any of them.

#### Scenario: The first mission after the pack
- **WHEN** a player in a new Survival world finishes the pack's mission
- **THEN** "Drink water" opens beside the trunk's next mission and is the
  current goal

#### Scenario: The same moment in Explorer
- **WHEN** a player in an Explorer world finishes the pack's mission
- **THEN** the trunk's next mission is the current goal, as before

#### Scenario: Explorer never sees it
- **WHEN** a player opens the Missions screen in an Explorer world
- **THEN** no Survival mission is shown, and none is counted as done or open

#### Scenario: The branch's payoff
- **WHEN** a Survival player finishes "The first night"
- **THEN** the bedroll recipe is unlocked, and a laid bedroll sets where they
  wake

### Requirement: Fishing and farming follow the fire under the Survival branch
In a Survival world the Missions screen SHALL draw the starter tree's
Fishing and Farming branches under the Survival branch, each a child of
"Make a fire", whose own ancestors SHALL include every basic tool mission
(the stone axe, the stone pickaxe and the stone shovel), so that the player
holds all basic tools before either branch opens. The missions SHALL be the
starter tree's own, with the same ids and records; an Explorer world SHALL
draw them as before.

#### Scenario: Fishing opens after the fire
- **WHEN** a Survival player finishes "Make a fire"
- **THEN** "Make a fishing rod" and "Find seeds" open, and the stone axe,
  pickaxe and shovel have all been made

#### Scenario: The same missions in Explorer
- **WHEN** an Explorer player opens the Missions screen
- **THEN** Fishing and Farming are drawn as their own branches, as before
