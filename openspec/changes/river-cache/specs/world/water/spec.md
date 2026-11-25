# World: water (river-cache)

## ADDED Requirements

### Requirement: A far river is drawn wherever it is in view

A river's water past the column tier, drawn as a sheet over each far cell at
the river's own surface, SHALL be listed for drawing whenever that sheet is
in the view, whatever the height of the river over the sea: the visibility
pass SHALL test the sheet's bounds at the radius the sheet is drawn at, not
on the sea's sphere. Proven by
`planet::visibility_tests::actual_gpu_lists_a_far_river_s_sheet_at_its_own_surface`
on the shipped shader (`river-cache` design 7).

#### Scenario: Looking down a river from its bank

- **WHEN** a player stands on the bank of a river 90 m over the sea and
  looks level down it, past the column tier's edge
- **THEN** the river's far sheet is drawn from the tier's edge outward, and
  its channel never reads dry while its water is in view

#### Scenario: The sea is unchanged

- **WHEN** a cell carries no river
- **THEN** its sea sheet is tested on the sea's sphere, as before


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
