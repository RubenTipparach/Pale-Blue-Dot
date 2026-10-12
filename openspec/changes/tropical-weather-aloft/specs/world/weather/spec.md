## MODIFIED Requirements

### Requirement: The wind at cloud height has no calm band at the equator
The wind at cloud height SHALL blow over the whole planet. Within the
tropics, where the jet's balance against the planet's spin does not hold, it
SHALL blow westward, against the planet's turn, at the configured tropical
easterly speed. To that it SHALL add the tropics' own weather: the outflow
from air rising in their storms, and the wind round the high those storms
build aloft, each at its configured speed. So the tropics aloft SHALL vary
round the planet, not form a band of one speed. The jet SHALL be faded in
before it is capped, so that within 20 degrees of the equator it takes the
shape of the air's temperature and is rarely at its cap. It SHALL turn into
the jets over a band of latitude, not at an edge. Between neighbouring
2.5-degree bands within 35 degrees of the equator, the zonal-mean speed SHALL
change by no more than 3.5 m/s per degree of latitude.

#### Scenario: The equator aloft
- **WHEN** the wind at cloud height is worked out on a settled climate and
  averaged round the planet within 5 degrees of the equator
- **THEN** it blows westward at no less than half the configured tropical
  easterly speed

#### Scenario: The tropics aloft vary round the planet
- **WHEN** the speed of the wind at cloud height is taken round the planet
  along each latitude within 10 degrees of the equator, on a settled climate
- **THEN** its spread round the planet averages at least 3 m/s

#### Scenario: The jet is not pinned at its cap in the tropics
- **WHEN** the jet's part of the wind at cloud height is worked out on a
  settled climate
- **THEN** no more than 20% of cells within 5 to 20 degrees of the equator
  are at the cap, and poleward of 30 degrees every cell's wind is what the
  rule before this change gave

#### Scenario: The jet's edge
- **WHEN** the zonal-mean speed of the wind at cloud height is taken in
  2.5-degree bands from 35 degrees south to 35 degrees north
- **THEN** no band differs from its neighbour by more than 3.5 m/s per degree

#### Scenario: No easterly asked for
- **WHEN** the tropical easterly, outflow and circling speeds are all set to
  zero
- **THEN** the wind at cloud height within 5.7 degrees of the equator is the
  surface wind, and the jet's edge still changes by no more than 3.5 m/s per
  degree
