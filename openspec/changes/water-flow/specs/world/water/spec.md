# World: water

## ADDED Requirements

### Requirement: Flowing water is swum

A walker SHALL swim in flowing water as in a source. Built (`Column::solid`
and `water_surface` read `Material::is_water`); not yet pinned by a test of
its own, so it stays here.

#### Scenario: Wading a stream

- **WHEN** a walker stands in a flowing cell
- **THEN** the walker's water depth is the flowing cell's surface over its
  floor

### Requirement: A wooden bucket takes and pours a source

An empty bucket SHALL take a water source and only a source; a full bucket
SHALL pour a source into air and nowhere else, each as one edit with the
bucket's change on the same line. The rule is proven by
`a_bucket_takes_a_source_and_pours_into_air`; the place path is not yet.

#### Scenario: Carrying water up a hill

- **WHEN** a player fills a bucket at a pool and pours it on a hillside
- **THEN** the pool's cell is air, the hillside cell is a source, and the
  water runs from it by the rule
