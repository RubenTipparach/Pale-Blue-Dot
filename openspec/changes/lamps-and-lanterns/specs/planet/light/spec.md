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

#### Scenario: The sun does not shine through the planet

- **WHEN** the sun is below a moving thing's horizon, or the sky does not
  reach where it is
- **THEN** none of the sun's direct light reaches it: it shows the sky's fill
  (at the floor, in a cave) and any lamp's light, as the ground beside it does

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

## MODIFIED Requirements

### Requirement: Light is what reached a cell, not how deep it is
Every cell of the column tier SHALL carry a sky level derived by propagation
from the open sky, losing one level per layer up or down and three levels per
cell across, about a metre a level either way, and stopping at anything solid.

#### Scenario: An enclosed cave is dark
- **WHEN** a cell is enclosed by solid material beyond the propagation range
- **THEN** its faces are lit only by the ambient floor
  (`light::tests::a_tunnel_longer_than_the_range_goes_out`)

#### Scenario: A cave mouth is bright although it is deep
- **WHEN** a cell is open to the sky
- **THEN** it is at full sky level whatever its depth below the surrounding
  ground
  (`light::tests::an_open_column_lights_a_cave_beside_it_at_the_caves_own_height`)

#### Scenario: Twilight reaches about eleven metres into a tunnel
- **WHEN** a level tunnel runs in from an open mouth
- **THEN** its first four cells read 12, 9, 6 and 3, and it is dark from the
  fifth, whatever the tunnel's depth
  (`light::tests::a_tunnel_darkens_with_distance_from_its_mouth_not_with_depth`)

#### Scenario: The open surface is unchanged
- **WHEN** a cell's column is open to the sky
- **THEN** the light term is the same as before the field existed
  (`light::tests::open_ground_is_full_daylight_and_the_rock_under_it_is_not_lit`)
