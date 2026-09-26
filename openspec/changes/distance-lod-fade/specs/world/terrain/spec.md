# Terrain: detail cross-fades with distance

## ADDED Requirements

### Requirement: Detail levels cross-fade with distance

Across a ring at each fine band's edge, `lod_fade_width` of the band wide and
centred on the camera within what the records hold, both detail levels SHALL
be drawn through complementary screen-space dither masks whose share follows
the distance, so moving through the ring changes the ground gradually and no
block appears or vanishes in one frame.

#### Scenario: Flying low past a band edge

- **WHEN** the camera flies low and the ground crosses a band edge
- **THEN** the blocks there dissolve from one level to the other over the ring as the camera moves, and do not switch at a fine-set landing

### Requirement: Trees nest across levels and fade with distance

A coarse cell's tree SHALL be the tree of the finest cell at its centre, and
across a ring the trees a coarse cell does not keep SHALL dither out with
distance; the last tree level SHALL fade to nothing across its outer ring.

#### Scenario: Flying over a forest

- **WHEN** the camera flies over a forest toward and away from it
- **THEN** trees thin and fill in gradually with distance, and none appears or vanishes whole
