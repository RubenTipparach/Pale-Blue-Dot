# Reference Frames Specification

## Purpose
System positions are astronomical and collision is local, so the two live in
different number formats and different origins. Keeping frame identity explicit
is what stops a 4 km planet from losing millimetres to a 150 km orbit.

## Requirements

### Requirement: f64 globally, f32 locally, subtract before casting
System position, orbit epoch, body attitude and frame transforms SHALL be
stored in `f64`. Rendering and collision SHALL run in a bounded local `f32`
frame. The origin SHALL be subtracted in `f64` before casting; casting two
absolute positions and subtracting afterwards SHALL NOT be done.

#### Scenario: Rebasing at astronomical distance
- **WHEN** a pose and velocity are rebased to a new origin far from zero
- **THEN** position and velocity are preserved to the precision the origin
  exists to protect

### Requirement: Composed motion reconstructs inertial motion
A body moving inside a moving frame SHALL compose to the correct world position
and velocity, including the angular term. Velocity SHALL NOT be inferred by
differencing a rebased transform.

#### Scenario: Local motion in a moving frame
- **WHEN** local position and velocity are composed through a frame with its own
  origin velocity and angular velocity
- **THEN** the reconstructed world motion matches the inertial answer

### Requirement: Rails are closed-form and do not drift
Planet, moon and station poses SHALL be closed-form functions of tick time and
stored orbital elements, with hierarchical parents composing in `f64`. Repeated
evaluation SHALL NOT accumulate drift.

#### Scenario: A circular orbit over many periods
- **WHEN** a circular rail is evaluated repeatedly
- **THEN** its radius, period and tangential speed do not drift

#### Scenario: A child orbit
- **WHEN** a child body is evaluated
- **THEN** it inherits its parent's position and velocity

#### Scenario: A stationary root
- **WHEN** a root body with no orbit is evaluated
- **THEN** it reports no position or velocity drift
