# World: cloud detail

## ADDED Requirements

### Requirement: Humid air is cloudy without rising
The atmosphere SHALL give a cell partial cloud cover from its relative humidity
alone, above a critical humidity, as well as from condensed water, and SHALL
lift air over ground warmer than the air above it. Cloud that comes from
humidity alone SHALL NOT rain, and the water budget SHALL be unchanged.

#### Scenario: Cloud away from the rain belt
- **WHEN** the climate report is run for a day at the northern solstice
- **THEN** the whole planet's mean cover is between 0.5 and 0.65
- **AND** the northern storm track (40-60 N) averages at least 0.3
- **AND** at least 30% of the planet is partly covered

#### Scenario: Land clouds over in the afternoon
- **WHEN** rain over land is binned by local hour over a day, at either solstice
- **THEN** more of it falls between 12:00 and 18:00 than between 00:00 and 06:00

### Requirement: Clouds carry detail finer than the simulation's cells
The clouds SHALL draw structure finer than the atmosphere's cells: a fully
covered area SHALL NOT be uniformly opaque, detail SHALL be drawn out along the
wind at cloud height, and convective cloud SHALL look cellular where
stratiform cloud looks smooth.

#### Scenario: A storm system from orbit has texture
- **WHEN** a fully covered area is captured from orbit
- **THEN** the spread of its cloud pixels' brightness is at least twice what
  it was before this change
