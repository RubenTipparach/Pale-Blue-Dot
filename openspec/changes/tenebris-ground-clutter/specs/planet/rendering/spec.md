# Surface Rendering Specification

## ADDED Requirements

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
