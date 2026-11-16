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

### Requirement: With RAIN SHADOW off, rain still fogs the air
While the rain volume is off (the player's RAIN SHADOW setting), rain over
the camera on a planet with air SHALL draw an exponential height fog in its
place, measured along the body's local up from the ground or sea under the
eye, coloured by the sky's light at that hour, and thickening with the rain.
It SHALL add no full-screen pass. While the rain volume is on, the fog SHALL
NOT draw.

#### Scenario: A storm with RAIN SHADOW off
- **WHEN** the rain is at full and RAIN SHADOW is off
- **THEN** the far side of a valley is lost in grey murk
- **AND** the murk is thicker in the valley than at the hilltop

#### Scenario: Clear air
- **WHEN** there is no rain
- **THEN** the picture is as it was before the fog existed
