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

### Requirement: The walking camera is drawn between physics steps

The walking camera's position SHALL be drawn between the walker's last two
physics steps, at the fraction the fixed clock has run past the last one. Its
look SHALL stay this frame's raw mouse. A position moved outside the physics
step (a respawn, a restore, boarding, a climb-out) or a step longer than 2 m
SHALL be drawn where it is, never glided to. A `--capture`, which runs one
physics step a frame, SHALL draw the step itself.

#### Scenario: A steady walk at an uneven frame rate

- **WHEN** the walker walks at a steady speed while frames run 0, 1 or 2
  physics steps
- **THEN** the camera moves by the walk's speed every frame, not by one step
  on one frame and two on the next

#### Scenario: A teleport

- **WHEN** the walker is put somewhere else between steps
- **THEN** the camera is drawn there in the same frame
