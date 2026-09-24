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

### Requirement: Weather is a deterministic field, not a stored state
Rain and cloud cover SHALL be pure functions of the body seed, a surface
direction and the elapsed time, evaluated on demand. No weather state SHALL be
stored, saved or synchronised, and the same direction at the same time SHALL
give the same weather on every run and every client.

#### Scenario: The same place at the same time
- **WHEN** the field is sampled twice for one direction and one time
- **THEN** both samples agree exactly

#### Scenario: Rain trails the cloud that made it
- **WHEN** a column's cover has just fallen below the rain threshold
- **THEN** it still counts as raining, because the field a trail-time earlier
  was over the threshold
- **AND** once that earlier sample is also under the threshold, it stops

### Requirement: Rain falls where it is overcast
Precipitation SHALL require cloud cover over the same column at or above a
configured threshold, so that a thin sky does not rain. What falls SHALL follow
the biome: snow over the cold biomes and rain elsewhere.

#### Scenario: Walking out of a storm
- **WHEN** a player walks far enough from a raining column
- **THEN** the rain they experience falls to nothing without any key being
  pressed

#### Scenario: A desert and a jungle under the same warmth
- **WHEN** the field is sampled over arid ground and over wet ground
- **THEN** the arid ground carries less cloud, and rains less often

### Requirement: Clouds are a lit slab with a thickness
The cloud layer SHALL be integrated through a shell of non-zero depth rather
than sampled at a single radius, accumulating transmittance and lighting each
sample toward the sun, so that a mass is brighter on top than underneath and a
grazing view crosses more of it than a vertical one.

#### Scenario: An overcast sky from below
- **WHEN** a player stands under full cover and looks up and then at the horizon
- **THEN** the horizon is more opaque than the zenith
- **AND** the underside of the cover is darker than its sunlit top

### Requirement: Cloud drifts at a pace the eye reads as weather
The simulation SHALL carry cloud with its steering wind scaled by a validated
`cloud_pace` in `0..1`, and SHALL carry vapour, heat, charge and the wind
itself unscaled. The wind map the renderer drifts cloud detail with SHALL be
that same carrying wind, so the texture moves with its cloud.

#### Scenario: The pace scales cloud and nothing else
- **WHEN** one carry is taken at a pace of one, of one half and of nought
- **THEN** the half-pace cloud moves half as far, the nought-pace cloud not at
  all, and vapour, heat, charge and wind are identical
  (`atmosphere::tests::the_cloud_pace_slows_the_cloud_and_nothing_else`)
