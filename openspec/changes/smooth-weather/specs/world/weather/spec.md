# Weather: smooth weather

## ADDED Requirements

### Requirement: Rain builds in and dies away

The rain a player sees at a place SHALL follow the simulation's rain there
(the cover where it rains, nothing where it does not) with a validated rise
time and a longer fall time, kept per cell in the atmosphere as derived state.
It SHALL NOT be saved or read by the simulation's physics, and the saved
weather SHALL be byte for byte what it was without it. The player's rain, the
rain shafts and the rain volume SHALL all read it.

#### Scenario: A burst of rain starts

- **WHEN** a cell's rain rate leaps past the raining threshold in one step
- **THEN** the rain seen there rises over the rise time rather than at once,
  moving no more than `1 - exp(-dt / rain_rise_s)` of the way in a step

#### Scenario: A short dry gap between two bursts

- **WHEN** the rain stops for 20 s and starts again
- **THEN** the rain seen falls over the fall time and has not reached nothing
  before it climbs again

#### Scenario: Saving is unchanged

- **WHEN** a stepped atmosphere is written with `to_bytes`
- **THEN** the bytes are those of the same state without the rain seen

### Requirement: What is shown of the weather never jumps

What the player sees of the weather - the cover and rain over them, the cloud
and wind maps the shaders read, the rain map and the rain shafts - SHALL run
between two published states of the atmosphere, mixed by how far it has come.
A state published during a blend SHALL wait for that blend to end. A blend
SHALL last the weather time between its two states, at least one step and at
most a configured cap. Only a capture, a first state and a clock that jumps
SHALL show a new state at once.

#### Scenario: An ordinary step

- **WHEN** one step is published while the clock runs at its pace
- **THEN** what is shown moves from the old state to the new over one second,
  continuously

#### Scenario: A held step catches up

- **WHEN** thirty steps are published at once after the weather was held
- **THEN** what is shown moves to the new state over the cap, not in one frame
