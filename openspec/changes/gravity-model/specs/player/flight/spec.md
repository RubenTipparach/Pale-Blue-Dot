# Assisted Flight Specification

## ADDED Requirements

### Requirement: An anchor field with a definite edge
Beside the orbital field there SHALL be an anchor field that names the single
body an actor is currently on or near, with full pull inside an inner radius, a
linear taper to zero across a band, and no contribution beyond it.

#### Scenario: Flying outward from a body
- **WHEN** an actor passes the inner radius
- **THEN** the pull begins tapering rather than switching off
- **AND** beyond the outer radius that body contributes nothing

### Requirement: The space transition is a property of the field
Whether an actor is in space SHALL be answered by the anchor field returning no
body, rather than by a separate threshold maintained elsewhere.

#### Scenario: Leaving a body entirely
- **WHEN** no body's anchor field reaches the actor
- **THEN** the actor is in space
- **AND** nothing else has to be consulted or kept in step to establish that

### Requirement: The strongest pull wins, not the nearest centre
Where several bodies' anchor fields overlap, the body with the largest
multiplier SHALL be chosen.

#### Scenario: Hovering over a moon near its parent
- **WHEN** an actor is close above a moon whose parent planet's centre is nearer
  than the moon's
- **THEN** the moon is the anchor body

### Requirement: One surface constant serves both fields
The surface acceleration SHALL be one declared value, read by the anchor field
and the orbital field alike, so the two agree at the surface by construction.
Its value SHALL carry a recorded reason.

#### Scenario: Evaluating both fields at the surface
- **WHEN** both fields are evaluated at a body's surface radius
- **THEN** they report the same acceleration

### Requirement: One implementation of the falloff
Gravity falloff SHALL be computed in exactly one place. A walking actor and a
flying one SHALL read the same function rather than two copies of the curve.

#### Scenario: A walker and a ship at the same altitude
- **WHEN** both query gravity at the same position
- **THEN** both get the same value from the same code path
