# Planet: light

## Purpose

The light field and everything that feeds or reads it: the sky channel that
darkens caves, the block channel that lamps fill, which materials emit and
when, and the sampler that lights things that move with the same field the
ground is lit by.

## ADDED Requirements

### Requirement: Every light source is a block with its own icon

A placeable light SHALL be a material carried by the same hotbar, aim, edit
path, save and relight as every other material. The torch, the lanterns (on a
post, on a wall bracket and hanging), the brazier and the candle SHALL each
have their own model, emission level and icon. No light SHALL show another
item's icon as a stand-in.

#### Scenario: A light is placed and lights at once

- **WHEN** a lantern is placed in the dark
- **THEN** the light around it changes on the same input frame, and the edit
  is in the durable log

#### Scenario: A light is taken back

- **WHEN** a lantern is removed
- **THEN** the light it gave is gone and the cells read as they did before it
  was placed

#### Scenario: A light is not a wall

- **WHEN** the player walks into a cell holding a lantern, a torch or a candle
- **THEN** they pass through it, and it does not shadow itself

#### Scenario: Each light has its own icon

- **WHEN** the hotbar holds a torch, a lantern, a brazier and a candle
- **THEN** each shows its own icon, and none shows a tinted tile of another
  material

#### Scenario: A candle lights a room, a brazier a square

- **WHEN** a candle and a brazier each stand alone in open, dark ground
- **THEN** the brazier's light reaches further than the candle's

### Requirement: A lamp burns all day or only from dusk to dawn

An emitter SHALL be either always lit or lit from dusk to dawn. The light
field SHALL change when the clock crosses dusk or dawn so that dusk-lit
emitters join or leave it. A light the player places SHALL be always lit
unless its kind is dusk-lit.

#### Scenario: Street lanterns come on at dusk

- **WHEN** the clock passes dusk near a row of dusk-lit lanterns
- **THEN** the ground around them is lit, and before dusk it was not

#### Scenario: They go out at dawn

- **WHEN** the clock passes dawn
- **THEN** the dusk-lit lanterns no longer light the ground, and an
  always-lit torch beside them still does

### Requirement: What moves takes the field's light

A public sampler SHALL answer both channels at an arbitrary point in the lit
tier. The held tool and hand, the ship, the fish, the float and any other
drawn thing that is not baked terrain SHALL be lit by that sampler every
frame. Beyond the lit tier the sampler SHALL answer the open sky's light and
no block light.

#### Scenario: A ship in a cave is dark

- **WHEN** the ship is parked in an unlit cave at night
- **THEN** it is drawn at the ambient floor, not as if it stood under the sky

#### Scenario: A player by a torch is lit

- **WHEN** the player stands beside a torch in the dark
- **THEN** the tool and hand they hold take the torch's warm light

#### Scenario: One field, not two

- **WHEN** a moving thing and the ground under it are lit
- **THEN** both read the same field, and neither has a lighting path of its
  own

### Requirement: Some flowers glow at night, and the CPU decides which

A share of flowers SHALL be a bioluminescent species that is a dusk-lit
emitter. Which cells carry one SHALL be decided once on the CPU and carried
in the cell record, never rolled independently by the shader.

#### Scenario: A meadow glows at night

- **WHEN** night falls over a meadow carrying the glowing species
- **THEN** the ground near those cells is lit, and by day it is not

#### Scenario: The shader and the bake agree

- **WHEN** the shader draws a glowing flower head
- **THEN** it is on a cell the bake treated as an emitter
