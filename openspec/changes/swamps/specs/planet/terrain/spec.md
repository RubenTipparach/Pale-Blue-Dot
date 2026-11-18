## ADDED Requirements

### Requirement: Swamps stand on the low wet land in version 11

In generator version 11, temperate land SHALL be Swamp where it is low for
how wet it is by the sea (up to 12 m over the sea where wettest, none below
moisture 0.47) or stands within 40 m of a river's banks no more than 3 m
over the river's water, so that swamps hold several percent of the land.
Version 11 SHALL move no height, and worlds made on earlier versions SHALL
keep their ground.

#### Scenario: Swamps are a real share of the land

- **WHEN** a version 11 world is sampled on 200,000 directions on the
  shipped seed and two others
- **THEN** the swamp holds between 4% and 9% of the land, and none of it in
  the cold band
  (`planet_gen::tests::swamps_hold_several_percent_of_the_land`)

#### Scenario: A version 10 swamp stays a swamp

- **WHEN** a direction is Swamp in version 10
- **THEN** it is Swamp in version 11, and every altitude is version 10's bit
  for bit
  (`planet_gen::tests::version_11_is_version_10_s_ground_with_its_swamps`)

#### Scenario: Swamp ground holds its water

- **WHEN** a version 11 swamp cell's top is water
- **THEN** no ground 2.9 m round it stands lower, so the pool is held; and
  the swamp's tops are moss, water and mud, moss the most of them
  (`planet_gen::tests::a_swamp_is_moss_pools_and_mud`)

#### Scenario: An old world

- **WHEN** a world made on version 10 is opened
- **THEN** its heights, biomes and top blocks are the same, cell for cell,
  as before version 11
  (`planet_gen::tests::version_10_keeps_its_ground`)

#### Scenario: Sites and the climate do not move

- **WHEN** the city survey or the climate reads a version 11 world's ground
- **THEN** they read it without its swamps, so its site list and its settled
  climate are version 10's
  (`planet_gen::tests::sites_and_climate_read_version_11_without_its_swamps`)
