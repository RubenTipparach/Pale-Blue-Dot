# Planet Scale Specification

## Purpose
How wide a tile is and how tall a terrain step is decide whether a human-sized
avatar reads as human-sized. Those numbers follow from the body radius and the
subdivision level, so they are measured from the topology rather than written
down, and they are reported where a reader will see them.

## Requirements

### Requirement: Tile width is measured, not asserted
The engine SHALL derive tile width from the actual topology as the
centre-to-centre distance between neighbouring columns, through one function
that every consumer calls. A second hand-written copy of that arithmetic SHALL
NOT exist.

#### Scenario: Slope shading and the scale readout
- **WHEN** the slope-occlusion term and the startup scale readout each need the
  distance between two neighbouring columns
- **THEN** both call `tile_width`
- **AND** neither recomputes the chord inline

### Requirement: The shipped scale is reported and pinned
Startup SHALL log the measured minimum, mean and maximum tile width, the
elevation step and the walker's eye height together, so the relationship
between the world and the avatar is visible without measuring it. A test SHALL
pin the shipped mean so a change to the subdivision level or the body radius
cannot move it silently.

#### Scenario: The current preview body
- **WHEN** the planet is built with a level-7 base on a 4,800 m radius and
  level 11 resident around the player
- **THEN** the measured mean base tile width is 45.33 m, ranging 41.52 m to
  49.62 m, and the finest tile measures 2.833 m
- **AND** the elevation step is 1 m
- **AND** the eye height is 1.6 m

#### Scenario: A subdivision halves the tile
- **WHEN** mean tile width is measured at two adjacent subdivision levels
- **THEN** the coarser is twice the finer within one percent
- **AND** each agrees within two percent with the equal-area hexagon
  `sqrt(4*pi*R^2 / cells / (sqrt(3)/2))`

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
SHALL lock an authored radius to the ladder `R = 300 m * 2^(L - 7)`. The
preview body sits at 4,800 m with level 11 underfoot, and `tile_width_m`
answers the width at any level from that one law.

#### Scenario: Reading a body's configuration
- **WHEN** a body declares a radius and a subdivision level
- **THEN** the pair sits on that ladder

#### Scenario: The finest tier on the preview body
- **WHEN** a level-11 cap is generated on the 4,800 m body
- **THEN** its mean tile width measures 2.833 m within the geodesic spread

### Requirement: The subdivision helper is not ported as-is
Tenebris's `subdivisions_for_radius` SHALL NOT be adopted as the rule. It writes
the constant as 1.05 where the measured value is 1.2087, and it clamps the level
at 7, so it stops holding the standard on any body large enough to need a higher
level.

#### Scenario: A body above the clamp
- **WHEN** a body needs a level above 7 to hold the standard
- **THEN** it is given that level
- **AND** the tile size is not allowed to grow instead

### Requirement: Relief is authored for a walker
Terrain relief SHALL be compressed so that summits land near 150 m and the
ocean floor near 60 m below the sea, with the seeded coastline unchanged, so a
mountain is something a walker climbs in one-metre steps rather than scenery.
A test SHALL pin the range.

#### Scenario: Measuring the relief
- **WHEN** `surface_height` is sampled evenly over the sphere
- **THEN** the highest point lies between 120 m and 180 m
- **AND** the lowest lies between 40 m and 90 m below the sea

### Requirement: The scale gap against the target is recorded
Where the preview differs from the engine the design targets, the difference
SHALL be stated as a measured fact rather than an adjective. The lateral gap is
closed: the tier a player occupies is the 2.833 m tile and the 1 m step. What
remains is that the surface is one height per column rather than a volume of
layered cells, which the voxel-engine-foundation change owns.

#### Scenario: Reading the comparison
- **WHEN** a reader asks how the preview surface differs from the target engine
- **THEN** the documentation states that the tile and the step match the
  standard
- **AND** it states that the surface is one height per column, not a volume
