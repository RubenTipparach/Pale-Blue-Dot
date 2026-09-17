# Surface Rendering Specification

## Purpose
The CPU owns the terrain; the GPU draws what the CPU says. Geometry is
reconstructed in the vertex shader from a persistent per-column record, so a
whole closed globe uses one persistent topology buffer with separate indirect
terrain and nearby-foliage draws rather than an expanded vertex buffer.

## Requirements

### Requirement: View-dependent shading is computed in the body's own frame
Every pass that shades a body SHALL receive the camera in that body's local
frame, and SHALL compute altitude, view direction, view distance and every
Fresnel or specular term in that frame. A pass SHALL NOT treat the world origin
as a body's centre.

#### Scenario: A body away from the world origin
- **WHEN** a body is rendered at a position far from the world origin
- **THEN** its altitude-gated fog and limb rim behave exactly as they do at the
  origin
- **AND** its Fresnel and specular terms resolve against the true view vector

#### Scenario: The same body at two positions
- **WHEN** a body is rendered at the origin and then offset, with nothing else
  changed
- **THEN** the two images match

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

#### Scenario: Insufficient visibility capacity
- **WHEN** the declared column count exceeds the input or either output buffer
- **THEN** both indirect instance counts remain zero
- **AND** no partial generation is drawn

### Requirement: GPU frustum culling preserves intersecting geometry
The visibility compute pass SHALL test conservative body-local bounds against
the supplied clip volume. Bounds SHALL include cap vertices and exposed wall
bottoms, respect the explicit five or six corners, and retain geometry that
intersects a clip plane. Infinite reverse-Z projections SHALL remain supported.

#### Scenario: A cliff or pentagon at a screen edge
- **WHEN** a cell centre lies outside the clip volume but its geometry crosses
  a clip plane
- **THEN** the cell remains eligible for drawing
- **AND** the unused sixth pentagon corner does not affect its bound

### Requirement: Only nearby eligible cells submit foliage vertices
The GPU SHALL compact foliage IDs separately from terrain IDs using the
material, stable seed and body-local camera distance. Terrain SHALL use a
54-vertex indirect draw, and foliage SHALL use a 108-vertex indirect draw
starting at vertex 54. Runtime per-cell visibility SHALL NOT return to the CPU.

#### Scenario: Distant or nonvegetated ground
- **WHEN** a column has no decorative tree or its base is at least 2,300 m from
  the camera
- **THEN** it submits no foliage vertices
- **AND** its terrain remains independently eligible for drawing

#### Scenario: A tree crown enters the view
- **WHEN** a nearby tree's crown intersects the frustum while its terrain does
  not
- **THEN** the foliage draw retains that tree independently
