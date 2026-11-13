## ADDED Requirements

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

#### Scenario: A town is baked again
- **WHEN** a town's outside finishes a bake, or the tier stops reaching it
- **THEN** its walls blend from the light they carried to the new light, and
  a bake finished mid-blend starts from what was on screen

#### Scenario: Dusk
- **WHEN** the clock crosses dusk and the field is relit with the dusk-lit
  lanterns burning
- **THEN** the light field blends from the field before the relight to the
  one after over `BAKE_EASE_S`, not in a frame

#### Scenario: A ship under a new field
- **WHEN** the field changes under a field-lit mesh that has not moved
- **THEN** its corners blend from what they showed to the new field

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

#### Scenario: An edit during a dusk blend
- **WHEN** an edit lands while the light field is blending across dusk
- **THEN** the edit's field is drawn whole on that frame and the blend ends
