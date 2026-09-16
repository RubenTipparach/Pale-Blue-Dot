# Surface Rendering Specification

## Purpose
The CPU owns the terrain; the GPU draws what the CPU says. Geometry is
reconstructed in the vertex shader from a persistent per-column record, so a
whole closed globe costs one storage buffer and one indirect draw rather than
an expanded vertex buffer.

## Requirements

### Requirement: The CPU is authoritative
Terrain heights, materials and edits SHALL live on the CPU. GPU geometry,
visibility compaction and shading SHALL be derived state. Collision, mining,
persistence and any future multiplayer decision SHALL NOT require GPU readback.

#### Scenario: A frame that draws the surface
- **WHEN** the surface is drawn
- **THEN** the columns it draws from are the same CPU records the walking
  controller collides against
- **AND** nothing read back from the GPU decides a gameplay outcome

### Requirement: Uploads match the shader ABI exactly
The Rust upload type and the WGSL storage layout SHALL be checked against each
other by inspecting the actual types and the actual parsed shader, not by
comparing two hand-written expected values.

#### Scenario: Building the pipeline
- **WHEN** the planet pipeline layouts are built
- **THEN** the per-column record's size and member offsets are asserted against
  the shader's declared layout
- **AND** storage bindings declare the access each pass actually uses

### Requirement: Pixel art is sampled unfiltered
Terrain material texels SHALL be sampled at nearest with an explicit tile grid.
Linear filtering SHALL NOT be substituted.

#### Scenario: Shading a terrain fragment
- **WHEN** the live surface material samples the atlas
- **THEN** it reads integer texels without interpolation
- **AND** the tile grid and its seam inset are explicit

### Requirement: Draw work is bounded and complete
A draw SHALL either publish a complete geometry generation or none. Vertex
budget MAY vary with camera altitude, but that switch SHALL change vertex work
only, never the authoritative topology resolution.

#### Scenario: Camera above the foliage cutoff
- **WHEN** the camera rises past the cosmetic-geometry altitude
- **THEN** the draw emits the terrain-only vertex count per visible column
- **AND** the column set and their heights are unchanged
