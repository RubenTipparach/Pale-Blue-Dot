# World: fair-weather clouds

## ADDED Requirements

### Requirement: Thin cover draws scattered small clouds
The clouds SHALL fill a share of the sky that tracks the simulated cover, so
that thin cover draws as scattered small clouds rather than as clear sky, and
air humid enough for fair-weather cumulus SHALL carry a thin cover that rains
nothing.

#### Scenario: Thin cover is visible
- **WHEN** the cloud shader's density is evaluated at a cover of 0.1, 0.2 or 0.3
- **THEN** the share of columns it draws is within 0.05 of that cover

#### Scenario: Few bald spots
- **WHEN** the climate report is run for a day at either solstice
- **THEN** less than 1% of the planet has no cover at all
- **AND** the share of the planet raining is unchanged within half a point
