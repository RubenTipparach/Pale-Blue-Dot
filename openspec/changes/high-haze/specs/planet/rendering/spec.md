# Rendering: the haze over the ground from above the atmosphere

## ADDED Requirements

### Requirement: The haze over the ground grows continuously with height

The haze laid over the ground SHALL change continuously as the eye climbs from
the surface through the top of the atmosphere shell into space. No part of the
sky or atmosphere SHALL begin to draw over the ground in one frame at a fixed
height. Over the ground the atmosphere's in-scattered light SHALL tend to the
sky's own colour as its opacity grows, so ground seen through a thin column of
air keeps its colours and texture.

#### Scenario: Climbing through the shell's top

- **WHEN** the eye climbs from 850 m to 1000 m above the ground at noon in a clear sky, past the shell 960 m above the sea
- **THEN** the ground's colour changes by no more than it does between 750 m and 850 m, and its texture stays

#### Scenario: Ground seen from one to three kilometres

- **WHEN** the eye is 1-3 km above the ground at noon in a clear sky
- **THEN** the biomes, the coast and the sea read under a thin haze of the sky's colour, not a milky veil

#### Scenario: From orbit

- **WHEN** the eye is in orbit and looks at the planet's limb
- **THEN** the ground near the limb fades into the same blue as the sky just above it
