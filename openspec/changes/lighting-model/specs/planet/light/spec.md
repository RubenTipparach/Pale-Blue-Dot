## ADDED Requirements

### Requirement: Sunlight takes the colour of the air it came through
The sun's light on every surface SHALL be the sun's light after the
atmosphere between it and the ground, so it is warmer and dimmer as the sun
gets low, and SHALL match today's brightness at noon.

#### Scenario: Golden hour
- **WHEN** the sun stands 5 degrees above the horizon on a clear day
- **THEN** sunlit ground is lit warmer and dimmer than at noon

### Requirement: Shade is lit by the sky that is there
Light that does not come straight from the sun SHALL come from the sky's
current radiance in the direction the surface faces, plus the light the
sunlit ground reflects from below, times the field's sky channel.

#### Scenario: Blue sky, blue shade
- **WHEN** a face is in the shade under a clear sky
- **THEN** it is lit by the sky's colour, and an upward face more than a downward one

#### Scenario: Under a storm
- **WHEN** the cover over a face is full
- **THEN** its fill is the storm's grey, and its shadows are soft

#### Scenario: An overcast night
- **WHEN** the sun is down and the cover over a face is full
- **THEN** the face is lit by the cloud deck's even glow from the moon and the stars, softly and from above, and is not black away from any lamp
