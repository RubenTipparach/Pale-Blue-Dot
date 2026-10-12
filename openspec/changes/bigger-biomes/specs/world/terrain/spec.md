## MODIFIED Requirements

### Requirement: The biome is a classification the surface reads
A cell's biome SHALL be one pure function of its direction and the world's
generator version (ocean, beach, tundra, mountains, desert, swamp, jungle,
fields), and the top block and the tree density SHALL both read it. The
moisture field that separates the temperate biomes SHALL be land-scale, with
a feature size the owner chose on the world map, so a biome is a region a
player can travel across and not a patch they pass through. No one temperate
biome SHALL hold the majority of the temperate land.

#### Scenario: A beach is sand
- **WHEN** a cell classifies as beach
- **THEN** its top block is sand

#### Scenario: Every biome occurs
- **WHEN** the sphere is sampled
- **THEN** every biome and at least six top materials occur on it

#### Scenario: A walk crosses biomes
- **WHEN** a kilometre of land is walked in a straight line
- **THEN** it passes through more than one biome

#### Scenario: The biomes are about four times wider
- **WHEN** kilometres of temperate land are walked in straight lines
- **THEN** they cross at most a third as many biome edges as the same
  generator with the 188 m moisture field does at the same thresholds

#### Scenario: Rock follows the contours
- **WHEN** a version 5 desert is sampled
- **THEN** every cell at one height has the same top block, so the rock lies
  in bands along the contours rather than along the latitude
- **AND** version 4's desert keeps the top blocks it had

#### Scenario: Grass is not the majority
- **WHEN** the temperate land (outside the cold band and below the mountain
  elevation) is sampled on the shipped seed and on four other seeds
- **THEN** fields, desert, and jungle with swamp each hold at least a fifth of
  it, and none holds more than half
