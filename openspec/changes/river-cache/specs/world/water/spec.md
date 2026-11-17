# World: water (river-cache)

## ADDED Requirements

### Requirement: A world's drainage map is read from a cache when it has one

A version 8 world opened a second time SHALL read its drainage map from a
cache file instead of building it, when the file names the same generator
version, seed, river settings and format and its digest holds. It SHALL be
read or built before `READY`, where the map is built today.

#### Scenario: A second open

- **WHEN** a world whose drainage map was cached is opened again
- **THEN** the map is read from the file and equals the map a build makes,
  node for node

### Requirement: The drainage-map cache is derived data, safe to delete

The cache file SHALL never be a player record. A missing, mismatched or
damaged file SHALL be rebuilt and rewritten without failing the load; no
edit SHALL invalidate or change it; deleting it SHALL change nothing a player
sees.

#### Scenario: A cache of another world

- **WHEN** the cache file names another seed, generator version or river
  setting, or its digest fails
- **THEN** the map is built as if there were no file, the file is rewritten,
  and the world opens as it was

#### Scenario: The cache deleted

- **WHEN** the cache file is deleted between two opens
- **THEN** the second open builds the map, and every column, record and
  river is what the first open had
