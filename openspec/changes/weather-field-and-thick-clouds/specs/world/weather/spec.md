# Weather Specification

## ADDED Requirements

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
