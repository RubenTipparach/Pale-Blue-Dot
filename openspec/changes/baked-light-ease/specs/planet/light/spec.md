## ADDED Requirements

### Requirement: Baked light eases from the bake before it
When a baked light replaces the one before it - the sun's bounce, the lamp
bake, a town's outside, a field-lit thing's corners, or the field across dusk
and dawn - every surface it lights SHALL show the old bake blended to the new
over `BAKE_EASE_S` by a smoothstep of the time since the swap, and SHALL NOT
change by more than a twentieth of the step between two frames at 60 Hz.

#### Scenario: The sun moves two degrees
- **WHEN** the sun's bounce is baked again for a sun two degrees on
- **THEN** a shaded wall's bounce blends from the old bake to the new over
  `BAKE_EASE_S`, and no frame shows the new one whole the frame after the old

#### Scenario: A town is baked again
- **WHEN** a town's outside finishes a bake, or the tier stops reaching it
- **THEN** its walls blend from the light they carried to the new light, and
  the bake they carried is kept until the blend is done

#### Scenario: Dusk
- **WHEN** the clock crosses dusk near a row of dusk-lit lanterns
- **THEN** the ground round them brightens over `DUSK_EASE_S`, not in a frame

### Requirement: A landing brings its own baked light
A fine set laid round a new anchor SHALL land with its sun's bounce and its
lamp bake already made, so the frame it lands draws them, and SHALL NOT draw
a frame without the bounce or with the lamp flood standing in for the bake.

#### Scenario: Walking at night past lanterns
- **WHEN** a set round a new anchor lands while lanterns are in reach
- **THEN** the lamp-lit ground keeps its baked light through the landing frame

#### Scenario: Walking by day past a cliff
- **WHEN** a set round a new anchor lands
- **THEN** the shaded faces keep their bounce through the landing frame

### Requirement: An edit's light is not delayed
A dig, a placed block or a placed or removed lamp SHALL change the field's
light on its own frame, without easing, so a dug face never flashes dark.

#### Scenario: A lantern placed at night
- **WHEN** the player places a lantern in the dark
- **THEN** the light round it is there on that frame
