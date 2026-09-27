## MODIFIED Requirements

### Requirement: Generation is deterministic and order-independent
Terrain SHALL be a function of integer coordinates, the seed, the generator
version, and the world's stored settlement records.
Visiting cells in a different order, or generating a chunk before or after its
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

#### Scenario: A town's terraces
- **WHEN** the columns of a settlement's footprint and margin are generated
  from inside the town outward, and again from outside in
- **THEN** every column matches exactly
