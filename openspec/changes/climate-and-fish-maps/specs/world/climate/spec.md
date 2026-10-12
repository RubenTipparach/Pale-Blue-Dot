# World: climate

## Purpose

What a place on the planet is like across the year, as opposed to what the
weather is doing now: normals measured from a year of the planet's own
atmosphere, shipped as a derived asset whose provenance is checked against
the world that reads it.

## ADDED Requirements

### Requirement: The climate is a year of the planet's own weather

The climate normals SHALL be measured by running the shipped atmosphere
forward for at least one full year from a new world, with the shipped
settings. They SHALL hold, per place:
- the mean temperature;
- the warmest and the coldest daily-mean temperature;
- the yearly rain;
- whether the water there freezes all year, for part of it, or never;
- the same per season.

They SHALL NOT be authored or smoothed by hand.

#### Scenario: The instrument and the asset agree

- **WHEN** the climate instrument is run with the shipped settings
- **THEN** it writes the asset that is committed, byte for byte

#### Scenario: A frozen coast

- **WHEN** a place's coldest daily-mean water temperature in the run is below
  freezing
- **THEN** the normals mark it as freezing for at least part of the year

### Requirement: The climate asset carries its provenance

The asset SHALL record the seed, the generator version and a digest of the
atmosphere settings it was made from. A world whose seed or generator version
differs SHALL NOT be shown the asset. A world whose atmosphere settings
differ SHALL be shown it marked as made before the weather was last retuned.

#### Scenario: Another world

- **WHEN** a world with a different seed or generator version opens the map
- **THEN** the climate layer is unavailable, and the legend says why

#### Scenario: A retune

- **WHEN** the shipped atmosphere settings change and the asset is not remade
- **THEN** the climate layer is shown with a notice that it predates the retune,
  and a test lists the asset as stale
