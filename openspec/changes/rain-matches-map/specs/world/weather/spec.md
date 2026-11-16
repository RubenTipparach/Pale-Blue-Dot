# Weather: the rain on the map is the rain you stand in

## ADDED Requirements

### Requirement: The map shows the rain the player stands in

The world map's rain (its clouds-and-rain layer and the RAIN overlay, on the
map and on the globe) SHALL draw the same field the player's rain reads: the
rain seen (`Atmosphere::rain_seen`), negative where it snows, through one
function in `pbd_core::weather`. Neither SHALL draw the simulation's rain
rate.

#### Scenario: Between two bursts of rain

- **WHEN** a place's rain rate falls under the raining threshold between two
  bursts and the rain seen there is still letting go
- **THEN** the player there stands in rain and the map shows rain there

#### Scenario: The same place on the map and on the ground

- **WHEN** the map's rain and the player's rain are read at the same place
  from the same weather state
- **THEN** they agree within a byte of the map's texel

### Requirement: Half the rain, none on the desert

The planet's rain SHALL come from the simulation's own water cycle at half
its earlier amount (evaporation and the rain threshold, never a scale on the
value shown), and rain falling over ground drier than `virga_dry_wetness`
SHALL evaporate before it lands and return to vapour, so none falls on the
desert. Saved weather SHALL be kept and stepped on under the new rules.

#### Scenario: A storm over the desert

- **WHEN** a desert cell's cloud is far past the rain threshold and struck
  by lightning
- **THEN** its rain rate and the rain seen there are zero
- **AND** cloud may still stand over it

#### Scenario: The planet over a month

- **WHEN** the settled climate is run for 30 world days
- **THEN** the share of the planet raining and its mean rain are about half
  what they were (0.5 +/- 0.1)

### Requirement: What falls in freezing air is snow

Where the air at the ground is at or below 0 C, what falls SHALL be snow,
through `shown_precipitation`, for the player and on the map alike. Snow
SHALL draw no rain streaks, rain fog, splashes, puddle rings, wet sheen or
lens drops: everything that is rain's water reads `Weather::liquid`, which
eases out over `snow_switch_s` where rain turns to snow.

#### Scenario: Rain turns to snow

- **WHEN** the air over the player falls to freezing while it rains
- **THEN** the flakes fall from that moment and the water's effects ease
  out over a few seconds and are gone
