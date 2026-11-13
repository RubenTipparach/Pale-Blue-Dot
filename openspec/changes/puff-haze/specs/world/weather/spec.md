## ADDED Requirements

### Requirement: Each puff is hazed to its own distance
When the puffs draw the clouds, each puff SHALL be faded by the air between
the eye and that puff, and SHALL hide the puffs behind it by its own coverage,
not by its faded coverage. The composite SHALL NOT fade the puffs again by one
distance for the whole texel. A far puff SHALL keep its own haze inside a near
puff's silhouette. The distances the silhouette tests read SHALL stay the
nearest puff's.

#### Scenario: A near puff in front of a far one
- **WHEN** a puff 800 m out lies in front of one 1600 m out, seen from the ground through the shipped haze
- **THEN** the far puff is drawn at the same strength just inside and just outside the near puff's footprint, and no line is drawn along the near puff's silhouette
  (`docs/screenshots/puff-haze/`, the north and south shots, and the high shot at the limb)

#### Scenario: Inside an opaque near puff nothing changes
- **WHEN** the near puff's coverage is whole, or only one puff covers the pixel
- **THEN** the pixel is what the old one-distance haze drew, and as the near puff's coverage falls to nothing the pixel goes to the far puff's own, with no step
  (`shader_tests::the_puff_blend_is_the_air_over_each_puff_with_no_step`)

#### Scenario: The shader hazes each puff and the composite only the march
- **WHEN** the shipped `water.wgsl` is read
- **THEN** the puff fragment measures `cloud_air` to the puff along its own ray, writes its hazed light to the history and its hazed coverage to the puff cover, and the composite takes that coverage and applies `cloud_air` only when the puffs do not draw the clouds
  (`shader_tests::each_puff_is_hazed_once_to_its_own_distance`)
