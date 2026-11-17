## ADDED Requirements

### Requirement: What is near the eye is drawn where it is

Render space SHALL be a bounded frame round the camera: the planet-local
position, in f64, minus a render origin that follows the camera, cast to f32
only after the subtraction. A thing drawn near the eye (the held tool, the
hand, the rod's line) SHALL move on screen by less than 0.05 px from one frame
to the next beyond its own motion, measured by `PBD_MOTION_TRACE` and
`tools/motion_graph.py`'s held-tool shimmer. Physics and simulation SHALL stay
in the planet's frame.

#### Scenario: Walking with a tool in hand

- **WHEN** the walker walks and turns with a tool in hand, anywhere on the
  planet
- **THEN** the tool's drawn point against its pose moves by under 0.05 px rms
  from frame to frame, where today it moves by 2.6 px rms with the origin at
  the planet's centre

#### Scenario: The origin moves

- **WHEN** the camera crosses into another 16 m cell and the origin moves
- **THEN** every drawn transform is written in the new frame in that same
  frame, and nothing is seen to jump
