## ADDED Requirements

### Requirement: The terrain generator is Tenebris's
The surface altitude SHALL be Tenebris's generator ported term for term at its
unit-sphere scales: a sharpened continent field, ridged mountains standing on
land, hills, detail, an ocean floor that deepens past the beach, islands,
rivers that reach the sea, eased shorelines and rocky highlands, with every
height authored in metres against this body's budget.

#### Scenario: Mountains stand on land
- **WHEN** a cell is above the mountain elevation
- **THEN** no cell within the beach band's distance of it is below sea level

#### Scenario: Rivers reach the sea
- **WHEN** a cell is a river channel
- **THEN** a downhill walk from it ends below sea level

### Requirement: The biome is a classification the whole world reads
A cell's biome SHALL be one pure function of its direction (ocean, beach,
tundra, mountains, desert, swamp, jungle, fields) and the top block, the trees
and the scatter SHALL all read it.

#### Scenario: A jungle is a jungle in every layer
- **WHEN** a cell classifies as jungle
- **THEN** its top block, its tree roster and its scatter are the jungle's
