# Planet: a night, and light that is not the sun

## ADDED Requirements

### Requirement: The sun moves, and there is one of it

The sun's direction SHALL be derived from a clock rather than held as a
constant, and every consumer - the sky, the terrain, the water, the clutter -
SHALL read the same published direction rather than a copy of its own.

#### Scenario: A day passes

- **WHEN** the clock advances through a full day
- **THEN** the sun's direction sweeps an arc and returns, and the world is lit
  from where the sun is

#### Scenario: No second copy of the sun

- **WHEN** the sun's direction is needed by a shader or a system
- **THEN** it is read from the one published value, and no call site normalises
  its own constant

#### Scenario: A capture can stop the clock

- **WHEN** an hour is pinned for a capture
- **THEN** the clock holds there, so the same run gives the same picture

### Requirement: A cell can emit light of its own

The light field SHALL carry a block channel beside the sky channel, propagated
by the same rule, so that an emitting cell lights what is near it whatever the
sky is doing.

#### Scenario: A lamp lights a dark place

- **WHEN** an emitter stands in a cell the sky never reaches
- **THEN** the cells around it are lit, falling off by one level a cell

#### Scenario: A lamp does not light through a wall

- **WHEN** solid material stands between an emitter and a cell
- **THEN** that cell takes nothing from the emitter

#### Scenario: The two channels are separate

- **WHEN** night falls
- **THEN** the sky channel dims and the block channel does not

### Requirement: A torch is a block like any other

A placeable light SHALL be a material carried by the same hotbar, targeted by
the same aim, written by the same edit path, recorded in the same save and
re-baked by the same light pass as every other material, and SHALL have an icon
in the change that adds it.

#### Scenario: A torch is placed and lights at once

- **WHEN** a torch is placed
- **THEN** the light around it changes on the same input frame, and the edit is
  in the durable log

#### Scenario: A torch is taken back

- **WHEN** a torch is removed
- **THEN** the light it gave is gone and the cell reads as it did before

#### Scenario: A torch is not a wall

- **WHEN** a player walks into a cell holding a torch
- **THEN** they pass through it, and it does not shadow itself

### Requirement: Some flowers are emitters, and the CPU decides which

A share of flowers SHALL be a bioluminescent species that emits light, and which
cells carry one SHALL be decided once on the CPU and carried in the cell record,
never rolled independently by the shader.

#### Scenario: A meadow glows at night

- **WHEN** night falls over a meadow carrying the glowing species
- **THEN** the ground near those cells is lit

#### Scenario: The shader and the bake agree

- **WHEN** the shader draws a glowing flower
- **THEN** it is on a cell the bake treated as an emitter

### Requirement: What moves samples the field

A public sampler SHALL answer the light at an arbitrary point in the tier, so
that anything not part of the baked geometry - the player, a dropped item, an
animal, a ship - is lit by the same field the ground is.

#### Scenario: A moving thing in a cave is dark

- **WHEN** a point inside an unlit cave is sampled
- **THEN** the sampler reports it unlit

#### Scenario: A moving thing by a torch is lit

- **WHEN** a point beside a torch is sampled
- **THEN** the sampler reports the torch's contribution

#### Scenario: One field, not two

- **WHEN** a moving object and the ground under it are lit
- **THEN** both read the same field, and neither has a lighting path of its own
