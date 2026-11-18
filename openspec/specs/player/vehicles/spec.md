# Vehicles Specification

## Purpose
The Loon, the harbours' canoes, and the made tube and raft: craft
stepped by one rigid-body model in `pbd-core` against the same sea, wind,
rain and current the world draws, boarded and left without ever ceasing to
exist, and saved through the durable path. The Kestrel tiltrotor is retired
(`retire-kestrel`, owner 2026-11-14), and so are the Tern sailing dinghy
(`retire-tern`, owner 2026-11-17) and the harbours' rowboats and cog
(`retire-rowboat-and-cog`, owner 2026-11-18): a save's record of any is kept
and not sailed. Aircraft return with `vehicle-yards`, new boats with M23 and M34.

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

### Requirement: An open boat's floor stays over the sea
An open boat (the Loon) SHALL be drawn with floorboards laid 3 cm over the
still water it floats in with its paddler aboard, along the trim the paddler
gives it, inside its drawn planks, so the sea's sheet never shows inside it
(`vehicle::tests::an_open_boats_floorboards_stand_over_the_sea_inside_its_hull`).

#### Scenario: Through a swell
- **WHEN** a Loon with its paddler aboard rides the sea's swell for 25 s
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
- **WHEN** the paddler leaves the Loon under way (the name is kept from when
  this was the Kestrel in the air), the world is saved and reloaded
- **THEN** the Loon with the same ID is in the world where the unattended
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

### Requirement: The Loon's blade is drawn as configured and where it works
The Loon's drawn blade SHALL match its configured area and its working stroke
or stern-rudder position, including at zero water speed
(`vehicles::draw::tests::paddle_mesh_and_motion_follow_area_and_working_rudder_without_force`).

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

### Requirement: A retired craft in a save is kept and not flown
A saved craft record whose kind the game has retired (the Kestrel, the Tern,
the rowboat, the cog) SHALL be held aside on load and SHALL NOT be drawn,
boarded or simulated. Every rewrite of the vehicle file SHALL write it back
unchanged, and no new craft SHALL take its ID. The vehicle file's layout
stays version 1
(`vehicles::tests::a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record`,
`vehicles::tests::a_world_saved_with_harbour_terns_keeps_every_record`,
`vehicles::tests::a_world_saved_with_rowboats_and_a_cog_keeps_every_record`,
`vehicles::tests::a_held_record_keeps_its_id_from_every_new_craft`; the
fixtures are old builds' own files, `vehicles/fixtures/`).

#### Scenario: Opening a world saved with a Kestrel
- **WHEN** a world whose vehicle file holds a Kestrel, a Tern and a Loon is
  opened, the Loon is moved, and the world is saved
- **THEN** the world has the Loon, no Kestrel or Tern is drawn, and both
  retired records in the saved file are byte-identical to the ones read

#### Scenario: Opening a world saved with harbour rowboats and a cog
- **WHEN** a world whose vehicle file holds harbour rowboats, a cog, a made
  rowboat and a Loon is opened and saved
- **THEN** only the Loon is in the world, and every rowboat and cog record
  in the saved file is byte-identical to the one read

### Requirement: A retired craft's saved rebinds are kept and read by nothing
A rebind in the player's settings file of a key only a retired craft read
SHALL be ignored on load, SHALL move no key, and SHALL be written back with
the rest of the file on the next change
(`settings::tests::a_rebind_of_a_retired_slot_is_ignored_and_kept`,
`controls::tests::rebinds_round_trip_through_their_names`).

#### Scenario: A Kestrel rebind in an old settings file
- **WHEN** a settings file that rebinds `kestrel:KeyW`, `tern:KeyW`,
  `rowboat:KeyW` and `cog:KeyA` is read and a setting is then changed
- **THEN** no key stands for any of them, and the file written still holds
  them all

### Requirement: A new world's fleet is the boats
A new world's fleet SHALL be the Loon, placed from the player's start as
before, and the start SHALL be found by the same rule as before
(`vehicles::tests::a_new_world_places_each_craft_where_it_can_work`,
`walking::tests::a_new_world_starts_on_level_ground_and_old_versions_keep_their_start`).
The Tern is retired (`retire-tern`, owner 2026-11-17).

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its fleet is a Loon, no Tern and no aircraft are placed, and the
  player's start is the same place it was before this change

### Requirement: A player loaded at a retired craft is put ashore
When a world opens with the walker over the sea within 6 m of a craft record
the game has retired, or within 9 m of a retired cog's, the walker SHALL be
put on the nearest dry land, keeping its heading. A walker anywhere else
SHALL be loaded exactly where the save left it
(`vehicles::tests::a_walker_at_a_retired_craft_over_the_sea_goes_ashore`).

#### Scenario: Saved aboard a Tern
- **WHEN** a world saved with the player aboard its Tern is opened by a build
  that has retired the Tern
- **THEN** the player stands on dry land, and the Tern's record is kept

#### Scenario: Saved on a cog's deck
- **WHEN** a world saved with the player standing on its cog's deck, 7 m
  from its middle, is opened by a build that has retired the cog
- **THEN** the player stands on dry land, and the cog's record is kept

### Requirement: Harbours moor no Tern
A harbour template's boats SHALL make a Loon for each `canoe` and a raft at
each `rowboat` or `sail` berth whose middle is at least 5.1 m from the rafts
given a berth before it; its cog SHALL make none. A raft's berth SHALL be
numbered apart from the template boat's, so a berth held by a retired record
still gets its raft, and a berth whose craft is held SHALL count as made, so
it is not filled again
(`towns::tests::every_harbours_boats_are_made_once_at_their_berths`).

#### Scenario: A harbour in an old world
- **WHEN** a world saved with harbour Terns, rowboats and a cog is opened and
  the player comes to the harbour
- **THEN** its canoes are moored as before, a raft lies at each raft berth,
  and no boat is made at the cog's berth

#### Scenario: A new world's harbour
- **WHEN** the fleet first meets a harbour in a new world
- **THEN** it moors its canoes and its rafts, no two rafts within 5.1 m of
  each other, each anchored at its berth

### Requirement: A chase camera avoids terrain
A chase camera SHALL shorten its boom when terrain obstructs it, with a
positive clearance, and retain the requested zoom distance for when the
obstruction clears. This correction SHALL NOT ease or delay raw mouse look
(`vehicles::chase::tests::a_shore_shortens_the_boom_and_clear_ground_restores_it`).

#### Scenario: Canoe beside a steep shore
- **WHEN** the Loon's desired chase camera lies behind shoreline terrain
- **THEN** the camera is placed on the clear part of the boom instead of inside the ground

### Requirement: Boat telemetry reads water motion apart from ground motion
Boat telemetry SHALL report motion through water separately from motion over
ground (`vehicle::tests::loon_instruments_separate_water_motion_from_ground_motion`).

#### Scenario: Drifting with a cross-current
- **WHEN** a boat moves with the water in a cross-current
- **THEN** its speed through water and leeway are near zero even though its ground speed is nonzero

### Requirement: A retired boat the player made is refunded once
A held record of a retired kind that the player made (`made: true`) SHALL
be refunded on load: its recipe's materials as it shipped and every stack in
its hold SHALL be given into the player's slots, all or nothing, in one
durable line with a `refund` record naming the craft. A craft with a
`refund` record SHALL never be refunded again. While the slots cannot take
all of it, nothing SHALL be given and it SHALL be tried again until they can
(`vehicles::tests::a_made_rowboat_is_refunded_once`,
`vehicles::tests::a_refund_waits_for_room_in_the_pack`).

#### Scenario: Opening a world with a rowboat the player made
- **WHEN** a world saved with a made rowboat is opened, saved and opened
  again
- **THEN** the player holds 16 more planks and 2 more rope after the first
  open, the same after the second, and the rowboat's record is unchanged

### Requirement: A retired boat good and its job give their materials back
A good for a retired boat in the player's slots SHALL be replaced by its
recipe's materials when they fit, and a queued job for a retired recipe
SHALL take no time and collect as its inputs
(`vehicles::tests::a_rowboat_good_becomes_its_materials`,
`station::tests::a_retired_recipes_job_collects_as_its_inputs`).

#### Scenario: A rowboat in the pack
- **WHEN** a world is opened with a rowboat good in the pack
- **THEN** the pack holds 16 planks and 2 rope in its place
