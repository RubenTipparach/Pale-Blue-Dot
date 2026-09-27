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
