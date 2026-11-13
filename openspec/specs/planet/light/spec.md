# Planet Light Specification

## Purpose
The light field and everything that feeds or reads it: the sky channel that
darkens caves, the block channel that lamps fill, which materials emit and
when. The field is baked on the CPU over the column tier and drawn by the GPU;
nothing reads it back. Synced from `voxel-light`, `night-and-lamps`,
`lamps-and-lanterns` and `baked-light-ease`, each requirement with the test
that pins it.

## Requirements

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

### Requirement: Some flowers glow at night, and the CPU decides which
A share of flowers SHALL be a bioluminescent species that is a dusk-lit
emitter. Which cells carry one SHALL be decided once on the CPU and carried
in the cell record, never rolled independently by the shader.

#### Scenario: A meadow glows at night
- **WHEN** night falls over a meadow carrying the glowing species
- **THEN** the ground near those cells is lit, and by day it is not
  (`planet::column::tests::a_glowing_flower_lights_its_sod_at_night_and_is_put_out_by_digging_it`,
  `flora::tests::the_share_that_glows_is_the_chance`)

#### Scenario: The shader and the bake agree
- **WHEN** the shader draws a glowing flower head
- **THEN** it is on a cell the bake treated as an emitter
  (`planet::column::tests::a_tiers_glowing_flowers_agree_between_record_bake_and_bit`,
  `planet::terrain::tests::the_shader_carries_the_reference_light_constants`)

### Requirement: Baked light eases from the bake before it
When a baked light replaces the one before it - the lamp bake, a town's
outside, a field-lit thing's corners under a new field, or the light field
across dusk and dawn - every surface it lights SHALL show the old bake
blended to the new over `BAKE_EASE_S` by a smoothstep of the time since the
swap, and SHALL NOT change by more than a twentieth of the step between two
frames at 60 Hz.

#### Scenario: The rule
- **WHEN** a bake is swapped in
- **THEN** its share is zero on the swap and one from `BAKE_EASE_S` on,
  rising between, and no 60 Hz frame moves it more than a twentieth
  (`light::tests::a_new_bake_eases_in_and_never_steps`)

#### Scenario: The lamp bake is baked again
- **WHEN** a lamp bake made after a relight lands
- **THEN** it blends in by the rule over the one before it
  (`planet::tests::a_landing_lamp_bake_is_whole_at_once_and_a_relit_one_blends`)

#### Scenario: A town is baked again
- **WHEN** a town's outside finishes a bake, or the tier stops reaching it
- **THEN** its walls blend from the light they carried to the new light, and
  a bake finished mid-blend starts from what was on screen
  (`field_light::tests::a_towns_new_bake_keeps_the_one_before_and_blends_from_it`,
  `field_light::tests::a_towns_row_carries_its_blend`)

#### Scenario: Dusk
- **WHEN** the clock crosses dusk and the field is relit with the dusk-lit
  lanterns burning
- **THEN** the light field blends from the field before the relight to the
  one after over `BAKE_EASE_S`, not in a frame
  (`planet::tests::only_a_dusk_relight_blends_the_light_field`)

#### Scenario: A ship under a new field
- **WHEN** the field changes under a field-lit mesh that has not moved
- **THEN** its corners blend from what they showed to the new field
  (`field_light::tests::corners_blend_from_what_they_showed`)

### Requirement: A landing brings its own baked light
A fine set laid round a new anchor SHALL land with its sun's bounce and its
lamp bake already made, so the frame it lands draws them, and SHALL NOT draw
a frame without the bounce or with the lamp flood standing in for the bake.

#### Scenario: Walking at night past lanterns
- **WHEN** a set round a new anchor lands while lanterns are in reach
- **THEN** the lamp-lit ground keeps its baked light through the landing frame
  (`planet::bounce::tests::a_landing_set_brings_its_bounce_and_lamps`,
  `planet::tests::a_landing_lamp_bake_is_whole_at_once_and_a_relit_one_blends`)

#### Scenario: Walking by day past a cliff
- **WHEN** a set round a new anchor lands
- **THEN** the shaded faces keep their bounce through the landing frame
  (`planet::bounce::tests::a_landing_set_brings_its_bounce_and_lamps`)

### Requirement: An edit's light is not delayed
A dig, a placed block or a placed or removed lamp SHALL change the field's
light on its own frame, without easing, so a dug face never flashes dark.

#### Scenario: An edit during a dusk blend
- **WHEN** an edit lands while the light field is blending across dusk
- **THEN** the edit's field is drawn whole on that frame and the blend ends
  (`planet::tests::only_a_dusk_relight_blends_the_light_field`)
