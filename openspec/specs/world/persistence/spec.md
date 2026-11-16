# World Persistence Specification

## Purpose
What a saved world is made of. This spec holds the parts of
`world-persistence` that are built: so far, a world's identity. The rest of
that change (records, the authored journal, world processes, loading by
region) is designed in `openspec/changes/world-persistence` and not built.

## Requirements

### Requirement: A world's identity names every version that shapes it

A saved world SHALL record its seed, its topology version, its generator
version and the schema version of each record kind it holds. A save SHALL NOT
be opened under a version the running build does not have. A save that
predates a recorded version SHALL read as the version it was made under.

#### Scenario: A new world
- **WHEN** a world is made
- **THEN** its `identity.ron` is written first, with a barrier, naming this
  build's generator, topology and cell-key versions
  (`saves::tests::a_new_world_is_made_with_its_identity`,
  `saves::format::tests::an_identity_round_trips_and_names_this_builds_versions`)

#### Scenario: A save from before identities
- **WHEN** a save with no identity is opened
- **THEN** it reads as generator version 4 and topology version 1, and the
  identity is written to it before anything else is; a log migrated to exact
  keys on that open is recorded as exactly keyed
  (`saves::tests::a_world_from_before_identities_gains_one_when_opened`)

#### Scenario: A version the build does not have
- **WHEN** a save records a generator, topology or cell-key version this build
  does not carry
- **THEN** it is refused, the saves screen names the version, and the save is
  left as it was
  (`saves::tests::a_world_of_a_version_this_build_lacks_is_refused_by_name`,
  `saves::format::tests::a_version_this_build_does_not_have_is_refused_by_name`)

### Requirement: A rebuilt tier holds every saved change

A fine set that lands, and the first set a world opens on, SHALL hold every
change the save holds for the cells it has columns for, including changes
accepted while it was being built, and SHALL draw an edited cell's surface as
the edit drew it. A saved change a set does not hold SHALL be logged as an
error naming the cell, the layer, what the save holds and what the set holds.

#### Scenario: A lamp placed while the tier is rebuilt
- **WHEN** lights, a dig and a block with a torch on it are accepted after a
  fine set was requested and before it lands
- **THEN** the landed set holds them all and equals a set built from the
  edits after, column, record and light; before the replay the check lists
  every one
  (`planet::lod::landing_replay::a_set_built_before_lamps_and_a_dig_lands_with_them`)

#### Scenario: Only what changed since is replayed
- **WHEN** the edits are compared with the copy a set was built from
- **THEN** the cells whose changes differ are listed in key order, and none
  when nothing changed
  (`edits::tests::the_cells_changed_since_a_copy_are_the_ones_it_has_not_seen`)
