# Terrain: detail fades instead of popping

## ADDED Requirements

### Requirement: Trees fade out before their draw distance

A tree SHALL be drawn with a screen-door fade over the last `tree_fade_m` of
the foliage range, and SHALL be fully faded before the visibility pass stops
submitting it, so no tree appears or vanishes whole as the camera moves.

#### Scenario: Walking toward a forest at the edge of the foliage range

- **WHEN** a tree line lies near 1200 m from the camera and the camera moves
- **THEN** the trees thin out or fill in pixel by pixel rather than appearing whole

### Requirement: A landing cross-fades the ground detail

When a new fine set lands, the previous partition of the ground into detail
levels SHALL remain drawn for `lod_fade_s`, each pixel showing one partition
or the other by an ordered mask against the fade's progress, so the blocks at
the band edges change shape gradually.

#### Scenario: Walking far enough for a new fine set

- **WHEN** a new fine set lands while the player walks
- **THEN** the blocks at the band edges dissolve from the old shape to the new over the fade
