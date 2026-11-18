## ADDED Requirements

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

## MODIFIED Requirements

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

### Requirement: An open boat's floor stays over the sea
An open boat (the Loon) SHALL be drawn with floorboards laid 3 cm over the
still water it floats in with its paddler aboard, along the trim the paddler
gives it, inside its drawn planks, so the sea's sheet never shows inside it
(`vehicle::tests::an_open_boats_floorboards_stand_over_the_sea_inside_its_hull`).

#### Scenario: Through a swell
- **WHEN** a Loon with its paddler aboard rides the sea's swell for 25 s
- **THEN** every corner of its floorboards stays more than 1.5 cm over the
  sea

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

## REMOVED Requirements

### Requirement: The rowboat is its own craft and rows
**Reason**: The rowboat is retired (owner, 2026-11-18: "Rowboats and cog
yes. Remove them. We are going to replace them with craftable ones").
**Migration**: A saved rowboat is held as its record; one the player made
is refunded as its materials.
