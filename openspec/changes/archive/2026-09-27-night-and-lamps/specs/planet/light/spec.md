# Planet: a night, and light that is not the sun

What this change built. The torch's icon, the glowing flowers and the sampler
were not built here; `lamps-and-lanterns` carries and builds them.

## ADDED Requirements

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
three levels a cell across, about a metre a level either way
(`lamps-and-lanterns` decision 7).

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
