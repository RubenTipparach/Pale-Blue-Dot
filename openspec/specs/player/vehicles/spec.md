# Vehicles Specification

## Purpose
The Tern and the Loon, and the harbours' rowboats and cog: craft stepped by
one rigid-body model in `pbd-core` against the same sea, wind, rain and
current the world draws, boarded and left without ever ceasing to exist, and
saved through the durable path. The Kestrel tiltrotor is retired
(`retire-kestrel`, owner 2026-11-14): a save's record of one is kept and not
flown, and aircraft return with `vehicle-yards`.

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

### Requirement: A sailing boat's keel lifts to the water under it
The Tern's keel SHALL go as deep as the water under it allows, its ballast
with it: it SHALL rise at once when the seabed comes up under it and lower at
its rate when the water deepens. A Tern SHALL float upright, moored, in half a
metre of water, and in open water its keel SHALL be all the way down
(`vehicle::tests::a_terns_keel_lifts_to_the_water_under_it`).

#### Scenario: Moored in the shallows
- **WHEN** a Tern is moored in 0.5 m of water in a 6 m/s wind
- **THEN** its keel lifts clear of the seabed at once and it heels under 8
  degrees

#### Scenario: Out into deep water
- **WHEN** a Tern with its keel lifted comes over deep water
- **THEN** the keel is about half way down after 1.5 s and all the way down
  after 3.5 s

### Requirement: The rowboat is its own craft and rows
The harbour's rowboat SHALL be its own craft, `Kind::Rowboat`, at the towns
mockup's size (4.2 m by 1.45 m, 0.6 m deep), rowed with the Loon's stroke on
its own spec (`rowboat` in `vehicles.ron`). It SHALL float upright empty and
with its rower aboard, and row ahead
(`vehicle::tests::a_rowboat_floats_upright_empty_and_with_its_rower_and_rows_ahead`).

#### Scenario: Afloat with its rower
- **WHEN** a rowboat with its rower aboard settles on the sea for 30 s
- **THEN** it stands within 3 degrees of upright, keel under the water and
  gunwale more than 0.2 m over it

#### Scenario: Rowing ahead
- **WHEN** its rower rows ahead for 30 s from rest
- **THEN** it has gone more than 20 m ahead and makes more than 0.8 m/s

### Requirement: An open boat's floor stays over the sea
An open boat (the Loon and the rowboat) SHALL be drawn with floorboards laid
3 cm over the still water it floats in with its paddler aboard, along the trim
the paddler gives it, inside its drawn planks, so the sea's sheet never shows
inside it (`vehicle::tests::an_open_boats_floorboards_stand_over_the_sea_inside_its_hull`).

#### Scenario: Through a swell
- **WHEN** a Loon or a rowboat with its paddler aboard rides the sea's swell
  for 25 s
- **THEN** every corner of its floorboards stays more than 1.5 cm over the
  sea

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
- **WHEN** the sailor leaves the Tern under way (the name is kept from when
  this was the Kestrel in the air), the world is saved and
  reloaded
- **THEN** the Tern with the same ID is in the world where the unattended
  policy put it, and F boards it

Pinned by
`vehicles::tests::a_craft_left_behind_stays_boardable_and_comes_back_from_its_save`.

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
One key, F, SHALL board the craft in reach from on foot and leave the craft
the player is in, on the press, and it SHALL appear in the binding table that
the settings page and `--help` print. It SHALL NOT be a key a craft uses as a
control (Q and E are hiking and the blade as a rudder), nor the tool
picker's key (G).

#### Scenario: Boarding the canoe from the pier
- **WHEN** a walker stands within reach of the Loon's boarding point and
  presses F
- **THEN** the walker is aboard as the paddler and the vehicle camera is active

#### Scenario: G beside a craft
- **WHEN** a walker within reach of a craft holds G
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

### Requirement: The Loon's blade is drawn as configured and where it works
The Loon's drawn blade SHALL match its configured area and its working stroke
or stern-rudder position, including at zero water speed
(`vehicles::draw::tests::paddle_mesh_and_motion_follow_area_and_working_rudder_without_force`).

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

### Requirement: A retired craft in a save is kept and not flown
A saved craft record whose kind the game has retired SHALL be held aside on
load and SHALL NOT be drawn, boarded or simulated. Every rewrite of the
vehicle file SHALL write it back unchanged, and no new craft SHALL take its ID.
The vehicle file's layout stays version 1
(`vehicles::tests::a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record`,
`vehicles::tests::a_held_record_keeps_its_id_from_every_new_craft`; the
fixture is an old build's own file, `vehicles/fixtures/kestrel-era-vehicles.ron`).

#### Scenario: Opening a world saved with a Kestrel
- **WHEN** a world whose vehicle file holds a Kestrel, a Tern and a Loon is
  opened, a boat is moved, and the world is saved
- **THEN** the world has the Tern and the Loon, no Kestrel is drawn, and the
  Kestrel's record in the saved file is byte-identical to the one read

### Requirement: A retired craft's saved rebinds are kept and read by nothing
A rebind in the player's settings file of a key only a retired craft read
SHALL be ignored on load, SHALL move no key, and SHALL be written back with
the rest of the file on the next change
(`settings::tests::a_rebind_of_a_retired_slot_is_ignored_and_kept`,
`controls::tests::rebinds_round_trip_through_their_names`).

#### Scenario: A Kestrel rebind in an old settings file
- **WHEN** a settings file that rebinds `kestrel:KeyW` is read and a setting
  is then changed
- **THEN** no key stands for that rebind, and the file written still holds it

### Requirement: A new world's fleet is the boats
A new world's fleet SHALL be the Tern and the Loon, placed from the player's
start as before, and the start SHALL be found by the same rule as before
(`vehicles::tests::a_new_world_places_each_craft_where_it_can_work`,
`walking::tests::a_new_world_starts_on_level_ground_and_old_versions_keep_their_start`).

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its fleet is a Tern and a Loon, no aircraft is placed, and the
  player's start is the same place it was before this change
