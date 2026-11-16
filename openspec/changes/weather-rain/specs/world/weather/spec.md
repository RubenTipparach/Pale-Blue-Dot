# Weather Specification

## ADDED Requirements

### Requirement: Rain lands on the surface and never under it
Precipitation SHALL fall from a column above the camera to the terrain or the
sea surface, whichever is higher, SHALL NOT be drawn below the waterline or
inside terrain, and SHALL NOT be drawn while the camera is under water or
below the ground.

#### Scenario: Standing on the shore in rain
- **WHEN** the camera stands on a beach at eye height with rain at full
- **THEN** streaks end at the sand on land and at the sea surface over water
- **AND** the water surface shows impact rings and the sand a wet sheen

### Requirement: Rain wets what the sky can see
Terrain wetness SHALL be gated by baked sky light and by being above the sea
radius, so a fragment the sky cannot reach and a submerged seabed stay dry.

#### Scenario: A cliff face in rain
- **WHEN** a vertical face is viewed in rain
- **THEN** rivulets run down it and it darkens
- **AND** the seabed below the waterline shows no rings

### Requirement: The lens wets and dries over time
The drops on the lens SHALL follow a lens wetness that rises and falls
exponentially, each way on its own time constant in `weather.ron`; while no
rain reaches the lens no new drop SHALL appear, and the drops there SHALL
shrink, thin and fade rather than vanish in one frame.

#### Scenario: Walking under cover
- **WHEN** the camera walks from rain under a roof
- **THEN** the drops on the lens dry off over a few seconds
- **AND** walking back out they build up again over a few seconds

### Requirement: Rain streaks take the scene's light
Under the new lighting model, the near shower and the cell shafts SHALL be
lit by the sky's light on a standing face at the camera, so a streak is not
brighter than the ground it falls in front of by more than its own colour.

#### Scenario: A storm at noon
- **WHEN** it rains under a full deck at noon
- **THEN** the streaks read as grey-blue rain over the ground, not as bright lines
