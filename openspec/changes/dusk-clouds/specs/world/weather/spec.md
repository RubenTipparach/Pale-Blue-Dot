## ADDED Requirements

### Requirement: Clouds take the sky's twilight
A cloud SHALL be lit by the same sun and sky the dome and the ground are: the
sun after the air at the cloud's own sun elevation, raised for its height as
the dome raises its upper sky and gone by the dome's twilight, and the sky's
own light at that elevation and cover. Past the sun's light the cloud SHALL
be no brighter than the sky's light makes it, and at night its floor SHALL
be the ground's.

#### Scenario: The sky term follows the sky
- **WHEN** the cloud's sky light is compared with the sky's mean light over the upper hemisphere, every two degrees of sun elevation, clear and overcast
- **THEN** it is within a quarter of it
  (`sky_light::tests::the_cloud_sky_follows_the_sky`)

#### Scenario: Tops keep the last light
- **WHEN** the sun goes down
- **THEN** a cloud's top is sunlit after its base is not, no part of it is sunlit by seven degrees under, and at noon the sun on it is within a tenth of white
  (`sky_light::tests::the_cloud_sun_is_the_domes_twilight`)

#### Scenario: Puffs and the march light alike
- **WHEN** the shipped shaders are read
- **THEN** the puffs and `cloud_light` both light a cloud by `cloud_sun` and `cloud_sky`, whose constants are the core's
  (`shader_tests::clouds_are_lit_by_the_cores_sun_and_sky`)

### Requirement: Cloud hides the stars
Where the puffs lie in front of the sky, the scene behind them SHALL be
hidden by the cloud's own coverage, and the air in front of the cloud SHALL
add only the sky's smooth light, never the stars'.

#### Scenario: The composite takes the sky without its stars
- **WHEN** the shipped `water.wgsl` is read
- **THEN** under the puffs, with the sky behind, the composite hides the scene by the puffs' own coverage and adds the dimmest of the pixel and its sky neighbours for the air's light
  (`shader_tests::cloud_hides_the_stars`)
