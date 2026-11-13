## ADDED Requirements

### Requirement: Each puff is hazed to its own distance
When the puffs draw the clouds, each puff SHALL be faded by the air between
the eye and that puff before it is laid over the puffs behind it, and the
composite SHALL NOT fade the puffs again by one distance for the whole texel.
A far puff SHALL keep its own haze inside a near puff's silhouette. The
distances the silhouette tests read SHALL stay the nearest puff's.

#### Scenario: A near puff in front of a far one
- **WHEN** a puff 800 m out lies in front of one 1600 m out, seen from the ground through the shipped haze
- **THEN** the far puff is drawn at the same strength just inside and just outside the near puff's footprint, and no line is drawn along the near puff's silhouette
  (`docs/screenshots/puff-haze/`, north and south shots)

#### Scenario: The shader hazes each puff and the composite only the march
- **WHEN** the shipped `water.wgsl` is read
- **THEN** the puff vertex measures `cloud_air` to its corner, the puff fragment multiplies its light and coverage by it, and the composite applies `cloud_air` only when the puffs do not draw the clouds
  (`shader_tests::each_puff_is_hazed_once_to_its_own_distance`)
