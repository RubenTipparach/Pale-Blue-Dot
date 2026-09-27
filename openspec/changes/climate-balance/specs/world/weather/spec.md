## ADDED Requirements

### Requirement: The sun holds the planet's average temperature

The area-weighted mean surface temperature of the whole planet, land and sea,
SHALL be held at a configured target, 15 °C as shipped, by scaling the sun's
strength. The scaling SHALL act on the planet's average only, leaving
latitude, season, day and night, and weather to the simulation. Its state
SHALL be saved with the weather, so a resumed world continues exactly. The
planet's mean net cloud effect SHALL lie between −10 and −30 W/m².

#### Scenario: Two game years from a new world

- **WHEN** a new world's atmosphere is run for 200 game days with the shipped
  settings
- **THEN** the mean surface temperature stays within 15 ± 1 °C from day 30 on

#### Scenario: Fish in the second year

- **WHEN** the water temperature is read over the second game year of that run
- **THEN** every species in the roster has water it can live in for at least
  part of the year

#### Scenario: A resumed world

- **WHEN** an atmosphere is saved mid-run and resumed
- **THEN** its sun's scaling and every field equal those of one stepped
  straight on

#### Scenario: A frozen world is opened

- **WHEN** a save whose weather has frozen over is opened on this build
- **THEN** its mean surface temperature rises toward 15 °C over the following
  game days

### Requirement: Heat is moved, never made or lost, inside the planet

Sunlight SHALL be the planet's only source of heat and the outgoing longwave
its only sink. Heat that moves between neighbouring cells, or between the
ground and the air, SHALL leave one place and arrive in the other in equal
amounts, each place's temperature changing by that heat over its own heat
capacity. So a coast, where a sea cell holds far more heat per kelvin than
the land beside it, SHALL NOT make or lose heat. The heat water takes to
evaporate SHALL be given back where it condenses.

#### Scenario: Land beside sea

- **WHEN** a grid of land and sea cells at different temperatures is stepped
  with only the spread between cells acting
- **THEN** its area-weighted heat total is unchanged to rounding
- **AND** a land cell beside the sea moves toward the sea's temperature
  further than the sea moves toward the land's

#### Scenario: Water evaporates here and rains there

- **WHEN** the heat budget is measured over a game day on the shipped
  settings
- **THEN** the heat evaporation takes from the ground and the heat
  condensation gives back to it differ by under 1 W/m² over the planet
