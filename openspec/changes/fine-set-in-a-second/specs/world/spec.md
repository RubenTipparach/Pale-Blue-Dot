# World: the fine set arrives with the player

The two requirements this change made true and pinned with tests (the build
does not depend on the thread count; a floor is present wherever the shader
reads one) are in `openspec/specs/planet/rendering/spec.md`. What remains here
is measured in-game rather than pinned by a passing test, or rests on digging,
which is itself still the `dig-and-place` change.

## ADDED Requirements

### Requirement: The fine set rebuilds in under a second

A whole fine-set rebuild (the four fine bands and the column tier) SHALL take
under one second in a release build on the owner's desktop.

#### Scenario: Arriving at new ground

- **WHEN** the player steps out of the ship onto ground the resident set does
  not cover
- **THEN** the finest level and the column tier land within one second

### Requirement: A dig never waits for the streaming

Digging or placing at a finest cell whose column is not resident SHALL
generate that column on the same frame, adopt it into the tier, and apply the
edit to it.

#### Scenario: A dig past the tier's edge lands

- **WHEN** the player digs a cell of the finest level outside the resident tier
- **THEN** the layer is taken and the column is resident afterwards, named by
  each resident neighbour
