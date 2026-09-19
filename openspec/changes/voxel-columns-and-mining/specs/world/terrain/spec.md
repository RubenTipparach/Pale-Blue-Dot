# Terrain Specification

## ADDED Requirements

### Requirement: The finest tier is volumetric
Within the finest level of detail, terrain SHALL be stored as a column of
materials over a fixed radial span rather than as a single surface height, so
that one direction can carry solid material, then air, then solid material
again. Coarser tiers MAY remain a surface height.

#### Scenario: A cave under solid ground
- **WHEN** a column carries air between two solid runs
- **THEN** the ground above it is drawn and collided with as a ceiling
- **AND** the ground below it is drawn and collided with as a floor

#### Scenario: The bottom of the world
- **WHEN** a player digs to the lowest layer
- **THEN** that layer is bedrock and cannot be removed

### Requirement: The surface has one source across both tiers
The top of a generated column SHALL agree with the surface height the coarse
tiers draw, to within the layer height. A second description of where the ground
is SHALL NOT exist.

#### Scenario: A cell at the band edge
- **WHEN** the same direction is asked for its surface height and for its column
- **THEN** the column's topmost solid layer is that height, to the metre

### Requirement: An edit is durable on the frame it happens
A block broken or placed SHALL be written to durable storage on the same input
frame. It SHALL NOT be batched, deferred to a timer, or left to a save on exit,
and it SHALL survive a reload.

#### Scenario: Digging and reloading
- **WHEN** a player digs a hole and the world is reloaded
- **THEN** the hole is still there

#### Scenario: Walking away
- **WHEN** an edited cell leaves the finest tier and is later returned to
- **THEN** the edit is still there, rather than regenerated flat
