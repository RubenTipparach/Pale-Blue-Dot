# Surface Rendering Specification

## ADDED Requirements

### Requirement: Hexagons at every distance
A body SHALL be drawn as hexagonal tiles at every level of detail, from the
ground to the far limb. A triangle-mesh impostor SHALL NOT be substituted at
distance.

#### Scenario: Viewing a body from orbit
- **WHEN** the camera is far enough that the coarsest tier is drawn
- **THEN** the body is made of hexagons and twelve pentagons
- **AND** its silhouette is polygonal rather than a smooth sphere

### Requirement: Coarse tiles are a prefix of the fine cells
A coarse level's tiles SHALL be the cells the finest level already carries, at
the same indices, rather than a separately generated or merged mesh. A coarse
tile's height SHALL be read from the same height data as the fine tile at that
index.

#### Scenario: A tile at two levels
- **WHEN** cell `i` exists at both a coarse and a fine level
- **THEN** its direction is identical at both
- **AND** its height comes from one array shared by both

### Requirement: Level selection and culling never return to the CPU
The level of detail for a tile, the visibility cull and the draw arguments SHALL
be computed on the GPU and consumed by an indirect draw. No per-tile LOD or
visibility state SHALL be read back, maintained per frame on the CPU, or
uploaded per frame.

#### Scenario: A frame while the camera moves
- **WHEN** the camera moves and the level of detail changes for some tiles
- **THEN** no buffer is read back from the GPU
- **AND** no per-tile visibility or level set is rebuilt on the CPU

### Requirement: The gold standard binds the tile underfoot
The 2.833 m tile width and 1 m cell height SHALL hold for the tier a player
occupies, on every body. A coarser tile on a distant part of the same body SHALL
NOT be treated as a violation of the standard.

#### Scenario: Standing on any body
- **WHEN** a player stands on a body and looks at the ground
- **THEN** the tiles they can walk on, dig, or stand beside are 2.833 m across
  and 1 m tall

### Requirement: A band boundary is not visible as a seam
Where two levels of detail meet, the surface SHALL remain closed: no crack to
space, no double-drawn ground, and no boundary that sweeps visibly across the
terrain as the camera moves.

#### Scenario: Crossing a level boundary
- **WHEN** the camera moves so that a region changes level
- **THEN** no gap opens between the two levels
- **AND** the change does not read as a line travelling over the ground
