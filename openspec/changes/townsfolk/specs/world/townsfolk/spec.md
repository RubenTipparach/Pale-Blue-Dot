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
- **THEN** it has 19 people, among them a smith hammering at the forge and a
  guard at the gate

#### Scenario: Built twice

- **WHEN** the same settlement is built twice from the same record
- **THEN** every person has the same role, path, clothes and starting point

### Requirement: People look as M5's Faces tab has them

A person SHALL be the figure the owner approved in M5 version 9's Faces tab
(design 13a, 21): a bevelled block head 0.5 m wide, 0.6 m tall and 0.45 m
deep at its head scale, on a short rounded body of six-sided pieces, about
1.55 m in all, with a face of layered pieces 2 mm proud of the head's front
(eye whites, pupils with catch-lights, lids, brows, a pixel mouth and blush),
coloured from its town's people's palette with no outline. Its look SHALL be
generated from the settlement and the person's index, the same on every
visit.

#### Scenario: A close look

- **WHEN** the camera is a metre from a townsperson
- **THEN** the block head, its eyes, brows and mouth read as the Faces tab's

#### Scenario: The same faces

- **WHEN** the same town is built twice
- **THEN** every person has the same build, height, head, eyes, hair and
  clothes

### Requirement: Faces follow what happens

A person's face SHALL take one of ten expressions (neutral, happy,
delighted, surprised, worried, sad, cross, sleepy, thinking, talking) from
what happens to them, and SHALL blink, look and talk on layers of its own:
- meeting the player within 5 m: surprised, then happy;
- at night: sleepy;
- the smith at work: brows drawn down;
- a part of the game that sets an expression: that one, while it holds.

#### Scenario: Meeting the player

- **WHEN** the player walks up to within 5 m of a townsperson
- **THEN** they look surprised, then happy, and look at the player

### Requirement: People are solid and lit

A person SHALL be a solid circle of 0.28 m, from the feet to 1.9 m, that the
walker slides round and never passes through. A person SHALL be lit by the
light field as the terrain is (`LitByField`, `LitLikeTerrain`).

#### Scenario: Walking into the smith

- **WHEN** the walker walks straight into a working smith
- **THEN** the walker slides round them, and at no tick overlaps them

#### Scenario: The forge at night

- **WHEN** the smith works at the forge at 22:30
- **THEN** the side of the smith facing the forge is lit by it
