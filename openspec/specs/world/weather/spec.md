# Weather Specification

## Purpose
Rain is one number and what the ground remembers of it is one more; every
effect rain has reads those two and keeps no state of its own.

## Requirements

### Requirement: One rain intensity drives every rain effect
The engine SHALL hold one rain intensity in `0..1` and one wetness that follows
it on a configured time constant. The water surface ripples, the terrain
wetness, the lens droplets and the precipitation SHALL all read those two
values and SHALL NOT keep a rain state of their own.

#### Scenario: Rain stops
- **WHEN** the rain intensity drops to zero
- **THEN** the streaks and the lens droplets stop with it
- **AND** the ground stays wet and dries over the configured time constant
