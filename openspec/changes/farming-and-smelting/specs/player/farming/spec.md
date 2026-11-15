## ADDED Requirements

### Requirement: The shovel tills soil
With the shovel in hand, the use key on a grass, dry grass, jungle grass,
dirt, soil or sand block with air over it SHALL turn that block into tilled
soil, as an edit on the durable path. Snow, stone, rock and ore SHALL NOT
till. Tilling a grassy block SHALL turn up one seed of a crop that suits the
cell's climate, the same crop for the same cell every time, as a drop saved
on the edit's own line.

#### Scenario: Tilling a meadow cell
- **WHEN** the player holds the shovel and uses a grass cell on a temperate
  meadow
- **THEN** the cell's top is tilled soil, and a seed of a temperate crop
  floats out of it

#### Scenario: Tilling rock
- **WHEN** the player uses a rock cell with the shovel in hand
- **THEN** nothing changes

### Requirement: A crop is planted on tilled soil where its climate suits
A seed in hand and the right button on tilled soil with air over it SHALL
plant that seed's crop, spending one seed, if the cell's climate is one the
crop grows in, and SHALL refuse otherwise. The plot and the slots after it
SHALL reach the save in one line.

#### Scenario: Wheat in the tundra
- **WHEN** the player plants wheat seed on tilled soil in the tundra
- **THEN** nothing is planted and the seed stays in hand

### Requirement: A crop grows on world time
A crop's stage SHALL be derived from the world time it was planted, the world
time now and its days to ripe: sown below a quarter of the way, a sprout
below 0.6, growing below one, ripe from one. Growing SHALL write nothing, and
SHALL pass only while the world is played.

#### Scenario: A wheat crop after three game days
- **WHEN** wheat (6 days) was planted 3 game days ago
- **THEN** it is a sprout, and at 6 days it is ripe

### Requirement: A ripe crop is harvested with the use key
The use key on a ripe crop SHALL give its produce and its seeds by the give
order and clear the plot, the soil staying tilled, in one saved line. On an
unripe crop it SHALL say what is growing and how long it has left.

#### Scenario: Harvesting wheat
- **WHEN** the player uses a ripe wheat crop
- **THEN** three wheat and two wheat seed go into the slots and the cell is
  bare tilled soil

### Requirement: Crops are drawn as faded, field-lit sprites
A planted crop within reach of sight SHALL be drawn as crossed sprites of its
stage, nearest-sampled and alpha-tested, lit by the same field, sun and lamps
as the ground, and SHALL fade in, out and between stages rather than appear or
vanish in one frame.

#### Scenario: A field at night
- **WHEN** a field stands beside a lit lantern at midnight
- **THEN** the crops near the lantern are lit by it and those far from it are
  dark
