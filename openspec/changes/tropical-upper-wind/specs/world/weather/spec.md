## ADDED Requirements

### Requirement: The wind at cloud height has no calm band at the equator
The wind at cloud height SHALL blow over the whole planet. Within the
tropics, where the jet's balance against the planet's spin does not hold, it
SHALL blow westward, against the planet's turn, at the configured tropical
easterly speed. It SHALL turn into the jets over a band of latitude, not at
an edge. Between neighbouring 2.5-degree bands within 35 degrees of the
equator, the day's zonal-mean speed SHALL change by no more than 3.5 m/s per
degree of latitude.

#### Scenario: The equator aloft
- **WHEN** the wind at cloud height is averaged over a day and round the
  planet, within 5 degrees of the equator, on a settled climate
- **THEN** it blows westward at no less than half the configured tropical
  easterly speed

#### Scenario: The jet's edge
- **WHEN** the day's zonal-mean speed of the wind at cloud height is taken in
  2.5-degree bands from 35 degrees south to 35 degrees north
- **THEN** no band differs from its neighbour by more than 3.5 m/s per degree

#### Scenario: No easterly asked for
- **WHEN** the tropical easterly speed is set to zero
- **THEN** the wind at cloud height within 5 degrees of the equator is the
  surface wind, and the jet's edge still changes by no more than 3.5 m/s per
  degree
