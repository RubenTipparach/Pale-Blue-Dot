## MODIFIED Requirements

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

#### Scenario: A save from before the version was recorded
- **WHEN** a save with no recorded generator version is opened
- **THEN** it is read as version 4, and its terrain is version 4's, cell for
  cell

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its save records the current generator version

#### Scenario: A version names its config
- **WHEN** the configuration for each shipped version is asked for
- **THEN** each version answers one configuration, and an unknown version is
  refused rather than read as the current one
