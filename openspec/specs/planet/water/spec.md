# Water Specification

## Purpose
The sea is one sheet drawn by one shader from both sides of its surface, over
a seabed the terrain draws. Which side the camera is on is decided once, on
the CPU, and every water term reads that decision.

## Requirements

### Requirement: The camera's side of the surface is one of three states
The renderer SHALL classify every view as dry, straddling or under from the
camera's body-local radius against the local sea surface, which is the sea
radius plus the shared sea function's height at the camera, and whether its
direction is over water. The straddling band SHALL be a fixed width about that
local surface.

#### Scenario: Diving through the surface
- **WHEN** a camera descends from above the sea through the surface band
- **THEN** the state passes dry, straddling, under in that order
- **AND** an emerge window arms when it comes back up and expires 2.6 s later

#### Scenario: Over land at any radius
- **WHEN** the camera's direction is over a land cell
- **THEN** the state is dry whatever its radius

#### Scenario: In a trough
- **WHEN** a camera is 0.3 m above the sea radius and a trough 1 m deep is
  passing under it
- **THEN** the state is dry

### Requirement: The sea surface is one function the renderer draws and the physics floats on
The sea's height and water velocity at a point SHALL be computed by one
function in `pbd-core` from a fixed table of components, the local sea state
and the saved world clock. The water shader SHALL draw that same table, read
from its uniform rather than from literals, and a test SHALL run the shipped
shader's height function on a headless adapter and compare it with the core
function.

#### Scenario: A hull and the drawn sea agree
- **WHEN** the core function and the shader's height function are evaluated at
  the same body-local point, time and sea state
- **THEN** the two heights agree to within 1 mm

#### Scenario: No wave literals in the shader
- **WHEN** the shipped `water.wgsl` is read
- **THEN** its displacement reads the component table from the view uniform
  and contains no hard-coded wave terms

### Requirement: Waves follow the wind without the sea jumping
Each component's wavenumber and deep-water frequency SHALL be fixed for a
planet. The local wind SHALL set only the components' amplitudes, from a
Pierson-Moskowitz spectrum scaled to a significant height of 0.21 U^2 / g, and
the sea state SHALL follow the wind with a lag rather than at once.

#### Scenario: A change of wind
- **WHEN** the wind at a place changes from one tick to the next
- **THEN** no crest moves by more than its own amplitude change
- **AND** the sea state approaches the new wind over the configured lag

#### Scenario: Wave height scales with wind and gravity
- **WHEN** the sea state has settled at a wind of U on a body with gravity g
- **THEN** the table's significant height is 0.21 U^2 / g within 5 %

### Requirement: Waves shoal in shallow water
Each component's amplitude SHALL fall to zero as the water depth goes to zero,
and the total wave height SHALL NOT exceed 0.6 times the local depth.

#### Scenario: A storm sea at the beach
- **WHEN** a 2 m sea meets water 1 m deep
- **THEN** the local wave height is at most 0.6 m
