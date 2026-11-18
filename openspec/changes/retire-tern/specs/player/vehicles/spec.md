## ADDED Requirements

### Requirement: A player loaded at a retired craft is put ashore
When a world opens with the walker over the sea within 6 m of a craft record
the game has retired, the walker SHALL be put on the nearest dry land,
keeping its heading. A walker anywhere else SHALL be loaded exactly where the
save left it (`vehicles::place::tests::a_walker_at_a_retired_craft_over_the_sea_goes_ashore`).

#### Scenario: Saved aboard a Tern
- **WHEN** a world saved with the player aboard its Tern is opened by a build
  that has retired the Tern
- **THEN** the player stands on dry land, and the Tern's record is kept

### Requirement: Harbours moor no Tern
A harbour template's `sail` boats SHALL make no craft, and a berth whose
craft is held as a retired record SHALL count as made, so it is not filled
again (`towns::tests::harbours_moor_their_boats_on_the_water_once`).

#### Scenario: A harbour in an old world
- **WHEN** a world saved with harbour Terns is opened and the player comes to
  the harbour
- **THEN** its rowboats, canoes and cog are moored as before, and no boat is
  made at a Tern's berth

### Requirement: A chase camera avoids terrain
A chase camera SHALL shorten its boom when terrain obstructs it, with a
positive clearance, and retain the requested zoom distance for when the
obstruction clears. This correction SHALL NOT ease or delay raw mouse look.

#### Scenario: Canoe beside a steep shore
- **WHEN** the Loon's desired chase camera lies behind shoreline terrain
- **THEN** the camera is placed on the clear part of the boom instead of inside the ground

### Requirement: Boat telemetry reads water motion apart from ground motion
Boat telemetry SHALL report motion through water separately from motion over
ground.

#### Scenario: Drifting with a cross-current
- **WHEN** a boat moves with the water in a cross-current
- **THEN** its speed through water and leeway are near zero even though its ground speed is nonzero

## MODIFIED Requirements

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

### Requirement: A retired craft in a save is kept and not flown
A saved craft record whose kind the game has retired (the Kestrel, the Tern)
SHALL be held aside on load and SHALL NOT be drawn, boarded or simulated.
Every rewrite of the vehicle file SHALL write it back unchanged, and no new
craft SHALL take its ID. The vehicle file's layout stays version 1
(`vehicles::tests::a_world_saved_with_a_kestrel_opens_with_its_boats_and_keeps_the_record`,
`vehicles::tests::a_world_saved_with_harbour_terns_keeps_every_record`,
`vehicles::tests::a_held_record_keeps_its_id_from_every_new_craft`; the
fixtures are old builds' own files, `vehicles/fixtures/`).

#### Scenario: Opening a world saved with a Kestrel
- **WHEN** a world whose vehicle file holds a Kestrel, a Tern and a Loon is
  opened, the Loon is moved, and the world is saved
- **THEN** the world has the Loon, no Kestrel or Tern is drawn, and both
  retired records in the saved file are byte-identical to the ones read

### Requirement: A retired craft's saved rebinds are kept and read by nothing
A rebind in the player's settings file of a key only a retired craft read
SHALL be ignored on load, SHALL move no key, and SHALL be written back with
the rest of the file on the next change
(`settings::tests::a_rebind_of_a_retired_slot_is_ignored_and_kept`,
`controls::tests::rebinds_round_trip_through_their_names`).

#### Scenario: A Kestrel rebind in an old settings file
- **WHEN** a settings file that rebinds `kestrel:KeyW` and `tern:KeyW` is read
  and a setting is then changed
- **THEN** no key stands for either rebind, and the file written still holds
  both

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

## REMOVED Requirements

### Requirement: A sailing boat is driven by the apparent wind on its sail
**Reason**: The Tern, the only craft it described, is retired (owner,
2026-11-17). The cog's square sail has its own model (`sail-the-cog`).
**Migration**: None; a saved Tern is held as its record.

### Requirement: A sailing boat's keel lifts to the water under it
**Reason**: The lifting keel was the Tern's alone, and the Tern is retired.
**Migration**: None; a saved Tern is held as its record.

### Requirement: Chase views frame the craft and avoid terrain
**Reason**: Its first clause was the Tern's masthead. The rest is kept as
"A chase camera avoids terrain".
**Migration**: None.

### Requirement: Boat instruments distinguish water motion from ground motion
**Reason**: Its sail instruments were the Tern's. The rest is kept as "Boat
telemetry reads water motion apart from ground motion".
**Migration**: None.
