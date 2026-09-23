# World: the fine set arrives with the player

## ADDED Requirements

### Requirement: The fine set rebuilds in under a second

A whole fine-set rebuild (the four fine bands and the column tier) SHALL take
under one second in a release build on the owner's desktop, and SHALL produce
the same records whatever the thread count.

#### Scenario: Arriving at new ground

- **WHEN** the player arrives at ground the resident set does not cover
- **THEN** the finest level and the column tier land within one second

#### Scenario: The parallel build is the serial build

- **WHEN** the same anchor is built on one thread and on many
- **THEN** every record is byte-identical

### Requirement: A fine floor is present wherever the shader reads one

Every side of a coarse fine record whose neighbour the shader treats as drawn
by the next finer band SHALL carry the full fine floor for that edge.

#### Scenario: The shader's own test

- **WHEN** the wall branch's `covered_by_finer` test holds for a side
- **THEN** that side's floor equals `fine_floor` for the edge

### Requirement: A dig never waits for the streaming

Digging or placing at a finest cell whose column is not resident SHALL
generate that column on the same frame, adopt it into the tier, and apply the
edit to it.

#### Scenario: A dig past the tier's edge lands

- **WHEN** the player digs a cell of the finest level outside the resident tier
- **THEN** the layer is taken and the column is resident afterwards, named by
  each resident neighbour
