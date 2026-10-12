# Planet: orbit and seasons

## ADDED Requirements

### Requirement: The planet orbits its sun
The clock SHALL carry world time, and the sun's latitude SHALL follow the
planet's orbit: the obliquity north at the northern solstice, over the equator
at the equinoxes, and the obliquity south at the southern solstice, over a year
of a configured number of days. Noon SHALL stay at the middle of every day.

#### Scenario: The solstices and equinoxes
- **WHEN** the clock reaches day 0, 25, 50 and 75 of a 100-day year
- **THEN** the sun stands over 23.45 N, the equator, 23.45 S and the equator

#### Scenario: A year of days
- **WHEN** a whole year passes
- **THEN** the sun and the stars are back where they were
- **AND** the planet has turned one more time against the stars than there
  are days

### Requirement: A world resumes in its season
The world time SHALL be saved with the world and restored on load.

#### Scenario: Reopening a world
- **WHEN** a world saved on day 40 is opened
- **THEN** its clock reads day 40
