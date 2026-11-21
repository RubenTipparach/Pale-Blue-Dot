# World: townsfolk

## Purpose

The people of a settlement: who stands where and what they do, how they
look, and that they are solid and lit like the rest of the world, all
derived from the settlement record and the seed.

## ADDED Requirements

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

### Requirement: People are lit like the terrain

A person SHALL be lit by the light field as the terrain is (`LitByField`,
`LitLikeTerrain`): by the sun with its shadows and by the cell light field's
sky and lamps.

#### Scenario: The forge at night

- **WHEN** the smith works at the forge at 22:30
- **THEN** the side of the smith facing the forge is lit by it
