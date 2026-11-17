## ADDED Requirements

### Requirement: Fruit is picked where it hangs

F SHALL offer and pick a tree's fruit only while the eye ray meets one of its
hanging fruit within reach.

#### Scenario: Aiming at the trunk

- **WHEN** the player aims at a fruiting tree's trunk, away from its fruit
- **THEN** no pick is offered

#### Scenario: Aiming at an apple

- **WHEN** the player aims at an apple hanging on the tree within reach
- **THEN** F picks the tree's apples
