# Weather Specification

## ADDED Requirements

### Requirement: One rain intensity drives every rain effect
The engine SHALL hold one rain intensity in `0..1` and one wetness that follows
it on a configured time constant. The water surface ripples, the terrain
wetness, the lens droplets and the precipitation SHALL all read those two
values and SHALL NOT keep a rain state of their own.

#### Scenario: Rain stops
- **WHEN** the rain intensity drops to zero
- **THEN** the streaks and the lens droplets stop with it
- **AND** the ground stays wet and dries over the configured time constant

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
