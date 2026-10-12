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

### Requirement: The fine set does not depend on the thread count
A fine-set build SHALL produce the same records, byte for byte, whatever the
number of threads it is built on. A height is a pure function of its
direction, so a record is the same whichever thread and memo built it.

#### Scenario: One thread and six
- **WHEN** the same anchor is built on one thread and on six
- **THEN** every level's records, the finest neighbour table, the complete
  radii and the column records are identical
  (`fast_build_tests::the_parallel_build_is_the_serial_build`)

### Requirement: A fine floor is present wherever the shader reads one
Every side of a coarse fine record on which the wall branch of
`planet_surface.wgsl` reads a fine floor (a level coarser than the finest,
the neighbour found by reflecting the centre through the edge midpoint inside
the next finer level's complete radius) SHALL carry the full fine floor for
that edge. Sides the shader never reads MAY carry the neighbour's height.

#### Scenario: The shader's own test at the spawn
- **WHEN** the shader's `covered_by_finer` holds for a side
- **THEN** the record's floor equals `fine_floor` for that edge
  (`fast_build_tests::a_floor_is_computed_wherever_the_shader_reads_one`)

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
A tile's level of detail SHALL be a function of its distance from the player,
quantised into bands of 2,400, 1,200, 600 and 300 m for levels 8 to 11.

- **Distance** is the slant distance, which counts the player's height above
  the ground. A band of radius `B` seen from height `h` SHALL cover the ground
  out to `sqrt(B^2 - h^2)`, and SHALL be empty when `h >= B`.
- **One table:** the band thresholds SHALL be computed once per frame from the
  player's position and published, and the GPU's level selection and the
  fine-set builder SHALL both read that one table.
- **Owner:** the level SHALL be decided by the level below's cell the tile
  belongs to (its owner), against one published player position. Midpoint
  cells with one fine owner SHALL be drawn and split per fragment along the
  owner boundary.
- **Not the camera:** the level SHALL NOT be anchored to the camera, so that
  looking around does not change any tile's level.

#### Scenario: The player stands still and looks around
- **WHEN** the camera turns or pulls back while the player does not move
- **THEN** no tile changes level

#### Scenario: Two adjacent tiles away from a threshold
- **WHEN** two neighbouring tiles are both well inside one band
- **THEN** both are assigned the same level, without consulting each other

#### Scenario: The player climbs
- **WHEN** the player is 300 m or more above the ground
- **THEN** no tile is drawn at level 11
- **AND** when the player is 2,400 m or more above the ground, no fine set is built at all

### Requirement: Ground clutter is derived, procedural and short-range
Decorative ground clutter SHALL be built in the vertex shader from the cell's
own record, submitted by its own indirect draw, and selected by the visibility
compute pass alone. It SHALL NOT be a CPU mesh, SHALL NOT enter the
authoritative terrain, and SHALL NOT be submitted for a cell whose geometry the
vertex path would then discard.

#### Scenario: A grass cell near the player
- **WHEN** a finest-level cell carrying a grass material is within the clutter
  radius and inside the frustum
- **THEN** the clutter draw submits one instance for it
- **AND** its blades are placed, sized and oriented by a hash of the cell id, so
  the same cell grows the same sward on every frame and every run

#### Scenario: A grass cell beyond the clutter radius
- **WHEN** the same cell is beyond the clutter radius
- **THEN** no clutter instance is submitted for it
- **AND** the cells just inside the radius draw blades whose height has faded to
  nothing, so the tier's edge is not visible

#### Scenario: A cell that grows nothing
- **WHEN** a cell's material is sand, stone, snow or water, or the cell is
  coarser than the finest level, or either of its owners is coarse
- **THEN** no clutter instance is submitted for it

### Requirement: A blade is the reference's blade
A grass blade SHALL be built from stacked tapering quads on the cell's own
surface cap, in Tenebris's own terms: a centre that walks up and leans on the
square of the height fraction, a half-width tapering by `1 - 0.55 * f`, the
ground tile's own atlas column cropped to a per-blade vertical slice, and a
base-to-tip light gradient from the configured base shade. Its normal SHALL be
the surface up, so it shades as the cap it stands on rather than popping against
it, and it SHALL be visible from both sides.

#### Scenario: A sward seen from either side
- **WHEN** the camera passes a blade so that its back faces the viewer
- **THEN** the blade is still drawn, at the same shade

#### Scenario: Blades differ within a tile
- **WHEN** one cell's blades are drawn
- **THEN** each takes its own height, width, angle, position and atlas slice from
  its own hash, within the configured bounds

### Requirement: Clutter tuning lives in validated data
Every clutter density, size and distance SHALL be read from
`assets/config/scatter.ron` through the validated loader, with the reference's
own shipped values as the defaults. No clutter number SHALL be a Rust constant
or a shader literal, and an absent field SHALL inherit the default rather than
being treated as zero.

#### Scenario: A partial override
- **WHEN** the shipped config sets only the blade count
- **THEN** every other clutter value keeps its default
- **AND** a value outside its valid range fails the load rather than drawing
