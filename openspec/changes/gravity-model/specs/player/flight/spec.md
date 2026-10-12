# Assisted Flight Specification

## MODIFIED Requirements

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
