# World: persistence

## Purpose

What a saved world is made of, and how every saved change finds its way back
to the place it was made. This first requirement makes a cell's saved key
exact. `world-persistence` adds the rest of the save model: the identity,
stored records, the authored journal, and loading by region.

## ADDED Requirements

### Requirement: An edit applies to the cell it was made on, and no other

Every cell SHALL have a key that no other cell has, from which its lattice
address can be recovered. An edit SHALL be saved and applied by that key. A
cell SHALL roll the same clutter and texture variation from its key as it did
from the hash it replaces. A save made with hashed keys SHALL be migrated once
to exact keys, and SHALL keep its old log unchanged beside the new one.

#### Scenario: Every finest cell is its own

- **WHEN** the key of every finest cell on the planet is computed
- **THEN** no two are equal, and each unpacks to its own cell

#### Scenario: A colliding pair

- **WHEN** the player digs one cell of a pair whose old hashes collide
- **THEN** that cell is dug and its twin is untouched, before and after a reload

#### Scenario: The ground looks the same

- **WHEN** the clutter and texture seeds are computed from the new keys and from
  the old hashes, for every finest cell of a spawn tier
- **THEN** every cell's seed is the same

#### Scenario: An old save

- **WHEN** a save whose log uses hashed keys is opened
- **THEN** its edits are written to `edits.v1.log` with exact keys, each on the
  cell it was made on where the surface or the saved position tells them
  apart, and `edits.log` is left byte for byte as it was
