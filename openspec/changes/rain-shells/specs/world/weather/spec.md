## ADDED Requirements

### Requirement: Rain round the eye is drawn as streak shells
Rain seen from under the cloud base SHALL be drawn as streak textures on a few
cylinders centred on the eye, falling at the rain's speed, with no march along
the ray. Each shell SHALL read the rain map where it stands, fade into the
cloud base at its top and into the ground or the sea at its bottom, and take
the air's haze with distance.

#### Scenario: Under a storm, looking along the ground
- **WHEN** the eye is under the cloud base in rain
- **THEN** streaks fall at every shell, the far ones finer in angle than the near
- **AND** where a shell meets the ground there is no hard line

#### Scenario: A storm off to one side
- **WHEN** it rains 1 km away and is dry at the eye
- **THEN** only the shells that reach the storm show rain, and only in its direction

#### Scenario: Flying through rain
- **WHEN** the eye moves fast through rain
- **THEN** the streaks slant and pass it, and do not travel with it
