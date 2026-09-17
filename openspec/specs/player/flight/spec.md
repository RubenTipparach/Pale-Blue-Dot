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
The inverse-square reference field SHALL fall off as inverse square outside
the body radius and SHALL remain finite and continuous inside it, reaching
zero at the centre. Gameplay actors SHALL use the bounded anchor field.

#### Scenario: Twice the radius
- **WHEN** the reference field is sampled at twice the body radius
- **THEN** the acceleration is a quarter of surface gravity
- **AND** the default anchor field contributes nothing there

#### Scenario: Inside the body
- **WHEN** a position inside the radius, including the exact centre, is sampled
- **THEN** both fields report finite acceleration, zero at the exact centre

### Requirement: An anchor field with a definite edge
Beside the inverse-square reference field there SHALL be an anchor field that
names the single body an actor is currently on or near, with full pull through
`1.4 R`, a linear taper to zero at `1.8 R`, and no contribution at or beyond that
edge. Per-body band overrides SHALL be explicit optional values; zero SHALL
NOT mean inheritance.

#### Scenario: Flying outward from a body
- **WHEN** an actor passes the inner radius
- **THEN** the pull begins tapering rather than switching off
- **AND** at or beyond the outer radius that body contributes nothing

#### Scenario: Explicitly disabling the full-pull plateau
- **WHEN** a body sets its inner multiplier to Some(0)
- **THEN** the taper begins at its centre instead of inheriting the default
- **AND** invalid bands, including an outer radius no greater than the inner
  radius, are rejected

### Requirement: The space transition is a property of the field
Whether an actor is in space SHALL be answered by the anchor field returning no
body, rather than by a separate threshold maintained elsewhere.

#### Scenario: Leaving a body entirely
- **WHEN** no body's anchor field reaches the actor
- **THEN** the actor is in space
- **AND** the flight readout reflects that query, including per-body overrides

### Requirement: The strongest pull wins, not the nearest centre
Where several bodies' anchor fields overlap, the body with the largest
multiplier SHALL be chosen. Equal multipliers SHALL choose the lower stable
body index regardless of iteration order.

#### Scenario: Overlapping fields with different radii
- **WHEN** the farther body's normalized band multiplier exceeds the nearer
  body's, even if the nearer body has a greater surface gravity
- **THEN** the farther body is the anchor

#### Scenario: Equal full-pull fields
- **WHEN** two bodies have equal multipliers
- **THEN** the lower stable body index wins in either iteration order

### Requirement: One surface constant serves both fields
The surface acceleration SHALL be one declared value, read by both fields:
25 m/s^2 per g to preserve Tenebris's human-scale fall and jump feel, independent
of planet radius. Bodies SHALL scale it with gravity_g, including explicit zero.

#### Scenario: Evaluating both fields at the surface
- **WHEN** both default fields are evaluated at a body's surface radius
- **THEN** they report the same acceleration

#### Scenario: Human-scale falling and jumping
- **WHEN** a walker falls one metre from rest at 1 g
- **THEN** the fall takes approximately 0.28 s
- **AND** a 12 m/s jump rises approximately 2.88 m, within fixed-tick integration
  tolerance

### Requirement: One implementation of the falloff
Gameplay gravity falloff SHALL be computed in exactly one place. A walking
actor, a flying one, and ship hover compensation SHALL read the same anchor
query rather than separate copies of the curve. The inverse-square reference
SHALL NOT be summed into assisted ship motion.

#### Scenario: A walker and a ship at the same altitude
- **WHEN** both query gravity at the same position in the full-pull band, taper,
  or space
- **THEN** both get the same acceleration from the same code path

#### Scenario: Released assisted controls
- **WHEN** the player releases controls inside an anchor field
- **THEN** hover assistance compensates the actual selected field

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
