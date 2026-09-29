## MODIFIED Requirements

### Requirement: The biome is a classification the surface reads
A cell's biome SHALL be one pure function of its direction and the world's
generator version (ocean, beach, tundra, mountains, desert, swamp, jungle,
fields), and the top block and the tree density SHALL both read it. The
moisture field that separates the temperate biomes SHALL be land-scale, with
a feature size the owner chose on the world map, so a biome is a region a
player can travel across and not a patch they pass through. No one temperate
biome SHALL hold the majority of the temperate land, and the desert SHALL hold
the smaller share the owner chose (survey B6), the fields and the jungle
splitting the rest.

#### Scenario: A beach is sand
- **WHEN** a cell classifies as beach
- **THEN** its top block is sand
  (`planet_gen::tests::every_biome_and_material_occurs_and_the_beach_is_sand`)

#### Scenario: Every biome occurs
- **WHEN** the sphere is sampled
- **THEN** every biome and at least six top materials occur on it
  (`planet_gen::tests::every_biome_and_material_occurs_and_the_beach_is_sand`)

#### Scenario: A walk crosses biomes
- **WHEN** a kilometre of land is walked in a straight line
- **THEN** it passes through more than one biome
  (`planet_gen::tests::a_kilometre_of_land_crosses_more_than_one_biome`)

#### Scenario: The biomes are about four times wider
- **WHEN** kilometres of temperate land are walked in straight lines
- **THEN** they cross at most a third as many biome edges as the same
  generator with the 188 m moisture field does at the same thresholds
  (`planet_gen::tests::the_biomes_are_about_four_times_wider`)

#### Scenario: Rock follows the contours
- **WHEN** a version 5 desert is sampled
- **THEN** every cell at one height has the same top block, so the rock lies
  in bands along the contours rather than along the latitude
- **AND** version 4's desert keeps the top blocks it had
  (`planet_gen::tests::a_deserts_rock_follows_the_contours`,
  `planet_gen::tests::version_4_keeps_its_top_blocks`)

#### Scenario: Grass is not the majority
- **WHEN** the temperate land (outside the cold band and below the mountain
  elevation) is sampled on the shipped seed and on four other seeds
- **THEN** fields and jungle with swamp each hold more than a third of it,
  the desert holds the owner's share of it within a few points, and none
  holds half
