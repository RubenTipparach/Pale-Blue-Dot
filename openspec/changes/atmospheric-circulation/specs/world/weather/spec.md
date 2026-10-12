# World: atmospheric circulation

## REMOVED Requirements

### Requirement: Weather is a deterministic field, not a stored state
**Reason**: A field sampled on demand has no wind, pressure or heat, so nothing
in it flows, equalizes or is driven by the sun. The owner asked for a
simulated atmosphere on the planet's cells.
**Migration**: "Weather is a simulated atmosphere on the planet's cells"
replaces it. Determinism moves from "a pure function of time" to "a
deterministic step from a seed", and the state is saved with the world.

## MODIFIED Requirements

### Requirement: Rain falls where it is overcast
Precipitation SHALL require cloud cover over the same column at or above a
configured threshold, so that a thin sky does not rain. What falls SHALL follow
the surface temperature the atmosphere simulates: snow below freezing and rain
above it.

#### Scenario: Walking out of a storm
- **WHEN** a player walks far enough from a raining column
- **THEN** the rain they experience falls to nothing without any key being
  pressed

#### Scenario: A desert and a jungle under the same warmth
- **WHEN** the atmosphere runs over arid ground and over wet ground
- **THEN** the arid ground carries less cloud, and rains less often

## ADDED Requirements

### Requirement: Weather is a simulated atmosphere on the planet's cells
Cloud, precipitation, wind and lightning SHALL come from one atmosphere
simulation stepped on the planet's hexagonal cells at a fixed step. From the
same seed and the same number of steps, the state SHALL be identical on every
run of a build, and it SHALL be saved and restored with the world.

#### Scenario: Two runs from one seed
- **WHEN** the atmosphere is stepped twice from one seed for the same number
  of steps
- **THEN** the two states agree bit for bit

#### Scenario: Saved and resumed
- **WHEN** the state is saved, loaded and stepped on
- **THEN** it equals the state that was stepped on without saving

### Requirement: The sun drives the weather
The atmosphere SHALL be heated by sunlight from the world clock's sun, with the
ocean storing heat more slowly than the land and clouds reflecting sunlight
away. Air SHALL flow from high pressure toward low, pressure differences SHALL
even out, and the flow SHALL turn with the planet's spin.

#### Scenario: A tropical rain belt and desert belts
- **WHEN** the atmosphere has run for a day
- **THEN** the most rain falls within 20 degrees of the latitude under the sun
- **AND** each hemisphere has a drier band 15 to 40 degrees from it

#### Scenario: Storms turn with the spin
- **WHEN** air flows into a low in the northern hemisphere
- **THEN** it turns anticlockwise round it, and clockwise in the southern

#### Scenario: A disturbance evens out
- **WHEN** a single pressure bump is released with no forcing
- **THEN** it spreads and dies away, and the mean pressure is unchanged

### Requirement: Cloud comes from water the air carries
Water SHALL evaporate from the sea more than from land, travel with the wind,
condense where air converges and rises, and rain out of cloud that is thick
enough. Moving water between cells SHALL neither create it nor destroy it.

#### Scenario: The sea waters the land
- **WHEN** cover is averaged over a day
- **THEN** it is higher over the ocean than over land

### Requirement: Cloud cover differs from place to place
Clouds SHALL be drawn with the cover, height and wind of the place they are
over, not the cover over the player, and SHALL move with the wind at their
height, including a jet where the temperature gradient is steepest.

#### Scenario: The planet from orbit
- **WHEN** the planet is seen whole
- **THEN** some of it is clear and some of it is under full cover at once

### Requirement: Lightning comes from the storms themselves
Lightning SHALL strike where the simulation's storms have built up charge, and
each strike SHALL discharge its cell and push a cold outflow into the cells
around it. Where it strikes decides where the flash is drawn.

#### Scenario: A calm sky
- **WHEN** no cell has built up charge
- **THEN** no lightning strikes

#### Scenario: The outflow
- **WHEN** a cell strikes
- **THEN** on the next step air flows out of it into its neighbours

### Requirement: The ocean has currents
The ocean SHALL carry a surface current driven by the wind, turned by the same
spin the air feels, blocked by the coasts, and carrying the sea's heat; the
water surface SHALL move with it.

#### Scenario: A wind-driven gyre
- **WHEN** the trade winds and the westerlies blow over an ocean basin
- **THEN** the basin's water circulates in a gyre, fastest along its western side

#### Scenario: A coast is a wall
- **WHEN** a current reaches a coast
- **THEN** no water flows into the land cell
