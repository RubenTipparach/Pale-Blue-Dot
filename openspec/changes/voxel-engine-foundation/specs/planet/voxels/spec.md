# Volumetric Voxels Specification

## ADDED Requirements

### Requirement: Occupancy, not height
A body's near-player terrain SHALL be volumetric occupancy rather than one
height per column, so a cave, an overhang and an independently removable block
are all representable. A heightmap SHALL NOT be treated as sufficient.

#### Scenario: Digging into a slope
- **WHEN** a player removes a cell below the surface
- **THEN** the cell becomes air and the cells above it remain solid
- **AND** the opened faces are drawn

### Requirement: The whole detailed globe is never allocated
Unvisited terrain SHALL be a seed and generation rules. Resident chunks and
persistent edits SHALL be sparse. A dense representation of a whole body at
near-player resolution SHALL NOT be allocated.

#### Scenario: Entering a new body
- **WHEN** a player arrives at a body never visited
- **THEN** only the chunks near them are resident
- **AND** memory does not scale with the body's surface area

### Requirement: Chunk jobs are versioned and cannot resurrect
Generation, meshing, light and collider jobs SHALL carry
`(chunk_key, revision, origin_epoch)`. A result arriving for a stale revision or
an evicted chunk SHALL be discarded.

#### Scenario: A chunk evicted while its mesh job is running
- **WHEN** the job completes after eviction
- **THEN** its result is discarded
- **AND** the chunk is not brought back by it

### Requirement: Geometry generations publish whole
A derived geometry generation SHALL be published complete or not at all.
Capacity overflow SHALL be handled without drawing partial data or writing out
of bounds.

#### Scenario: Face extraction exceeds its buffer
- **WHEN** a chunk's extracted faces would exceed the allocated capacity
- **THEN** the previous complete generation stays visible
- **AND** nothing is written past the end of the buffer

### Requirement: Edits are durable on acceptance
Every accepted world mutation SHALL enter the durable transaction path
immediately. A queued write alone SHALL NOT be treated as committed, and
commitment SHALL be acknowledged only after the storage backend succeeds.
Save-on-exit and autosave timers SHALL NOT be relied on.

#### Scenario: A block broken and the process killed
- **WHEN** a player breaks a block and the process dies immediately afterwards
- **THEN** the edit is present on the next load, or it was never acknowledged
