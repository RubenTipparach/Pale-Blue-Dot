# Weather: the lens in rain and in cloud

## ADDED Requirements

### Requirement: Rain drops on the lens are small

Rain drops on the lens SHALL be drawn at `rain_lens_scale` times smaller than
the Tenebris drop space, their trails, beads and refraction scaled with them,
so each drop bends the same share of its own width.

#### Scenario: Standing in rain

- **WHEN** the walker stands in rain at 1440x900
- **THEN** the largest falling drop on the lens is about 30 px across at the default scale, not about 55 px

### Requirement: The lens fogs in cloud

The lens SHALL mist over while the eye is inside cloud, driven by the same
cloud density the renderer draws, building over `lens_mist_fog_s` and
clearing over `lens_mist_clear_s` (both faster with airspeed), clearing from
the edges inward, with rain drops cutting clear tracks through it, and none
under water.

#### Scenario: Flying through a cloud

- **WHEN** the camera flies into a cloud and out again
- **THEN** the lens mists over within about a second, and clears from its edges after leaving

#### Scenario: Between puffs of broken cloud

- **WHEN** the camera flies through clear air between separate clouds
- **THEN** the lens does not mist
