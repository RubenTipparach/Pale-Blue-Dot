# World: townsfolk

## Purpose

The people of a settlement: who stands where and what they do, how they
look, and that they are solid and lit like the rest of the world, all
derived from the settlement record and the seed.

## ADDED Requirements

### Requirement: Every settlement has its people, as the mockup has them

Each settlement SHALL have people in the roles the approved mockup gives it:
- walkers going back and forth between two points;
- workers working in place (the smith hammering, the innkeeper wiping);
- idlers looking about;
- a guard at each gate.

Who they are, where they stand, and their path or post SHALL be derived from
the settlement record and the seed, and SHALL be the same each time the town
is built.

#### Scenario: The walled town

- **WHEN** the walled town is built
- **THEN** it has 16 people, among them a smith hammering at the forge and a
  guard at the gate

#### Scenario: Built twice

- **WHEN** the same settlement is built twice from the same record
- **THEN** every person has the same role, path, clothes and starting point

### Requirement: People are built from hexagonal blocks

A person SHALL be made of hexagonal prisms in the terrain's pixel style (legs,
body, belt, head, hair and arms), with clothes and skin coloured from their
settlement's people and biome. No person SHALL be drawn from cubes.

#### Scenario: A close look

- **WHEN** the camera is a metre from a townsperson
- **THEN** each body part shows six sides and the terrain's pixel texture

### Requirement: People are solid and lit

A person SHALL be a solid circle of 0.28 m, from the feet to 1.9 m, that the
walker slides round and never passes through. A person SHALL be lit by the
light field through the same sampler as the player.

#### Scenario: Walking into the smith

- **WHEN** the walker walks straight into a working smith
- **THEN** the walker slides round them, and at no tick overlaps them

#### Scenario: The forge at night

- **WHEN** the smith works at the forge at 22:30
- **THEN** the side of the smith facing the forge is lit by it
