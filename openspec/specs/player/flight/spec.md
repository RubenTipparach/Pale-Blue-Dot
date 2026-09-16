# Assisted Flight Specification

## Purpose
Ships are free-flying rigid bodies with thrust, a bounded local gravity field,
and configurable assistance. This is a bounded gameplay field, not a conserved
orbital potential, and it is deliberately not a rocket simulator.

## Requirements

### Requirement: No orbital rocket mechanics
Ship flight SHALL NOT use patched conics, Kepler element propagation,
sphere-of-influence trajectory solvers, transfer-window planning, maneuver
nodes, or n-body integration. Planets, moons and stations MAY use analytic
on-rails orbits; ships MAY NOT.

#### Scenario: A ship under gravity
- **WHEN** a ship coasts inside a gravity field
- **THEN** its motion comes from integrating thrust, gravity and damping
- **AND** no orbital element solver participates

### Requirement: Gravity falls off and stays finite
Gravity SHALL fall off as inverse square outside the body radius and SHALL
remain finite and continuous inside it, reaching zero at the centre.

#### Scenario: Twice the radius
- **WHEN** a ship sits at twice the body radius
- **THEN** the acceleration is a quarter of surface gravity

#### Scenario: Inside the body
- **WHEN** a position inside the radius, including the exact centre, is sampled
- **THEN** the acceleration is finite and continuous at the surface

### Requirement: Limits are magnitudes, and both are respected
Designer limits SHALL be magnitudes, so diagonal input cannot gain a factor of
`sqrt(3)`. Speed and acceleration SHALL both be respected at once: a speed
clamp alone is not an acceleration limiter, and generic damping alone is not a
speed controller.

#### Scenario: Sustained diagonal thrust under gravity
- **WHEN** full diagonal thrust is held while gravity acts
- **THEN** linear speed, linear acceleration, angular speed and angular
  acceleration all stay inside their limits

#### Scenario: Arriving above the cap
- **WHEN** the ship is already faster than the current mode's cap
- **THEN** steering stays inside both the speed ball and the acceleration ball
- **AND** control authority is not lost to rounding at the limit

### Requirement: Changing mode brakes rather than snaps
Lowering the speed profile SHALL ramp the target down. It SHALL NOT snap
velocity or produce an acceleration spike.

#### Scenario: Dropping to a slower profile
- **WHEN** the flight mode changes to a lower speed cap
- **THEN** velocity is not discontinuous
- **AND** no acceleration spike is produced

### Requirement: Dampeners are assistance, not authority
Inertial and rotational dampeners SHALL be switchable. Rotation damping SHALL
stabilise the ship's physical attitude and SHALL NOT smooth or delay the
player's view.

#### Scenario: Disabling dampeners
- **WHEN** dampeners are turned off
- **THEN** translation and rotation are no longer damped
- **AND** mouse look is unaffected either way

### Requirement: Terrain clearance protects against the drawn surface
Assisted flight SHALL enforce clearance against the actual rendered cap
heights. A recovery SHALL move the camera directly to the safe position rather
than easing toward it.

#### Scenario: A full circumnavigation
- **WHEN** a scripted circuit flies the whole way around the body, including
  over both poles
- **THEN** it completes using the physics integrator
- **AND** it needs no terrain projection event
