# Assisted Flight Specification

## MODIFIED Requirements

### Requirement: No orbital rocket mechanics
Provisional on the owner's survey answer G4 (recommendation B). Assisted ships,
hover and walkers SHALL use the bounded anchor field. Rockets SHALL be
integrated Newtonian in the orbital field: inverse-square pull from every
on-rails body and the star, with no cutoff. A rocket's predicted path SHALL be
that same integration run forward and SHALL NOT come from a second model of
motion. Ship flight SHALL NOT use patched conics, Kepler element propagation,
sphere-of-influence trajectory solvers, transfer-window requirements, maneuver
nodes, or ship-to-ship gravity. Planets, moons and stations MAY use analytic
on-rails orbits; ships MAY NOT.

#### Scenario: A ship under gravity
- **WHEN** a ship coasts inside a gravity field
- **THEN** its motion comes from integrating thrust, gravity and damping
- **AND** no orbital element solver participates

#### Scenario: A rocket's predicted path
- **WHEN** a rocket coasts in low orbit round Pale Blue and its predicted path is drawn
- **THEN** the path is the orbital field integrated forward from the rocket's state with the game's step
- **AND** the rocket, left alone, flies along it

#### Scenario: An assisted ship far from every body
- **WHEN** an assisted ship flies beyond 1.8 R of every body
- **THEN** it feels no gravity, as today
