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

### Requirement: The wind at cloud height has no calm band at the equator
The wind at cloud height SHALL blow over the whole planet. Within the
tropics, where the jet's balance against the planet's spin does not hold, it
SHALL blow westward, against the planet's turn, at the configured tropical
easterly speed. It SHALL turn into the jets over a band of latitude, not at
an edge. Between neighbouring 2.5-degree bands within 35 degrees of the
equator, the zonal-mean speed SHALL change by no more than 3.5 m/s per degree
of latitude. Pinned by `atmosphere::tests::the_tropics_blow_easterly_aloft_and_the_jet_has_no_edge`
and `atmosphere::tests::with_no_easterly_the_tropics_aloft_have_the_surface_wind`.

#### Scenario: The equator aloft
- **WHEN** the wind at cloud height is worked out on a settled climate and
  averaged round the planet within 5 degrees of the equator
- **THEN** it blows westward at no less than half the configured tropical
  easterly speed

#### Scenario: The jet's edge
- **WHEN** the zonal-mean speed of the wind at cloud height is taken in
  2.5-degree bands from 35 degrees south to 35 degrees north
- **THEN** no band differs from its neighbour by more than 3.5 m/s per degree

#### Scenario: No easterly asked for
- **WHEN** the tropical easterly speed is set to zero
- **THEN** the wind at cloud height within 5.7 degrees of the equator is the
  surface wind, and the jet's edge still changes by no more than 3.5 m/s per
  degree
