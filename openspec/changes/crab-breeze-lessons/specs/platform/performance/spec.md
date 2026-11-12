# Performance: lessons from Crab Breeze

## ADDED Requirements

### Requirement: The presentation queue depth is a launch setting

The game SHALL accept `--frame-latency <1|2>` and pass the value to the
window's `desired_maximum_frame_latency` on every run that is not a
`--capture`. The frame log's header SHALL record the value used.

#### Scenario: A benchmark run asks for a queue of two

- **WHEN** the game runs with `--no-vsync --frame-latency 2 --frame-log out.csv`
- **THEN** the window is created with a maximum frame latency of 2, and the
  header of `out.csv` names a frame latency of 2

### Requirement: Pipelines compiled after the world is ready are counted

From the frame on which the loading screen lifts, the game SHALL count every
render pipeline that finishes compiling. It SHALL log each one with its
label, and the frame log SHALL carry the running count.

#### Scenario: A material first seen in play

- **WHEN** a mesh with a material key not drawn behind the loading screen
  first comes into view after the screen has lifted
- **THEN** a `pipeline late:` line naming its pipeline is logged and the
  frame log's late-pipeline count rises by at least one

### Requirement: Town meshes that share a texture share a material

Field-lit town meshes that use the same base texture SHALL be drawn with one
shared material. Each mesh's own field and light values SHALL be read from
a per-mesh row in a storage buffer. A change to one mesh's field or lights
SHALL NOT modify any material asset.

#### Scenario: The sun moves over a town

- **WHEN** the sun moves far enough that a town mesh's field values change
- **THEN** that mesh's row is rewritten, no `FieldLitMaterial` asset is
  modified, and the town draws the same pixels as with one material per
  mesh

### Requirement: Cached ground heights equal the generator's

Any cache of the terrain generator's ground height SHALL return exactly the
value the uncached generator returns for the same generator version and
direction.

#### Scenario: A tile edge near a pentagon

- **WHEN** the cached and uncached ground are read at seeded directions on
  tile edges and next to each of the twelve pentagons
- **THEN** every pair of results is bit-identical
