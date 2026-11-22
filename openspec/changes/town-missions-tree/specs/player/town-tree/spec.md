# Player: town tree

## Purpose

A second mission tree, apart from the starter tree, that walks a player
through the town: people, trade, the board, the special jobs, roads, carts,
a house and logs, paying coins and unlocking recipes. Proposed; nothing here
is built.

## ADDED Requirements

### Requirement: The town tree is its own tree

The town tree SHALL be read from its own file, counted into its own record
(`town_missions`) and shown on its own tab of the Missions screen. No mission
of the town tree SHALL be a parent of a starter mission, or the other way
round, and every town mission SHALL have exactly one parent but the root.

#### Scenario: Two trees, two records

- **WHEN** a town mission is done
- **THEN** the `town_missions` record changes and the starter tree's
  `missions` record does not

### Requirement: The tree opens on reaching a town

The town tree SHALL open the first time the player reaches a town, whatever
the starter tree's state, and its tab SHALL not be shown before then.

#### Scenario: Early in a town

- **WHEN** a player who has not finished the starter tree's trunk walks into
  a town for the first time
- **THEN** the town tree's tab appears and "Meet the townsfolk" is open

### Requirement: A mission pays coins once

A town mission SHALL pay its coins into the purse when it is done, in the
same durable line as the mission's done mark, and SHALL never pay twice.

#### Scenario: Paving pays

- **WHEN** "Pave it" is done
- **THEN** the purse holds 30 more coins, and reloading the world neither
  pays again nor undoes the mission

### Requirement: Town missions count with the one rule

Town missions SHALL be counted by the starter missions' counting function
from the game's own events, and a thing named `any:<thing>` SHALL match that
thing in every town. One event MAY count for a starter mission, a town
mission and a taken job at once.

#### Scenario: A sale in any town

- **WHEN** "Buy and sell" is open and the player sells a fish in any town
- **THEN** its sell count goes up by one, and a taken job for fish in that
  town counts it too

### Requirement: Town missions unlock recipes

A recipe a town mission names SHALL be locked until that mission is done, by
the same rule as the starter tree's unlocks, and a recipe whose output the
world already holds SHALL stay open.

#### Scenario: The mixer waits for crushed stone

- **WHEN** "Crush stone" is not done
- **THEN** the mixer's recipe shows as locked, naming "Crush stone"

### Requirement: Old worlds open the tree in place

A world saved before the town tree SHALL open with no change to any record
it holds. If it holds proof the player has been to a town (a purse, goodwill,
a taken job, a lost villager, or the starter tree's "Find a town" done), the
town tree SHALL open on load, with no mission done and no coins paid for
work done before it.

#### Scenario: A world with coins

- **WHEN** a world with a purse but no `town_missions` record is opened
- **THEN** the town tree is open at its root, the purse is unchanged, and no
  other record differs
