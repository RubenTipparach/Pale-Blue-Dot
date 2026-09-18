## ADDED Requirements

### Requirement: The terrain generator is Tenebris's
The surface altitude SHALL be Tenebris's generator ported term for term at its
unit-sphere scales: a sharpened continent field, ridged mountains standing on
land, hills, detail, an ocean floor that deepens past the beach, islands,
rivers that reach the sea, eased shorelines and rocky highlands, with every
height authored in metres against this body's budget and the noise primitive
pinned bit for bit against the reference.

#### Scenario: Mountains stand on land
- **WHEN** a cell is above the mountain elevation
- **THEN** no cell within three cells of it is below sea level

#### Scenario: Rivers reach the sea
- **WHEN** a cell is a river channel
- **THEN** a downhill walk from it ends below sea level

#### Scenario: The budget holds
- **WHEN** the sphere is sampled
- **THEN** the summit is between 130 and 180 m, the floor between 60 and
  110 m down, and land is between 40 and 60 percent of it

### Requirement: The biome is a classification the surface reads
A cell's biome SHALL be one pure function of its direction (ocean, beach,
tundra, mountains, desert, swamp, jungle, fields), and the top block and the
tree density SHALL both read it.

#### Scenario: A beach is sand
- **WHEN** a cell classifies as beach
- **THEN** its top block is sand

#### Scenario: Every biome occurs
- **WHEN** the sphere is sampled
- **THEN** every biome and at least six top materials occur on it
