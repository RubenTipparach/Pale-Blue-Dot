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
them. An Explorer world SHALL NOT show or count any of them.

#### Scenario: The first mission after the pack
- **WHEN** a player in a new Survival world finishes the pack's mission
- **THEN** "Drink water" opens beside the trunk's next mission

#### Scenario: Explorer never sees it
- **WHEN** a player opens the Missions screen in an Explorer world
- **THEN** no Survival mission is shown, and none is counted as done or open

#### Scenario: The branch's payoff
- **WHEN** a Survival player finishes "The first night"
- **THEN** the bedroll recipe is unlocked, and a laid bedroll sets where they
  wake
