# Terrain: a landing dissolves from the records it replaces

## ADDED Requirements

### Requirement: A landing dissolves the whole of the old partition

When a fine set lands and the partition of the ground into detail levels
changes, the records the old partition was drawn from SHALL stay resident
until the landing's dissolve ends, and the old half of the dissolve SHALL be
drawn from them. Every cell the old partition drew the frame before the
landing SHALL be drawn in the old half of the dissolve and every cell the new
partition draws in the new half, so no ring of ground switches in one frame,
whatever the new set's records hold.

#### Scenario: Climbing past a level's height

- **WHEN** a set lands that lays no level 11, the camera having climbed past 300 m
- **THEN** the level-11 ground under the camera dissolves into level 10 over `lod_fade_s`

#### Scenario: Descending fast, the anchor far behind the camera

- **WHEN** a set lands whose records stop short of the band the old partition drew
- **THEN** that band dissolves like the rest of the landing, and nothing is drawn whole from the new partition alone

#### Scenario: A cell both partitions draw alike

- **WHEN** a fine cell is drawn the same by the old and the new partition
- **THEN** it is drawn once, from the new set's records, and its copy in the old set's records is not drawn

### Requirement: A landing's dissolve is never cut short

A set that finishes building while the previous landing's dissolve runs SHALL
wait until that dissolve has ended before it is drawn or used, and a new
version that does not move the partition (an edit, a relight) SHALL leave a
running dissolve running.

#### Scenario: Two sets inside one dissolve

- **WHEN** a set lands less than `lod_fade_s` after the last one
- **THEN** it is held until the last dissolve ends, then swapped in and dissolved, and the log says how long it waited

#### Scenario: Digging just after a landing

- **WHEN** an edit makes a new version of the set while a landing's dissolve runs
- **THEN** the dissolve runs on to its end

### Requirement: Every landing dissolves but the first

Every landing that changes the partition SHALL dissolve, including the first
set after an empty one (it dissolves out of the base) and a set that empties
the fine levels (it dissolves into the base). Only the set that ends startup
and the first set after a change of generator SHALL be drawn at once.

#### Scenario: Descending from above the coarsest band

- **WHEN** the first fine set lands after a set with no fine levels
- **THEN** the fine levels dissolve in over the base over `lod_fade_s`

### Requirement: A tree two partitions draw differently dissolves

A tree SHALL be drawn once for both partitions only where they draw its cell
alike; elsewhere it SHALL be drawn once per partition, each through its own
half of the landing's dissolve, so its dither does not jump at a landing.

#### Scenario: A tree inside a band's cross-fade ring at a landing

- **WHEN** a set lands and the old and new partitions put a tree's cell at different depths in a band's ring
- **THEN** the tree's pixels dissolve from the old dither to the new over `lod_fade_s`

### Requirement: The dissolve runs on the game's clock

A landing's dissolve SHALL measure its progress on the game's elapsed time,
so a fixed-step capture shows it over the same game time play does.

#### Scenario: A fixed-step capture at a landing

- **WHEN** a capture steps at 60 frames a second of game time across a landing
- **THEN** the dissolve spans `lod_fade_s` of those frames
