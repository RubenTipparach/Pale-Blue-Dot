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
