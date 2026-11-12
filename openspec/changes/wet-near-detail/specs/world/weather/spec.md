## ADDED Requirements

### Requirement: Rain's fine detail is drawn only near the eye
The wet ground SHALL draw raindrop rings, the rippled water sheet and wall
rivulets only within a configured reach of the eye (20 m by default), fading
them out over a configured band before it. Past the reach the ground SHALL
stay wet: darkened, puddled and reflective, with the sun's and the lamps'
highlights.

#### Scenario: A puddle far down the street
- **WHEN** it rains and a puddle lies 40 m from the eye
- **THEN** it mirrors the sky and the scene with no rings drawn on it
- **AND** a puddle 5 m away shows the rings

#### Scenario: Walking toward a puddle
- **WHEN** the eye walks toward a puddle through the reach
- **THEN** its rings fade in over the fade band, with no frame where they appear at once
