# Fauna Specification

## Purpose
The fish of a body: which species live on it, where each spawns (by the class
of water and its temperature now), how a school swims, and what the field
guide says about each. The rules are `pbd_core::fauna`, the data is
`assets/config/fauna.ron`, and the app (`pbd_app::fish`) adapts them to the
world's sea, ground and atmosphere.

## Requirements

### Requirement: Fish swim in schools
Fish SHALL move as schools of boids: each fish steered by separation from
close neighbours, alignment with and cohesion toward its school, and a goal
the school shares. Every fish SHALL stay between the sea surface and the
seabed, and a bed dweller SHALL keep to the bed.

#### Scenario: A school over a shelving bed
- **WHEN** a school swims for two minutes over a bed that shelves to the shore
- **THEN** no fish of it leaves the water, enters the bed, or goes non-finite
  (`fauna::tests::a_school_stays_in_the_water`)

### Requirement: Rosters are per body, and lifeless bodies have none
Each body's fish species SHALL be listed in explicit data. No species SHALL
appear on two bodies. A body whose roster is empty SHALL spawn no fish.

#### Scenario: A barren moon's sea
- **WHEN** the player is on a body with an empty roster
- **THEN** no school spawns and the rod says nothing lives in the water
  (`fish::tests::a_body_with_no_roster_says_nothing_lives_here`)

#### Scenario: Two bodies share a species
- **WHEN** the roster lists one species on two bodies
- **THEN** validation refuses it
  (`fauna::tests::a_species_on_two_bodies_or_without_an_entry_is_refused`)

### Requirement: Schools are simulated on the CPU and deterministically
Schools SHALL be stepped on the CPU at a fixed rate, in packed arrays rather
than an entity per fish, and drawn from one mesh per school rebuilt from those
arrays, with no readback. A school stepped from the same seed with the same
inputs SHALL be the same school.

#### Scenario: Two runs
- **WHEN** a school is spawned from one seed twice and stepped the same number
  of ticks with the same splash
- **THEN** every fish is at the same place in both
  (`fauna::tests::a_school_is_a_function_of_its_seed_and_its_steps`)

### Requirement: Every species has a field-guide entry and a thumbnail
Every species on every body SHALL carry a field-guide entry (its text and an
angler's tip) and a 16x16 thumbnail. The slot and the field guide SHALL draw
that same thumbnail. The numbers an entry shows (water, temperature, depth,
how it is found, length, strength, hook window, speed) SHALL be read from the
species record the simulation and the hook use, not written a second time.
J SHALL open the guide, on the species in the selected slot when it holds a
fish.

#### Scenario: A species added without an entry
- **WHEN** a species has no entry text or no icon
- **THEN** validation refuses it, and a test fails if its icon file is missing
  (`fish::tests::every_species_and_every_tool_has_an_icon`)

#### Scenario: The hook window an entry shows
- **WHEN** the field guide shows a species of strength 5
- **THEN** it shows the 0.43 s window the bite uses
  (`fish::tests::the_guide_reads_its_numbers_off_the_species`)

#### Scenario: Opening an entry from the slots
- **WHEN** the player presses J with a fish in the selected slot
- **THEN** the field guide opens on that species' entry
  (`desktop::guide::tests::j_opens_the_guide_on_the_fish_in_hand`)

### Requirement: Icons reused from Tenebris are copies with provenance
A fish icon taken from Tenebris SHALL be a byte-for-byte copy inside this
repository, with its source path and commit recorded beside it and its hash
checked by a test. Nothing in the build SHALL read from the reference
checkout.

#### Scenario: A copied icon drifts
- **WHEN** a copied Tenebris icon is edited in place
- **THEN** the provenance test fails until the record is updated with it
  (`fish::tests::the_copied_tenebris_icons_are_unchanged`)

### Requirement: A species spawns only in its water, at its temperature
Each species SHALL list the water classes it lives in (river, shallows, shelf,
deep: from the terrain generator, with the depth limits in data) and a
temperature window. A school SHALL spawn only in a place whose class it lists
and whose water temperature at that moment, read from the atmosphere, is
inside its window. Nothing SHALL spawn in water below its freezing point. On
day one of the reference world, every point of open water SHALL be inside at
least one species' range.

#### Scenario: Reef fish in cool water
- **WHEN** a spawn candidate in the shallows reads 20 C
- **THEN** no reef fish school spawns there, since its window starts at 23 C

#### Scenario: A river is not the sea
- **WHEN** a spawn candidate is a river channel at 12 C
- **THEN** perch can spawn there, and silverfin cannot
  (`fauna::tests::a_species_spawns_only_in_its_water_at_its_temperature`)

#### Scenario: Frozen water
- **WHEN** the water temperature at a candidate is below -1.8 C
- **THEN** no school spawns there

#### Scenario: Day one
- **WHEN** every point of open water on the reference world is classed and
  read against the day-one atmosphere
- **THEN** none is outside every species' range
  (`fauna::tests::on_day_one_no_open_water_is_without_a_species`)

### Requirement: The catch record is saved with the catch
Each catch SHALL add to a per-species record of the count caught and the best
length. That record SHALL reach the durable save in the same line as the fish
itself, and SHALL be rebuilt from the save on load.

#### Scenario: Catch, quit and reload
- **WHEN** reef fish are caught and the world is reloaded
- **THEN** the record shows the count and the best length
  (`saves::tests::catches_and_the_tool_in_hand_come_back`)
