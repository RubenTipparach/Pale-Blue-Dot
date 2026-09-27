# World: persistence

## Purpose

What a saved world is made of. The seed and the rules derive everything they
can. What is generated once and may then change is stored as a record. Every
change after that, by the player or by the world itself, is an authored entry
in one durable journal. A world larger than memory loads by region.

## ADDED Requirements

### Requirement: A world's identity names every version that shapes it

A saved world SHALL record its seed, its topology version, its generator
version and the schema version of each record kind it holds. A save SHALL NOT
be opened under a version the running build does not have. A save that
predates a recorded version SHALL read as the version it was made under.

#### Scenario: A save from before identities

- **WHEN** a save with no identity is opened
- **THEN** it reads as generator version 4 and the current topology version,
  and the identity is written to it before anything else is

#### Scenario: A version the build does not have

- **WHEN** a save records a generator version this build does not carry
- **THEN** it is refused, and the menu names the version

### Requirement: Laws are derived, facts are stored

Anything the seed and the rules decide SHALL be derived when it is needed and
SHALL NOT be written to the save:
- terrain, biomes and rivers;
- trees and clutter;
- water classes and fish habitat;
- the candidates a site is chosen from.

Anything generated once that must then stay put or be able to change SHALL be
a record, generated when the world is made, written to the save before it is
shown, and read from the save from then on. This covers sites, settlements
and their buildings, landmarks, and species zones with a name or a state.

#### Scenario: A rule is retuned

- **WHEN** a derived rule changes, such as the tree scatter's density
- **THEN** an existing world shows the new rule, and its records are unchanged

#### Scenario: The rules that made a record change

- **WHEN** the site rules are retuned after a world was made
- **THEN** that world's sites are the ones in its save, and a new world gets the
  new rules' sites

### Requirement: Every change is an authored entry in one durable journal

Every change to the world after its creation SHALL be one journal entry,
naming its author: the player, a world process with the record it acted for,
or the world's creation. An entry SHALL be acknowledged only after the storage
backend has committed it. The journal's order SHALL be the world's history,
and replaying it SHALL rebuild the world exactly.

#### Scenario: A dig

- **WHEN** the player digs a cell
- **THEN** the journal gains one entry authored by the player, acknowledged once
  it is on disk

#### Scenario: A world change

- **WHEN** a world process marks a building abandoned
- **THEN** the journal gains one entry authored by that process for that
  building's record, and a reload shows it abandoned

### Requirement: The world yields to the player

A world-authored change SHALL be refused if it would alter a cell, a piece or
a record field that a player-authored entry has changed. The refusal SHALL be
a result the process sees, never a partial write.

#### Scenario: A house the player rebuilt

- **WHEN** a process would abandon a building whose door the player has opened
  or whose walls the player has edited
- **THEN** the change is refused, and the building is as the player left it

### Requirement: World processes are deterministic and never replayed

A world process SHALL decide its changes from the seed, the period of the
world clock being evaluated, and the stored state alone. Its changes SHALL be
journaled like a player's. Loading a save SHALL replay the journal and SHALL
NOT run a process again. World time SHALL pass only while the world is played. A world
evaluated over many periods at once SHALL reach the same state as one
evaluated a period at a time.

#### Scenario: Catching up

- **WHEN** a world is evaluated one period at a time for ten periods, and a copy
  of it is evaluated for all ten at once
- **THEN** both journals end with the same changes and both worlds are equal

#### Scenario: A reload

- **WHEN** a save whose journal holds process entries is loaded
- **THEN** no process runs during the load, and the world equals the one that
  was saved

#### Scenario: A world left closed

- **WHEN** a world is saved, left closed for a week, and opened again
- **THEN** its clock reads what it was saved with, and no process has anything
  due

### Requirement: Records survive builds that do not know them

A record SHALL carry its kind, a stable id and its kind's schema version. A
new record kind SHALL be added without changing the save's format. A build
that meets a record of a kind or schema version it does not know SHALL keep it
and write it back unchanged.

#### Scenario: A newer save in an older build

- **WHEN** a save holding a landmark record is opened by a build with no
  landmarks, and the player digs and quits
- **THEN** the landmark record is still in the save, byte for byte

### Requirement: A world larger than memory loads by region

The journal SHALL be checkpointed into snapshots, one per level-5 region.
Loading SHALL read the regions near the player and the journal entries after
the last checkpoint, and SHALL read other regions as they are approached. A
checkpoint SHALL write its new snapshots before it moves the journal's start,
so a crash at any point loses no committed entry.

#### Scenario: A million edits

- **WHEN** a save holds a million edits spread over the planet
- **THEN** it loads reading only the regions near the player and the journal's
  tail, and every edit is present when its region is reached

#### Scenario: A crash during a checkpoint

- **WHEN** the process is killed at each step of a checkpoint in turn
- **THEN** every reload has every committed entry, exactly once
