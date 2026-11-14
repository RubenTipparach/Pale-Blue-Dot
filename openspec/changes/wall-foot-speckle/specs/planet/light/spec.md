## ADDED Requirements

### Requirement: A face on a layer boundary reads one layer of baked light
The baked lamp light and the baked sun bounce SHALL be sampled from the
same layers at every pixel of a floor or a ceiling: an altitude within
`LAYER_SNAP_M` (1 cm) of a whole layer is taken as that layer, so the f32
step at the planet's radius never flips a face between the layer under it
and the layer over it.

#### Scenario: Both samplers snap
- **WHEN** `planet_surface.wgsl`'s `lamp_near` and `bounce_near` turn a
  point half a metre off a face into a layer
- **THEN** both take the altitude through `sample_layer`, whose snap is
  `LAYER_SNAP_M` = 0.01 m
  (`shader_tests::a_floor_reads_one_layer_of_baked_light`)

#### Scenario: A floor's sample lies within the snap
- **WHEN** floor points at the game's radius are offset half a metre up and
  measured as the shader measures them, in f32
- **THEN** their altitude straddles the whole layer (the flip is real) and
  every one lies within `LAYER_SNAP_M` of it
  (`shader_tests::a_floor_reads_one_layer_of_baked_light`)
