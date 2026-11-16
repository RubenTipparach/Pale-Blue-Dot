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

#### Scenario: Every carried version keeps its ground
- **WHEN** a version this build carries is asked for its ground
- **THEN** its heights, biomes and top blocks are the ones its worlds were
  made on, pinned by a digest taken before the next version landed

#### Scenario: A ground digest is pinned for each platform's maths library
- **WHEN** a carried version's ground digest is checked on macOS or on Linux
- **THEN** it is compared with the value that platform makes, both pinned
  beside each other, because the platforms' `sin`, `cos` and `pow` differ in
  the last bit and the digest folds every altitude bit for bit; the top
  blocks and the columns' materials agree on both and are pinned once
- **AND** a platform's value is pinned only once its samples are shown to be,
  byte for byte, what the commit that pinned the version made on that platform

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its save records the current generator version

#### Scenario: A version names its config
- **WHEN** the configuration for each shipped version is asked for
- **THEN** each version answers one configuration, and an unknown version is
  refused rather than read as the current one

#### Scenario: A world is opened on its own generator
- **WHEN** a world made on another generator version than the running
  planet's is opened from the saves page
- **THEN** the planet switches to that world's version in place, without
  relaunching the game, and the world opens on its own ground with
  everything done in it
