# Planet Light Specification

## Purpose
The light field and everything that feeds or reads it: the sky channel that
darkens caves, the block channel that lamps fill, which materials emit and
when. The field is baked on the CPU over the column tier and drawn by the GPU;
nothing reads it back. Synced from `voxel-light`, `night-and-lamps` and
`lamps-and-lanterns`, each requirement with the test that pins it.

## Requirements

### Requirement: Light is what reached a cell, not how deep it is
Every cell of the column tier SHALL carry a sky level derived by propagation
from the open sky, losing one level per cell in every direction and stopping at
anything solid.

#### Scenario: An enclosed cave is dark
- **WHEN** a cell is enclosed by solid material beyond the propagation range
- **THEN** its faces are lit only by the ambient floor
  (`light::tests::a_tunnel_longer_than_the_range_goes_out`)

#### Scenario: A cave mouth is bright although it is deep
- **WHEN** a cell is open to the sky
- **THEN** it is at full sky level whatever its depth below the surrounding
  ground
  (`light::tests::a_tunnel_darkens_with_distance_from_its_mouth_not_with_depth`,
  `light::tests::an_open_column_lights_a_cave_beside_it_at_the_caves_own_height`)

#### Scenario: The open surface is unchanged
- **WHEN** a cell's column is open to the sky
- **THEN** the light term is the same as before the field existed
  (`light::tests::open_ground_is_full_daylight_and_the_rock_under_it_is_not_lit`)

### Requirement: A face darkens where it is wedged into stone
A face's corner SHALL be darkened by how many of the two neighbouring cells
sharing that corner are solid at the layer the face opens onto, and a face's
corners SHALL be sampled independently so the value varies across it.

#### Scenario: A corner against one block is darker than an open one
- **WHEN** one of the two side neighbours is solid at that layer
- **THEN** that corner is darker than a corner with neither
  (`light::tests::a_corner_darkens_by_how_much_stone_it_is_wedged_between`)

#### Scenario: A corner wedged between two is darker still
- **WHEN** both side neighbours are solid at that layer
- **THEN** that corner is darker than one with a single neighbour
  (`light::tests::a_corner_darkens_by_how_much_stone_it_is_wedged_between`)

#### Scenario: A wall standing on flat ground has no black line at its foot
- **WHEN** every cell at a face's lowest metre is solid
- **THEN** the corner takes the light of the air above rather than reading dark
  (`light::tests::a_walls_foot_on_a_floor_and_its_head_under_a_lid_are_darker_than_its_middle`)

### Requirement: Nothing unlit is fully black
A surface the sun and sky never reach SHALL still be lit to an ambient floor,
so that an unlit interior reads as a dark place rather than as an absence.

#### Scenario: A sealed interior is legible
- **WHEN** a face has a sky level of zero
- **THEN** it is drawn at the ambient floor and its material is discernible
  (`planet::terrain::tests::the_shader_carries_the_reference_light_constants`,
  `field_light::tests::the_sky_the_cave_the_night_and_a_torch`)

### Requirement: The sun moves, and there is one of it
The sun's direction SHALL be derived from a clock rather than held as a
constant, and every consumer - the sky, the terrain, the water, the clutter -
SHALL read the same published direction rather than a copy of its own.

#### Scenario: A day passes
- **WHEN** the clock advances through a full day
- **THEN** the sun's direction sweeps an arc and returns, and the world is lit
  from where the sun is
  (`daylight::tests::the_sun_moves_through_the_day`,
  `daylight::tests::a_day_comes_round_and_the_clock_never_leaves_its_range`)

#### Scenario: No second copy of the sun
- **WHEN** the sun's direction is needed by a shader or a system
- **THEN** it is read from the one published value, and no call site keeps a
  constant of its own
  (`sky::tests::the_app_keeps_no_sun_of_its_own`)

#### Scenario: A capture can stop the clock
- **WHEN** an hour is pinned for a capture
- **THEN** the clock holds there, so the same run gives the same picture
  (`sky::tests::a_pinned_clock_holds_and_a_running_one_moves`)

### Requirement: A cell can emit light of its own
The light field SHALL carry a block channel beside the sky channel, propagated
by the same flood, so that an emitting cell lights what is near it whatever
the sky is doing. A lamp's light SHALL fall one level a layer up or down and
three levels a cell across, about a metre a level either way.

#### Scenario: A lamp lights a dark place
- **WHEN** an emitter stands in a cell the sky never reaches
- **THEN** the cells around it are lit, falling off about a level a metre
  (`light::tests::a_lamp_lights_a_buried_tunnel_and_the_sky_does_not_notice`,
  `light::tests::a_lamp_lights_a_shaft_a_level_a_layer_and_the_sky_steps_as_before`)

#### Scenario: A lamp does not light through a wall
- **WHEN** solid material stands between an emitter and a cell
- **THEN** that cell takes nothing from the emitter
  (`light::tests::a_lamp_does_not_light_through_a_wall`)

#### Scenario: The two channels are separate
- **WHEN** night falls
- **THEN** the sky channel dims and the block channel does not
  (`light::tests::the_two_channels_are_kept_apart_in_one_byte`,
  `field_light::tests::the_sky_the_cave_the_night_and_a_torch`)

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
  (`planet::column::tests::a_lantern_placed_lights_at_once_and_taken_back_leaves_the_field_as_it_was`,
  `saves::tests::a_world_comes_back_with_its_edits_its_hotbar_and_its_pose`)

#### Scenario: A light is taken back
- **WHEN** a lantern is removed
- **THEN** the light it gave is gone and the cells read as they did before it
  was placed
  (`planet::column::tests::a_lantern_placed_lights_at_once_and_taken_back_leaves_the_field_as_it_was`)

#### Scenario: A light is not a wall
- **WHEN** the player walks into a cell holding a lantern, a torch or a candle
- **THEN** they pass through it, and it does not shadow itself
  (`light::tests::each_light_has_its_level_and_none_is_solid`)

#### Scenario: Each light has its own icon
- **WHEN** the hotbar holds a torch, a lantern, a brazier and a candle
- **THEN** each shows its own icon, and none shows a tinted tile of another
  material
  (`hotbar::kit_tests::every_light_has_its_own_icon_and_nothing_else_does`)

#### Scenario: A candle lights a room, a brazier a square
- **WHEN** a candle and a brazier each stand alone in open, dark ground
- **THEN** the brazier's light reaches further than the candle's
  (`light::tests::a_brazier_reaches_further_than_a_candle`)

### Requirement: A lamp burns all day or only from dusk to dawn
An emitter SHALL be either always lit or lit from dusk to dawn. The light
field SHALL change when the clock crosses dusk or dawn so that dusk-lit
emitters join or leave it. A light the player places SHALL be always lit
unless its kind is dusk-lit.

#### Scenario: Street lanterns come on at dusk
- **WHEN** the clock passes dusk near a row of dusk-lit lanterns
- **THEN** the ground around them is lit, and before dusk it was not
  (`daylight::tests::dusk_lit_lamps_switch_once_at_dusk_and_once_at_dawn`,
  `planet::column::tests::a_dusk_lit_lantern_lights_only_at_night_and_a_torch_always`)

#### Scenario: They go out at dawn
- **WHEN** the clock passes dawn
- **THEN** the dusk-lit lanterns no longer light the ground, and an
  always-lit torch beside them still does
  (`planet::column::tests::a_dusk_lit_lantern_lights_only_at_night_and_a_torch_always`)
