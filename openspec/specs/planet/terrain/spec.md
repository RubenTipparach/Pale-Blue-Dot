# Terrain Generation Specification

## Purpose
Terrain is a pure function of position and seed, evaluated on the CPU, so that
two machines agree, a chunk generated in any order matches, and the GPU never
decides what the ground is.

## Requirements

### Requirement: Generation is deterministic and order-independent
Terrain SHALL be a function of integer coordinates and the seed alone. Visiting
cells in a different order, or generating a chunk before or after its
neighbour, SHALL produce identical results. Changing the seed SHALL change the
world.

#### Scenario: Two visit orders
- **WHEN** the same region is generated in two different orders
- **THEN** every sample matches exactly
- **AND** a different seed produces a different world

#### Scenario: Across a chunk boundary
- **WHEN** samples are taken on both sides of a chunk edge, including negative
  chunk coordinates
- **THEN** the columns are continuous across the edge
- **AND** each side reconstructs the same sample the other side sees

### Requirement: Heights are quantized and finite
Surface elevation SHALL be quantized to a single named step and SHALL be finite
for every direction, including a degenerate input. The step SHALL be declared
once, beside the body radius, rather than inline in the generator.

#### Scenario: Sampling the globe
- **WHEN** elevation is sampled over the whole sphere
- **THEN** every value is finite and an exact multiple of the elevation step
- **AND** a zero-length direction still yields a finite height

### Requirement: The generator is versioned
The generator SHALL carry a version that forms part of a saved world's
identity. Changing the algorithm for an existing save without bumping that
version SHALL NOT be done. Each version SHALL name the configuration that
generates it, so a world keeps generating the terrain it was made with after
the shipped generator moves on.

#### Scenario: Reading a saved world
- **WHEN** a world was saved under one generator version
- **THEN** its identity records that version
- **AND** terrain produced under a different version is not silently treated as
  the same world
  (`saves::tests::a_world_of_a_version_this_build_lacks_is_refused_by_name`)

#### Scenario: A save from before the version was recorded
- **WHEN** a save with no recorded generator version is opened
- **THEN** it is read as version 4, and its terrain is version 4's, cell for
  cell (`saves::tests::a_world_from_before_identities_gains_one_when_opened`,
  `planet_gen::tests::version_4_makes_the_ground_every_old_world_was_made_on`)

#### Scenario: Every carried version keeps its ground
- **WHEN** a version this build carries is asked for its ground
- **THEN** its heights, biomes and top blocks are the ones its worlds were
  made on, pinned by a digest taken before the next version landed
  (`planet_gen::tests::version_4_makes_the_ground_every_old_world_was_made_on`,
  `planet_gen::tests::version_4_keeps_its_top_blocks`,
  `planet_gen::tests::version_5_makes_the_ground_its_worlds_were_made_on`)

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its save records the current generator version
  (`saves::tests::a_new_world_is_made_with_its_identity`)

#### Scenario: A version names its config
- **WHEN** the configuration for each shipped version is asked for
- **THEN** each version answers one configuration, and an unknown version is
  refused rather than read as the current one
  (`planet_gen::tests::an_unknown_generator_version_has_no_config`)

#### Scenario: A world is opened on its own generator
- **WHEN** a world made on another generator version than the running
  planet's is opened from the saves page
- **THEN** the planet switches to that world's version in place, without
  relaunching the game, and the world opens on its own ground with
  everything done in it: the config every reader asks for, the heights and
  the biomes are that version's, work begun on the old planet is not drawn
  on the new one, a version this build does not carry is refused, and the
  rebuilt planet carries the world's edits
  (`tests/generator_switch.rs::a_load_switches_the_generator_in_place_and_back`,
  `planet::tests::a_rebuilt_planet_replaces_the_old_one_in_one_step`,
  `planet::terrain::tests::the_generator_switches_only_to_versions_this_build_carries`)

### Requirement: A world has both land and ocean
The shipped generator SHALL produce substantial continents and substantial
oceans rather than degenerating to one or the other.

#### Scenario: Sampling the shipped seed
- **WHEN** elevation is sampled across the sphere
- **THEN** more than a hundred samples are above sea level
- **AND** more than a hundred are below it
