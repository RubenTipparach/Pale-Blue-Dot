# Vehicles Specification

## ADDED Requirements

### Requirement: A hull floats on the drawn sea by its cells
A hull's buoyancy SHALL be the sum over the cells of its envelope of the
displaced water at the shared sea function's height, and its heave damping
SHALL act on each cell's velocity relative to the water there. The same shape
function SHALL produce the cells and the drawn hull.

#### Scenario: Settling at rest
- **WHEN** a boat is released on a calm sea
- **THEN** it settles with a displaced volume equal to its mass over the
  water's density within 2 %

#### Scenario: Riding a wave
- **WHEN** a wave longer than the hull passes under it
- **THEN** the hull heaves and pitches with the wave rather than staying at
  the sea radius

### Requirement: Every lifting surface is one foil model
Wings, tail, fin, sail, keel, rudder and hull lateral planes SHALL use one foil
function: flow taken at the foil's own point relative to the local fluid
velocity, a lift slope reduced for aspect ratio, a stall into flat-plate flow,
and induced drag.

#### Scenario: Past the stall
- **WHEN** a foil's angle of attack rises past its stall angle
- **THEN** its lift falls and its drag rises

### Requirement: The VTOL converts from rotor lift to wing lift
The Kestrel's rotors SHALL tilt between vertical and horizontal under the
pilot's control, their thrust SHALL fall with inflow along their axis and rise
in ground effect, and its wing SHALL carry the craft only above its stall
speed.

#### Scenario: Hover takeoff with assist
- **WHEN** the pilot holds climb with the nacelles vertical and assist on
- **THEN** the craft climbs at the commanded vertical speed without rolling
  or pitching more than a few degrees

#### Scenario: Converting too slowly
- **WHEN** the nacelles are tilted fully forward below the wing's stall speed
- **THEN** the craft sinks and the stall is shown

### Requirement: Assist in the hover is a dampener
With assist on and the stick centred, the Kestrel in the hover SHALL hold its
attitude, its vertical speed and zero velocity over the ground; with assist
off the stick SHALL act directly.

#### Scenario: A crosswind hover
- **WHEN** the Kestrel hovers with assist on and no input in a 10 m/s wind
- **THEN** it tilts into the wind and its ground speed stays near zero

### Requirement: A sailing boat is driven by the apparent wind on its sail
The Tern's boom SHALL swing free with the apparent wind until the sheet stops
it, its sail SHALL produce force from the angle between the stopped sail and
the apparent wind, and its keel SHALL resist leeway with lift.

#### Scenario: Close reach
- **WHEN** the Tern sails at 60 degrees to an 11 m/s true wind, sheeted to
  hold the sail at a driving angle
- **THEN** it makes way with a positive speed made good to windward

#### Scenario: In irons
- **WHEN** the Tern points within about 32 degrees of the apparent wind
- **THEN** the sail luffs and the boat loses way

#### Scenario: Hull speed
- **WHEN** the Tern is driven hard on a broad reach
- **THEN** its speed levels off near sqrt(g L / 2 pi) for its waterline length

### Requirement: A paddle stroke is drag on the blade
Each stroke SHALL apply the blade's hydrodynamic drag against the local water
velocity at the blade's position, so a stroke on one side turns the canoe and
a canoe cannot outpace its own blade.

#### Scenario: Turning by paddling on one side
- **WHEN** the paddler strokes only on the right
- **THEN** the canoe turns to the left

### Requirement: Water aboard is a load that can swamp a boat
Rain over a boat's open area and sea over its rim SHALL add water aboard,
bailing and drains SHALL remove it, and the water SHALL add mass and shift
toward the low side.

#### Scenario: Rain in an open canoe
- **WHEN** 60 mm/h of rain falls on the Loon for one minute
- **THEN** 3.6 kg of water is aboard

#### Scenario: Swamping
- **WHEN** the Loon's rim is held under the sea
- **THEN** water floods in until the canoe floats awash

### Requirement: Weather acts on every craft, occupied or not
Wind, gusts, rain, current and waves SHALL act on every craft through the same
functions whether or not anyone is aboard.

#### Scenario: A boat left untied
- **WHEN** a boat is left unmoored and unanchored in a wind
- **THEN** it drifts downwind

### Requirement: Leaving a craft changes its occupancy, never its existence
Boarding and leaving SHALL change only a craft's occupancy. A left craft SHALL
remain in the world, simulated or asleep under the documented unattended
policy, and boardable with the same ID.

#### Scenario: Leave in the air, reload, reboard
- **WHEN** the pilot leaves the Kestrel in flight, the world is saved and
  reloaded
- **THEN** the Kestrel with the same ID is in the world where the unattended
  policy put it, and G boards it

### Requirement: Vehicle records use the durable save path
A craft's record (ID, kind, frame, pose, velocity, water aboard, mooring,
occupancy and owner) SHALL enter the durable transaction path when the craft is
spawned, boarded, left, moored, anchored, cast off or comes to rest, and SHALL
be acknowledged only after the storage backend succeeds.

#### Scenario: Mooring is saved at once
- **WHEN** a boat is moored and the process is killed before any timer fires
- **THEN** on reload the boat is at the bollard, moored

### Requirement: One interaction key boards and leaves
One key SHALL board the craft in reach from on foot and leave the craft the
player is in, and it SHALL appear in the binding table that the settings page
and `--help` print.

#### Scenario: Boarding the canoe from the pier
- **WHEN** a walker stands within reach of the Loon's boarding point and
  presses the interaction key
- **THEN** the walker is aboard as the paddler and the vehicle camera is active

### Requirement: The vehicle camera is the active camera
While a craft is occupied its camera SHALL be the active camera, so every
system that reads the active camera (water state, weather, LOD anchor)
follows it, and its seat view SHALL use the frame's raw mouse displacement.

#### Scenario: Water state from a boat
- **WHEN** the player sits in the Loon at sea
- **THEN** the water state is computed at the vehicle camera, not at the
  parked walker
