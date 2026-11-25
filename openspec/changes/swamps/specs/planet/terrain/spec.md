## ADDED Requirements

### Requirement: Swamps stand on the low wet land in version 12

In generator version 12, temperate land SHALL be Swamp where it is low for
how wet it is by the sea (up to 12 m over the sea where wettest, none below
moisture 0.47) or stands within 40 m of a river's banks no more than 3 m
over the river's water, so that swamps hold several percent of the land.
Version 12 SHALL move no height, and worlds made on earlier versions SHALL
keep their ground.

#### Scenario: Swamps are a real share of the land

- **WHEN** a version 12 world is sampled on 200,000 directions on the
  shipped seed and two others
- **THEN** the swamp holds between 4% and 9% of the land, and none of it in
  the cold band
  (`planet_gen::tests::swamps_hold_several_percent_of_the_land`)

#### Scenario: A version 11 swamp stays a swamp

- **WHEN** a direction is Swamp in version 11
- **THEN** it is Swamp in version 12, and every altitude is version 11's bit
  for bit
  (`planet_gen::tests::version_11_is_version_10_s_ground_with_its_swamps`)

#### Scenario: Swamp ground holds its water

- **WHEN** a version 12 swamp cell's top is water
- **THEN** no ground round it, at any of the cells' widths, stands lower,
  so the pool is held; and the swamp's tops are moss, water and mud, moss
  the most of them
  (`planet_gen::tests::a_swamp_is_moss_pools_and_mud`)

#### Scenario: A fresh tier by a river holds its swamp's water

- **WHEN** a column tier is built by a river in a version 12 world
- **THEN** no water flows outside the river's banks: the swamp's pools are
  held and no cave opens under a swamp's top 6 m
  (`planet::column::river_tier_tests::a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`)

#### Scenario: An old world

- **WHEN** a world made on version 11 is opened
- **THEN** its heights, biomes and top blocks are the same, cell for cell,
  as before version 12
  (`planet_gen::tests::version_10_keeps_its_ground`)

#### Scenario: Sites and the climate do not move

- **WHEN** the city survey or the climate reads a version 12 world's ground
- **THEN** they read it without its swamps, so its site list and its settled
  climate are version 11's
  (`planet_gen::tests::sites_and_climate_read_version_11_without_its_swamps`)
