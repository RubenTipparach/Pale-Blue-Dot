# Planet Scale Specification

## ADDED Requirements

### Requirement: One hex size on every body
A cell SHALL be the same size on every body in the game. A cell is a unit of
material, and a player who digs a hex of dirt on one planet and flies to another
SHALL find the hexes the same size there.

The standard is `tenebris-rs`'s main Tenebris planet, radius 300 m at Goldberg
level 7: **2.833 m mean tile width and 1.000 m cell height**. That project's
other bodies are prototype stage and are not the reference.

#### Scenario: Travelling between two bodies
- **WHEN** a player leaves one body and lands on another
- **THEN** the mean tile width on both is 2.833 m
- **AND** one vertical layer on both is 1.000 m

#### Scenario: Authoring a new body
- **WHEN** a body is added to the catalog
- **THEN** its radius and its subdivision level together land its mean tile
  width within the measured spread of 2.833 m
- **AND** the level is chosen to serve the standard rather than the tile size
  being whatever falls out of a chosen radius

### Requirement: Radius and level are locked together
Because measured tile width is `1.2087 * R / 2^L`, holding it at the standard
SHALL lock an authored radius to the ladder `R = 300 m * 2^(L - 7)`.

#### Scenario: Reading a body's configuration
- **WHEN** a body declares a radius and a subdivision level
- **THEN** the pair sits on that ladder

### Requirement: The subdivision helper is not ported as-is
Tenebris's `subdivisions_for_radius` SHALL NOT be adopted as the rule. It writes
the constant as 1.05 where the measured value is 1.2087, and it clamps the level
at 7, so it stops holding the standard on any body large enough to need a higher
level.

#### Scenario: A body above the clamp
- **WHEN** a body needs a level above 7 to hold the standard
- **THEN** it is given that level
- **AND** the tile size is not allowed to grow instead
