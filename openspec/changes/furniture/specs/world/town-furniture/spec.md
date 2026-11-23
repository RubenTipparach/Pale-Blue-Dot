# World: town furniture

## Purpose

Town houses furnished by rule, townsfolk who sit and sleep in them, and town
containers holding loot the player may take.

## ADDED Requirements

### Requirement: Town houses are furnished from their records

Every town house, inn and hall SHALL be furnished by one rule from its
stored record, the same way in every build that has the rule, by the same
footprint and overlap rules as a player's furniture, and furnishing SHALL
change nothing saved and nothing a player built.

#### Scenario: An old world

- **WHEN** a world saved before furniture is opened
- **THEN** its town houses hold furniture and every saved cell, piece and
  record reads as it did

### Requirement: Townsfolk sit and sleep, one to a place

By night the townsfolk SHALL lie in their town's beds, and by day those at a
sitting post and a share of the idle SHALL sit at seats near their post. No
seat or bed place SHALL hold two bodies, the player's included.

#### Scenario: Two people, one bed

- **WHEN** two townspeople would take the same bed at night
- **THEN** one lies in it and the other takes another bed, or stays at their
  post

### Requirement: Town loot can be taken and stays taken

A town container SHALL hold seeded loot fitting its household's trade until
the player first changes it, the player SHALL be able to take any of it with
no penalty, and what is taken SHALL stay taken across a reload.

#### Scenario: Taken is taken

- **WHEN** the player empties a town chest, quits and loads the world
- **THEN** the chest is empty
