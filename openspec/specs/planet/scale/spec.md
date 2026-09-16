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
- **WHEN** the planet is built at subdivision 8 on a 4,000 m radius
- **THEN** the measured mean tile width is 18.88 m, ranging 17.30 m to 20.67 m
- **AND** the elevation step is 6 m
- **AND** the eye height is 1.6 m

#### Scenario: A subdivision halves the tile
- **WHEN** mean tile width is measured at two adjacent subdivision levels
- **THEN** the coarser is twice the finer within one percent
- **AND** each agrees within two percent with the equal-area hexagon
  `sqrt(4*pi*R^2 / cells / (sqrt(3)/2))`

### Requirement: The scale gap against the target is recorded
Where the preview's resolution differs from the resolution the engine design
targets, the difference SHALL be stated as a measured ratio rather than as an
adjective, and the options for closing it SHALL carry their real costs.

#### Scenario: Reading the comparison
- **WHEN** a reader asks how coarse the preview surface is
- **THEN** the documentation states 16x coarser laterally and 6x coarser
  vertically than the targeted 1 m layers and roughly 1.18 m cells
- **AND** it states why the target is unreachable by eagerly building the whole
  globe, with the memory figure that makes it unreachable
