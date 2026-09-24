# Fauna Specification

## ADDED Requirements

### Requirement: Fish swim in schools
Fish SHALL move as schools of boids: each fish steered by separation from
close neighbours, alignment with and cohesion toward its school, and a goal
the school shares. Every fish SHALL stay between the sea surface and the
seabed, and inside its species' depth band.

#### Scenario: A school over a shelving bed
- **WHEN** a school's goal lies over water shallower than its band
- **THEN** no fish of it enters that water or leaves the sea

### Requirement: Rosters are per body, and lifeless bodies have none
Each body's fish species SHALL be listed in explicit data. No species SHALL
appear on two bodies. A body whose roster is empty SHALL spawn no fish.

#### Scenario: A barren moon's sea
- **WHEN** the player casts on a body with an empty roster
- **THEN** no school spawns and no bite comes

#### Scenario: Two bodies share a species
- **WHEN** the shipped roster lists one species id on two bodies
- **THEN** a test fails

### Requirement: Schools are simulated on the CPU and deterministically
Schools SHALL be stepped on the CPU at a fixed rate, in packed arrays rather
than an entity per fish, and drawn from an uploaded instance buffer with no
readback. Two runs from the same world, seed and inputs SHALL produce the
same schools.

#### Scenario: Two runs
- **WHEN** a world is loaded twice and stepped the same number of ticks with
  the same inputs
- **THEN** every fish of every school is at the same place in both
