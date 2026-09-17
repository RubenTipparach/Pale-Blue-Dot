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
60-vertex indirect draw (the cap, six walls and the cut wall), and foliage SHALL
use a 108-vertex indirect draw starting at vertex 60. Foliage SHALL be eligible
on the three finest levels out to the level-9 band, at a per-material density
out of 256 taken from Tenebris's scatter rule (forest 34, grass 13, scrub 2)
times four per level above the finest, so the cover per area is the same at
every drawn distance. Runtime per-cell visibility SHALL NOT return to the CPU.

#### Scenario: Distant or nonvegetated ground
- **WHEN** a column has no decorative tree or its base is at least 1,200 m
  from the camera
- **THEN** it submits no foliage vertices
- **AND** its terrain remains independently eligible for drawing

#### Scenario: A tree crown enters the view
- **WHEN** a nearby tree's crown intersects the frustum while its terrain does
  not
- **THEN** the foliage draw retains that tree independently

### Requirement: Hexagons at every distance
A body SHALL be drawn as hexagonal tiles at every level of detail, from the
ground to the far limb. A triangle-mesh impostor SHALL NOT be substituted at
distance.

#### Scenario: Viewing a body from orbit
- **WHEN** the camera is far enough that only the base level is drawn
- **THEN** the body is made of hexagons and twelve pentagons
- **AND** its silhouette is polygonal rather than a smooth sphere

### Requirement: A resident set of levels around the player
The base level (level 7, the whole globe) SHALL be uploaded once. Each finer
level SHALL be resident only as a band around the player, regenerated as one
set off the CPU height function when the player has moved farther than the
regeneration distance, and published whole: a fine region is either the
previous complete set or the next, never a partial one. A tile's height SHALL
come from the one height function at every level, so a coarse cap and the fine
cells it covers agree.

#### Scenario: A vertex-centred fine cell
- **WHEN** a fine cell is centred on a coarse vertex
- **THEN** its direction and height are the coarse cell's, bit for bit

#### Scenario: Capacity overflow
- **WHEN** a band holds more cells than its region's capacity
- **THEN** the farthest cells are dropped, the walker's contact trusts the
  finest level only inside the radius that stayed complete, and nothing is
  written past the region

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

### Requirement: Level is quantised from distance to the player
A tile's level of detail SHALL be a function of its great-circle distance from
the player, quantised into bands of 2,400, 1,200, 600 and 300 m for levels 8 to
11, decided by the level below's cell the tile belongs to (its owner) against
one published player direction. Midpoint cells with one fine owner SHALL be
drawn and split per fragment along the owner boundary. It SHALL NOT be anchored
to the camera, so that looking around does not change any tile's level.

#### Scenario: The player stands still and looks around
- **WHEN** the camera turns or pulls back while the player does not move
- **THEN** no tile changes level

#### Scenario: Two adjacent tiles away from a threshold
- **WHEN** two neighbouring tiles are both well inside one band
- **THEN** both are assigned the same level, without consulting each other
