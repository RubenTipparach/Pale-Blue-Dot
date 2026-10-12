# Walking and direct camera revision

The desktop explorer now starts on dry ground in first-person walking mode.
Mouse yaw and pitch use the current frame's raw displacement at 0.002 radians
per pixel, without interpolation, easing, or a ship-rotation clamp. Walking
keeps a radial up direction; flight supports free rotation while physical ship
acceleration and angular limits remain independent of the view.

## Implemented behavior

- WASD walks at 8 m/s, Shift sprints at 14 m/s, and Space jumps at 12 m/s.
  Eye height is 1.6 m; the configured step height is 0.6 m.
- F changes between walking and flight, retaining the ship entity and keeping
  exactly one camera active. Returning to walking places the capsule on dry
  ground below or nearby, accounting for its footprint at ledges.
- R resets the active mode. Click captures the cursor, Esc releases it, and
  F12 saves a native screenshot. `--fly` selects the former flight startup.
- CPU ground queries intersect the same flat hex/pentagon cap triangles that
  the GPU draws, including their height quantization. A bounded neighbor
  traversal supports contacts without scanning the planet or reading back GPU
  geometry. Swept radial contact prevents walking through tall terrace walls.
- Avian integrates the character once; the walking motor supplies gravity,
  tangent velocity, jumping, and surface contact. A blocked wall cannot accumulate
  downward velocity while grounded.
- Images default to nearest filtering, the terrain atlas explicitly requests
  nearest filtering, and the live terrain shader uses integer `textureLoad`.
- The extra interactive frame-pacing sleep has been removed. Interactive play
  uses VSync and requests a maximum frame latency of one; capture benchmarks
  request no VSync. The backend controls the actual presentation queue.

The contact resource shares the existing approximately 80 MiB of packed cell
data, plus approximately 15 MiB of neighbor indices and a 24 KiB search-seed
table. It does not create an ECS entity or Avian collider per terrain cell.

## Scope

This remains the subdivision-8 surface prototype: an 8 km diameter planet,
655,362 cells, roughly 19 m spacing, and 6 m height terraces. Jumping can clear
one such step; taller walls require another route or flight. Water blocks
walking entry. Swimming, caves, collision with decorative trees, physical
boarding, and editable volumetric terrain remain later work.

The [earlier validation](validation.md) and [flight performance baseline](performance.md)
preserve the previous build's evidence. See [controls](flight-controls.md) for
the current input contract and [benchmark instructions](performance-harness.md)
for reproduction.
