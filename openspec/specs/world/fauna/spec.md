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

### Requirement: Bed crawlers and surface floaters move with the world
Besides schools, the fauna SHALL have bed crawlers that walk the bed and
surface floaters that ride the sea function, appended to the roster after
the fish. Both SHALL be simulated on the CPU and deterministically, as
schools are, and have their own spawn cap so they never take a school of
fish's place.

#### Scenario: A floater rides the drawn sea
- **WHEN** a floating otter and the sea function are sampled at the same
  point and time
- **THEN** the otter's height is the sea's height

#### Scenario: A crawler keeps to the bed
- **WHEN** a crab walks for two minutes over a shelving bed
- **THEN** it is never more than 0.4 m off the bed nor in it

### Requirement: Protected creatures cannot be caught
Reef turtles, sea otters and ducks SHALL NOT be caught: only fish that swim
in schools take a lure, and a catch of a protected creature SHALL be refused
with the reason.

#### Scenario: A line over a turtle
- **WHEN** the float lands by a reef turtle
- **THEN** no turtle takes the lure, and the reason is "protected: it may
  not be caught"

### Requirement: A school is laid each frame into the buffers it holds
A school's mesh, laid again every frame, SHALL be laid into the attribute
buffers it already holds, so that once a school has reached its size a frame
allocates no new buffer for it; and what it lays SHALL be what laying it
into a new mesh would, flat normals included (`memory-leak`).

#### Scenario: Thirty frames of one school
- **WHEN** a school is laid into its mesh thirty frames in a row
- **THEN** no attribute's buffer moves or changes length after the first
  (`fish::tests::laid_in_place::a_school_is_laid_into_the_buffers_it_holds`)

#### Scenario: Relaid against fresh
- **WHEN** the same school with the same history is laid once into a mesh
  that already held it and once into a new mesh
- **THEN** every attribute is the same, byte for byte, and the normals are
  what `Mesh::compute_flat_normals` makes of the positions
  (`fish::tests::laid_in_place::a_school_relaid_is_the_school_laid_fresh`)

### Requirement: Water life is seen through the water it is in
Every creature and sea place under water SHALL be lit and tinted as the
seabed beside it is: by the terrain's light (its fill, its sun, the sun's
shadow and the lamps), dimmed by the daylight left at its depth under the
surface of the water it is in, and taking the same share of the water's
deep colour as the seabed at that depth. Seen from under the water it SHALL
take the murk by distance, and from above the water's transmission, as the
seabed does. Its atlas and palette SHALL be kept, with no outline.

#### Scenario: A fish near the surface is not a sticker

- **WHEN** the eye is 2 m under a shallow sea by day, looking at a school 3 m off
- **THEN** the fish are dimmed and tinted toward the water as the reef
  beside them is, not drawn in the atlas's full colours

#### Scenario: Fresh water dims too

- **WHEN** a school of perch swims 1 m under a lake's surface that stands
  over sea level
- **THEN** its depth is read from the lake's surface and it is dimmed by it

### Requirement: Beavers are seen only by a colony
A beaver SHALL spawn only within 40 m of the site of a beaver colony that is
not gone (`near_colony_m` in `fauna.ron`). Like every species it is appended
to the roster, never inserted, and like the otters it is protected.

#### Scenario: By a colony and away from one
- **WHEN** a spawn point lies 35 m from a living colony's site, 60 m from it,
  by a colony that has moved to its next site, or by a gone colony
- **THEN** a beaver is offered only at the first
  (`beaver::tests::beavers_are_seen_only_within_reach_of_a_colony_not_gone`,
  `fish::tests::a_beaver_is_offered_only_by_a_colony`)

#### Scenario: The roster grows at its end
- **WHEN** the home roster is read
- **THEN** the beaver and the river otter follow the seal, every earlier
  species at its old index
  (`fauna::tests::the_defaults_validate_and_carry_the_eight_fish_then_the_water_life`)

### Requirement: The beavers and otters move as their approved mockup does
The beaver, the river otter and the sea otter SHALL be drawn with M25
version 5's sculpted, skinned bodies and its motion: every action in four
stages on one beat, gaits whose phase runs on the distance moved, the
forelimbs held within each animal's joint limits, and the tunables read
from `creature_motion.ron`, validated, with one source of defaults. The
motion SHALL be drawn only: nothing in the game reads it back.

#### Scenario: The gnaw keeps its beat
- **WHEN** a beaver gnaws a stick for six bites
- **THEN** the k-th bite lands at k beats plus the windup, anticipation and
  hit, within a frame (`critters::tests::the_gnaws_hit_is_on_its_beat`)

#### Scenario: A planted foot stays planted
- **WHEN** a beaver walks and a river otter bounds across flat ground
- **THEN** no foot down in its stance moves more than 1.5 mm in a frame
  (`critters::tests::no_foot_slides_in_stance`)

#### Scenario: The forelimbs within their limits
- **WHEN** each animal is left to its seeded choices for 600 s by a pond,
  startled now and then
- **THEN** its elbow and wrist never pass its limits
  (`critters::tests::the_forelimbs_never_pass_their_limits_over_a_seeded_run`)

#### Scenario: The tunables
- **WHEN** `creature_motion.ron` is read
- **THEN** it validates and equals the defaults, a block tuned on the
  mockup pastes in, and a value off the mockup's sliders is refused
  (`critters::tests::the_shipped_motion_is_the_defaults_and_validates`,
  `critters::tests::a_block_tuned_on_the_page_pastes_in_and_a_bad_one_is_refused`)
