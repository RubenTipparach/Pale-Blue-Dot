## ADDED Requirements

### Requirement: A room face on a band's edge takes its light whole
A building's room light SHALL light the band of height its data names
widened by `BAND_SLACK_M` (5 cm) on both edges, so that a room face built on
an edge is lit by it everywhere or nowhere, never pixel by pixel.

#### Scenario: The shader keeps the slack
- **WHEN** `field_lit.wgsl`'s `room_lights` tests a light's band
- **THEN** it keeps the light over `span.x - BAND_SLACK .. span.y +
  BAND_SLACK` and `BAND_SLACK` is `pbd_core::settlement::pieces::BAND_SLACK_M`
  (`field_light::tests::the_band_slack_is_the_cores`)

#### Scenario: No level room face lies on an edge
- **WHEN** Holbrook is laid at the game's radius and every level room face
  in a light's reach is measured as the shader measures it, in f32
- **THEN** none lies within a centimetre of its band's widened edges
  (`towns::tests::no_room_face_lies_on_a_lights_band_edge`)
