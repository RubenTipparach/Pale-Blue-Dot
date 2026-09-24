# Fauna Specification

## ADDED Requirements

### Requirement: Fish swim in schools
Fish SHALL move as schools of boids: each fish steered by separation from
close neighbours, alignment with and cohesion toward its school, and a goal
the school shares. Every fish SHALL stay between the sea surface and the
seabed, and inside its species' depth band.

#### Scenario: A school over a shelving bed
- **WHEN** a school's goal lies over water shallower than its band
- **THEN** no fish of it enters that water or leaves the sea

### Requirement: Rosters are per body, and lifeless bodies have none
Each body's fish species SHALL be listed in explicit data. No species SHALL
appear on two bodies. A body whose roster is empty SHALL spawn no fish.

#### Scenario: A barren moon's sea
- **WHEN** the player casts on a body with an empty roster
- **THEN** no school spawns and no bite comes

#### Scenario: Two bodies share a species
- **WHEN** the shipped roster lists one species id on two bodies
- **THEN** a test fails

### Requirement: Schools are simulated on the CPU and deterministically
Schools SHALL be stepped on the CPU at a fixed rate, in packed arrays rather
than an entity per fish, and drawn from an uploaded instance buffer with no
readback. Two runs from the same world, seed and inputs SHALL produce the
same schools.

#### Scenario: Two runs
- **WHEN** a world is loaded twice and stepped the same number of ticks with
  the same inputs
- **THEN** every fish of every school is at the same place in both

### Requirement: Every species has a field-guide entry and a thumbnail
Every species on every body SHALL carry a field-guide entry (its text and an
angler's tip) and a 16×16 thumbnail in the item manifest. The slot and the
field guide SHALL draw that same thumbnail. The numbers an entry shows (depth,
how it is found, length, strength, hook window, speed) SHALL be read from the
species record the simulation and the hook use, not written a second time. A
generated wiki page SHALL equal the output of its generator run on the
shipped roster.

#### Scenario: A species added without an entry
- **WHEN** a species is added to `fauna.ron` with no entry text or no icon
- **THEN** a test fails

#### Scenario: The hook window an entry shows
- **WHEN** the field guide shows a species of strength 5
- **THEN** it shows the 0.43 s window the bite uses

#### Scenario: Opening an entry from the slots
- **WHEN** the player clicks a slot holding a fish, or presses J
- **THEN** the field guide opens on that species' entry, or on the list

### Requirement: Icons reused from Tenebris are copies with provenance
A fish icon taken from Tenebris SHALL be a byte-for-byte copy inside this
repository, with its source path and commit recorded beside it and its hash
checked by a test. Nothing in the build SHALL read from the reference
checkout.

#### Scenario: A copied icon drifts
- **WHEN** a copied Tenebris icon is edited in place
- **THEN** the provenance test fails until the record is updated with it

### Requirement: A species lives in the water zones it lists
Each species SHALL list the water zones it lives in: tropical, temperate or
cold. A school SHALL spawn only in a cell whose zone, classified from the
sea's surface temperature at spawn against thresholds in data, is one it
lists. Every zone on a living body SHALL have at least one species.

#### Scenario: Reef fish in cold water
- **WHEN** a spawn candidate's surface temperature is below the cold
  threshold
- **THEN** no reef fish school spawns there

### Requirement: The catch record is saved with the catch
Each catch SHALL add to a per-species record of the count caught and the best
length. That record SHALL reach the durable save in the same line as the fish
itself, and SHALL be rebuilt from the save on load.

#### Scenario: Catch, quit and reload
- **WHEN** a first reef fish of 44 cm is caught and the world is reloaded
- **THEN** the field guide shows one reef fish caught, best 44 cm
