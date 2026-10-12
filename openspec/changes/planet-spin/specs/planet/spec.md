# Planet: the spin and the sky

## ADDED Requirements

### Requirement: The sky turns with one spin

The sun, the star field and the moon SHALL be fixed directions in the system
frame carried into the planet's frame by the inverse of one rotation about the
planet's pole, so that all of them cross the sky together as the planet turns.

#### Scenario: The stars move with the sun

- **WHEN** the clock advances a quarter of a day
- **THEN** the star field and the sun have both turned a quarter turn about
  the pole, in the same sense

#### Scenario: A day is forty-eight minutes

- **WHEN** 2880 seconds of wall time pass
- **THEN** the sun is back where it was

### Requirement: The sun is drawn

The sky SHALL draw a sun disc along the sun direction, occluded by the
planet's shadow and by cloud.

#### Scenario: Noon

- **WHEN** the sun is above the horizon at a point
- **THEN** a disc is visible in the sky there along the sun direction

#### Scenario: Night

- **WHEN** the sun is below the horizon at a point
- **THEN** no disc is drawn there
