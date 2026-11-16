# Player Walking Specification

## ADDED Requirements

### Requirement: Water is entered, not blocked
A water surface SHALL NOT block the walker's tangential motion. Walking off a
beach into the sea SHALL carry the player into the water, and the seabed SHALL
be walkable ground for as long as the player's eyes are above the surface.

#### Scenario: Walking off a beach
- **WHEN** a walker moves from dry land toward a water cell
- **THEN** the move is accepted and the walker enters the water
- **AND** the walker is not held at the waterline

#### Scenario: Wading on the seabed
- **WHEN** a walker stands on a seabed shallower than their eye height
- **THEN** they are grounded and can walk along the bottom

### Requirement: Submersion changes speed, gravity and drag
While the walker's body is submerged, movement speed, gravity and vertical drag
SHALL take the reference's water multipliers, and while the eyes are submerged
the walker SHALL NOT be grounded, so a submerged walker always falls and cannot
take a standing jump.

#### Scenario: Sinking
- **WHEN** a submerged walker provides no vertical input
- **THEN** they sink, approaching a terminal speed set by the water gravity and
  the drag rate rather than accelerating without limit

#### Scenario: Swimming up
- **WHEN** a submerged walker holds the jump control
- **THEN** they rise continuously toward a terminal speed
- **AND** releasing it returns them to sinking

#### Scenario: Jumping from the seabed
- **WHEN** a walker standing on the seabed with their body in water jumps
- **THEN** the jump is weaker than the same jump on dry land

### Requirement: A swimmer can leave the water
A walker floating at the surface beside a bank SHALL be able to rise and climb
out under their own control, without a separate flight mechanic.

#### Scenario: Climbing out of deep water
- **WHEN** a walker in deep water beside a ledge holds the jump control
- **THEN** they rise to and above the surface
- **AND** can then step onto the ledge

### Requirement: Leaving the ship over water lands in the water
Toggling from flight to walking SHALL put the walker where the ship is,
floating at the surface if that is over water, and SHALL NOT move them to the
nearest shore.

#### Scenario: Toggling to walk over the open sea
- **WHEN** a pilot over deep water toggles to walking
- **THEN** the walker appears under the ship, with their feet at the water
  surface, not grounded
- **AND** the swim model takes over from there

### Requirement: A walker in water climbs a low bank on its own
While the walker's body is in water and the forward control presses it into a
bank face it is facing, the walker SHALL climb onto the ledge, without the jump
control, when the ledge's top is above the water surface and no more than two
layers over it (or, wading, no more than two layers over the feet), and SHALL
do so as a rise and a step forward of about 0.4 s, never a jump in position.
A higher ledge SHALL NOT be climbed this way. The climb SHALL follow the auto
step setting.

#### Scenario: Swimming up to a one- or two-layer bank
- **WHEN** a swimmer at the surface holds forward into a bank one or two layers
  over the water
- **THEN** they rise and step onto it within half a second and stand there

#### Scenario: A three-layer bank
- **WHEN** a swimmer holds forward into a bank three layers over the water
- **THEN** they stay in the water

#### Scenario: Swimming along the bank
- **WHEN** a swimmer moves along a low bank rather than into it
- **THEN** they do not climb it

#### Scenario: Wading at a two-layer bank
- **WHEN** a walker wading in a shallow holds forward into a bank two layers
  over the feet
- **THEN** they climb onto it
