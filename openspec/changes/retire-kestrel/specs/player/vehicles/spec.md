## ADDED Requirements

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

### Requirement: The Loon's blade is drawn as configured and where it works
The Loon's drawn blade SHALL match its configured area and its working stroke
or stern-rudder position, including at zero water speed
(`vehicles::draw::tests::paddle_mesh_and_motion_follow_area_and_working_rudder_without_force`).

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: The VTOL converts from rotor lift to wing lift
**Reason**: The Kestrel is retired (owner, 2026-11-14: "can we remove it,
since we are building better plane systems?").
**Migration**: None in play. A saved Kestrel is kept by "A retired craft in a
save is kept and not flown"; aircraft return with `vehicle-yards`.

### Requirement: Assist in the hover is a dampener
**Reason**: The Kestrel is retired; it was the only craft that hovered.
**Migration**: None; `vehicle-yards` designs its own assist.

### Requirement: Rotor steering requires rotor power
**Reason**: The Kestrel is retired; it was the only craft with rotors.
**Migration**: None.

### Requirement: Vehicle visuals follow configured surfaces and blade state
**Reason**: Its Kestrel half (the wing panels) goes with the Kestrel; the
Loon half is kept, unchanged, as "The Loon's blade is drawn as configured and
where it works".
**Migration**: The Loon's test moves to the new requirement's name.
