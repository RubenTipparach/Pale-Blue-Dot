# Terrain Generation Specification

## Purpose
Terrain is a pure function of position and seed, evaluated on the CPU, so that
two machines agree, a chunk generated in any order matches, and the GPU never
decides what the ground is.

## Requirements

### Requirement: Generation is deterministic and order-independent
Terrain SHALL be a function of integer coordinates and the seed alone. Visiting
cells in a different order, or generating a chunk before or after its
neighbour, SHALL produce identical results. Changing the seed SHALL change the
world.

#### Scenario: Two visit orders
- **WHEN** the same region is generated in two different orders
- **THEN** every sample matches exactly
- **AND** a different seed produces a different world

#### Scenario: Across a chunk boundary
- **WHEN** samples are taken on both sides of a chunk edge, including negative
  chunk coordinates
- **THEN** the columns are continuous across the edge
- **AND** each side reconstructs the same sample the other side sees

### Requirement: Heights are quantized and finite
Surface elevation SHALL be quantized to a single named step and SHALL be finite
for every direction, including a degenerate input. The step SHALL be declared
once, beside the body radius, rather than inline in the generator.

#### Scenario: Sampling the globe
- **WHEN** elevation is sampled over the whole sphere
- **THEN** every value is finite and an exact multiple of the elevation step
- **AND** a zero-length direction still yields a finite height

### Requirement: The generator is versioned
The generator SHALL carry a version that forms part of a saved world's
identity. Changing the algorithm for an existing save without bumping that
version SHALL NOT be done.

#### Scenario: Reading a saved world
- **WHEN** a world was saved under one generator version
- **THEN** its identity records that version
- **AND** terrain produced under a different version is not silently treated as
  the same world

### Requirement: A world has both land and ocean
The shipped generator SHALL produce substantial continents and substantial
oceans rather than degenerating to one or the other.

#### Scenario: Sampling the shipped seed
- **WHEN** elevation is sampled across the sphere
- **THEN** more than a hundred samples are above sea level
- **AND** more than a hundred are below it
