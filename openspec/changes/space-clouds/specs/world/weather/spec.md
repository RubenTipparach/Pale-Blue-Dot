## ADDED Requirements

### Requirement: Puffs come in many sizes and gather in clumps
The puffs' sites SHALL give their puffs a wide spread of sizes, many small
and a few large, and under a partial cover the sites that carry cloud SHALL
gather into groups rather than scatter evenly, while the share of sites that
carry cloud SHALL still follow the cover.

#### Scenario: Sizes spread
- **WHEN** every site's puff is laid out under one cover
- **THEN** the 90th percentile radius is more than two and a half times the 10th
  (`cloud_puffs::tests::puffs_come_in_many_sizes`)

#### Scenario: Puffs gather
- **WHEN** puffs are laid out under a cover of 0.3
- **THEN** a carrying site's nearest neighbours carry cloud far more often than three times in ten
  (`cloud_puffs::tests::puffs_gather_in_clumps`)

### Requirement: Thick cloud is opaque from above
A puff over a column of the shown maps' median optical depth SHALL be drawn
at least three quarters opaque. A puff whose ray goes on to the ground or
the sea SHALL be faded only by the share of the air in front of it that the
air behind it, between it and that surface, would also hide; a puff with the
sky behind it SHALL fade into the sky as before.

#### Scenario: A median column
- **WHEN** a site under full cover and an optical depth of 7 is laid out with the shipped settings
- **THEN** its puff's opacity is at least 0.75
  (`config::tests::a_median_cloud_column_is_mostly_opaque`)

#### Scenario: The shader fades a cloud over the sea by the air it does not share
- **WHEN** the shipped `water.wgsl` is read
- **THEN** the puff fragment measures `cloud_air` from the puff to the ray's stop and scales its haze by it only where that stop is short of the far cap
  (`shader_tests::a_puff_over_a_surface_fades_only_by_the_air_it_does_not_share`)
