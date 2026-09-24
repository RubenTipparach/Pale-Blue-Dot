# Weather Specification

## ADDED Requirements

### Requirement: Wind at a point is the field's wind, sheared and gusting
The wind acting on a body SHALL be the atmosphere's wind at its direction,
scaled by a log-law profile that is 1 at 10 m above the local surface, plus
gusts that are a pure function of position, world time and the local gust
strength, so that equal inputs give equal gusts on every client.

#### Scenario: Two clients, one gust
- **WHEN** two processes evaluate the wind at the same point, world time and
  atmosphere state
- **THEN** they return the same vector bit for bit

#### Scenario: Gusts average out
- **WHEN** the wind at a fixed point is averaged over ten minutes of world time
- **THEN** the mean is the sheared field wind within 2 %

#### Scenario: Near the water
- **WHEN** the wind is sampled 1 m above the sea
- **THEN** it is weaker than at 10 m by the log-law ratio for the sea's
  roughness length

### Requirement: Rain brings a downdraft and stronger gusts
Where rain falls, the wind SHALL gain a downward component that grows with the
square root of the rain rate, and the gust strength SHALL grow with the rain
rate.

#### Scenario: A hover under a squall
- **WHEN** a craft hovers where the rain rate is 90 mm/h
- **THEN** the air at the craft sinks at about 2.8 m/s on average
