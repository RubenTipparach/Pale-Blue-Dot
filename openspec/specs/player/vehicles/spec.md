# Vehicles Specification

## Purpose
The Kestrel, the Tern and the Loon: three craft stepped by one rigid-body
model in `pbd-core` against the same sea, wind, rain and current the world
draws, boarded and left without ever ceasing to exist, and saved through the
durable path.

## Requirements

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
A craft's record (ID, kind, frame, pose, velocity, water aboard, mooring and
owner) SHALL be queued on the world's save writer in the same update the craft
is spawned, boarded, left, anchored, cast off or comes to rest, and on the pose
cadence while any craft moves. Once the writer has reported a failure the write
SHALL be refused and stay owed, as every other durable write is. Occupancy is
not saved: a player who quits aboard is loaded on foot beside the craft.

#### Scenario: Anchoring is saved at once
- **WHEN** a boat is cast off, then anchored again
- **THEN** the save holds each new state after the one update it happened in

### Requirement: One interaction key boards and leaves
One key SHALL board the craft in reach from on foot and leave the craft the
player is in, and it SHALL appear in the binding table that the settings page
and `--help` print. On foot, the board SHALL happen on a tap of that key: a
release before the tool picker's hold threshold. Holding it past the
threshold SHALL open the tool picker and board nothing.

#### Scenario: Boarding the canoe from the pier
- **WHEN** a walker stands within reach of the Loon's boarding point and
  taps the interaction key
- **THEN** the walker is aboard as the paddler and the vehicle camera is active

#### Scenario: Holding the key beside a craft
- **WHEN** a walker within reach of a craft holds the interaction key past the
  hold threshold
- **THEN** the tool picker opens and the walker is not aboard
  (`vehicles::tests::holding_g_beside_a_craft_boards_nothing`)

### Requirement: Foil forces remain oriented through reverse flow
Every vehicle foil SHALL orient lift perpendicular to relative flow with a
consistent span direction, including reverse flow. Drag SHALL oppose motion
relative to the fluid; changing flow direction SHALL NOT flip lift at
normal incidence through an artificial normal-vector sign change.

#### Scenario: Reverse flow from below
- **WHEN** a symmetric horizontal foil meets reverse flow from below
- **THEN** its vertical force is upward, and its net force does nonpositive work against its motion through the fluid

### Requirement: Rotor steering requires rotor power
The Kestrel's rotor steering SHALL be bounded by delivered rotor power.
Zero throttle with assist off SHALL produce no powered rotor force or
cyclic/yaw torque. Assisted controls SHALL stay finite when local gravity
falls to zero.

#### Scenario: Unpowered controls
- **WHEN** an airborne stationary Kestrel has zero throttle and assist off and the pilot requests rotation
- **THEN** its rotors produce no thrust or steering torque

#### Scenario: Gravity fades away
- **WHEN** the assisted Kestrel is stepped with zero local gravity and centred input
- **THEN** its pose, velocity and controls remain finite

### Requirement: Fleet placement respects the planet frame
A new fleet SHALL be placed from the player's body-local position regardless
of the planet's render-frame translation. Boarding and vehicle camera poses
SHALL use the same frame conversion.

#### Scenario: Translated planet
- **WHEN** the planet and walker are translated together within the render frame
- **THEN** the new craft retain equivalent body-local berths and are boardable with correctly translated cameras

### Requirement: Vehicle input is immediate and respects menus
Vehicle mouse look SHALL consume the current frame's raw displacement and
change the camera on that update independently of fixed physics ticks.
An open menu SHALL suppress vehicle controls and mouse look.

#### Scenario: Looking between physics ticks
- **WHEN** mouse displacement arrives while aboard without a physics tick
- **THEN** the camera turns on that update while the craft's physical orientation stays unchanged

#### Scenario: Menu owns input
- **WHEN** the menu is open while vehicle keys and mouse motion arrive
- **THEN** the vehicle control request and look do not change the craft or camera

### Requirement: The vehicle camera is the active camera
While a craft is occupied its camera SHALL be the active camera, so every
system that reads the active camera (water state, weather, LOD anchor)
follows it, and its seat view SHALL use the frame's raw mouse displacement.

#### Scenario: Water state from a boat
- **WHEN** the player sits in the Loon at sea
- **THEN** the water state is computed at the vehicle camera, not at the
  parked walker

### Requirement: Chase views frame the craft and avoid terrain
The Tern's default chase view SHALL include its masthead. A chase camera
SHALL shorten its boom when terrain obstructs it, with a positive clearance,
and retain the requested zoom distance for when the obstruction clears.
This correction SHALL NOT ease or delay raw mouse look.

#### Scenario: Canoe beside a steep shore
- **WHEN** the Loon's desired chase camera lies behind shoreline terrain
- **THEN** the camera is placed on the clear part of the boom instead of inside the ground

#### Scenario: Tern rig in view
- **WHEN** the Tern is boarded in its default chase view
- **THEN** the masthead and hull fit within the vertical field of view

### Requirement: Boat instruments distinguish water motion from ground motion
Boat telemetry SHALL report motion through water separately from motion over
ground. Tern leeway SHALL measure water-relative motion, and its instruments
SHALL show apparent-wind side and angle, windward speed made good over ground,
and a labeled hull-speed percentage.

#### Scenario: Drifting with a cross-current
- **WHEN** a boat moves with the water in a cross-current
- **THEN** its speed through water and leeway are near zero even though its ground speed is nonzero

#### Scenario: Trimming on either tack
- **WHEN** apparent wind comes from either side of the Tern
- **THEN** the panel identifies port or starboard and shows apparent-wind angle and windward VMG

### Requirement: Vehicle visuals follow configured surfaces and blade state
The Kestrel's drawn wing panels SHALL match their configured area, incidence
and dihedral. The Loon's drawn blade SHALL match its configured area and
its working stroke or stern-rudder position, including at zero water speed.

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

#### Scenario: Wing configuration
- **WHEN** the Kestrel mesh is built from its vehicle configuration
- **THEN** both panel areas and foil axes agree with the simulation configuration
