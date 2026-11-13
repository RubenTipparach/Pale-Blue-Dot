# Player: craft control

## Purpose

How a built rocket, plane or ship is steered. Proposed with the mockups;
nothing here is built.

## ADDED Requirements

### Requirement: A rocket turns by gimballing its engines
Each rocket engine SHALL gimbal within its part's limit, and the turning
torque SHALL be the sum of each engine's thrust times the sine of its gimbal
times its distance from the centre of mass. A tier-1 rocket SHALL have no
other source of torque in vacuum.

#### Scenario: No thrust, no turn in vacuum
- **WHEN** a rocket in vacuum has its engines off and the stick held over
- **THEN** its angular velocity does not change

#### Scenario: A longer arm turns faster
- **WHEN** the same engine gimbals the same angle on two stacks of equal mass
  and inertia, one with the engine farther below the centre of mass
- **THEN** the farther engine gives the larger turn rate, in proportion to
  its distance

### Requirement: A plane is steered by its control surfaces
The ailerons, elevator and rudder SHALL be the foils of the cells that carry
them, deflected within their limit, and their torque SHALL grow with dynamic
pressure. The aircraft builder SHALL refuse a plane with no aileron, no
elevator or no rudder, and SHALL name which is missing.

#### Scenario: No authority at rest
- **WHEN** a plane stands still on the runway and the stick is moved
- **THEN** the surfaces deflect and the plane does not rotate

#### Scenario: A plane with no elevator
- **WHEN** a design has wings and a fin but no tailplane
- **THEN** the builder refuses it with "no elevator: it cannot pitch"

### Requirement: A ship steers by its rudder only with way on
A ship's rudder SHALL be a wet foil, so a ship SHALL turn only while water
flows past it.

#### Scenario: Becalmed
- **WHEN** a sailing ship has no speed through the water and the helm is put
  over
- **THEN** the ship does not turn

### Requirement: A ship with no one aboard furls and stops
When the last person leaves a ship, its sails SHALL furl and its engines SHALL
stop, so that it comes to rest. It SHALL stay a stored record where it
stopped, and SHALL sail again when someone boards and takes the helm.

#### Scenario: Falling overboard
- **WHEN** the only sailor aboard a ship under sail falls into the sea
- **THEN** the sails furl within a few seconds, and the ship slows to rest
  and drifts without driving away
