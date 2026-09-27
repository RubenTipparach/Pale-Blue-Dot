## ADDED Requirements

### Requirement: A local frame can turn
A local frame SHALL carry an orientation and an angular velocity as well as an
origin and a velocity. Composing a position and velocity through it SHALL give
the inertial answer, and a position composed out and back SHALL return to
within the `f32` precision of the local frame.

#### Scenario: A point on a turning deck
- **WHEN** a point 8 m from a frame's origin is composed through a frame
  turning at 0.3 rad/s
- **THEN** its world velocity includes the `ω × r` term and matches the
  inertial answer

#### Scenario: Out and back
- **WHEN** a local position is composed to the world and back
- **THEN** it returns to within 1 mm
