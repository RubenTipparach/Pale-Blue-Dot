# Planet Topology Specification

## Purpose
A landable body's surface is the dual of a midpoint-subdivided icosahedron: a
closed grid of hexagons with exactly twelve pentagonal defects. Every other
capability addresses the surface through this grid, so its identity, its
adjacency and its shared corners are the foundation the rest stands on.

## Requirements

### Requirement: Twelve pentagons on a closed dual
The surface SHALL be the dual of a midpoint-subdivided icosahedron, giving
`10 * 4^level + 2` cells of which exactly twelve are pentagons and the rest are
hexagons. A flat axial grid wrapped to look spherical SHALL NOT be substituted.

#### Scenario: Any subdivision level
- **WHEN** `dual_sphere(level)` is built for any level from 0 to 4
- **THEN** it returns `10 * 4^level + 2` cells
- **AND** exactly twelve of them have five corners
- **AND** every remaining cell has six

### Requirement: Corners are shared, not recomputed
Adjacent cells SHALL read one another's shared corner positions from the same
triangle centre, so no seam crack can open between two columns. Each cell SHALL
carry an ordered outward-wound corner ring and a reciprocal neighbour index per
side.

#### Scenario: Walking a cell's ring
- **WHEN** a cell's side `i` names neighbour `n`
- **THEN** `n` names this cell back at some side `j`
- **AND** the two corners of side `i` are the same two corner positions as side
  `j` of `n`, in reversed order
- **AND** the ring is wound counter-clockwise seen from outside the body

### Requirement: Stable cell and chunk addressing
Cell and chunk addresses SHALL round-trip exactly, including at negative
coordinates and chunk boundaries, so a saved address means the same cell on
every machine and after every reload.

#### Scenario: Negative and boundary coordinates
- **WHEN** a voxel at a negative or chunk-boundary coordinate is split into a
  chunk address and a local offset
- **THEN** rejoining them returns the original voxel
- **AND** the local offset is inside the chunk edge on every axis
