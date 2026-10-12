# Terrain Specification

## ADDED Requirements

### Requirement: The land is several masses, not one
The continent field SHALL produce a body whose land is divided into multiple
separate masses with open ocean between them, rather than a single connected
supercontinent. No one mass SHALL hold the great majority of the land.

#### Scenario: Flood-filling the land
- **WHEN** the land cells of the body are grouped into connected masses
- **THEN** the largest mass holds well under half of all land
- **AND** several masses each hold a substantial share of it

#### Scenario: Islands exist
- **WHEN** the same grouping is taken
- **THEN** there is a long tail of small masses beyond the largest few
- **AND** they are separated from the continents by ocean rather than joined by
  a one-cell land bridge

### Requirement: The land fraction stays in the band that keeps them apart
The share of the body above sea level SHALL stay low enough that the land does
not percolate into one mass. Raising it back above the percolation threshold
SHALL be treated as a regression, whatever the continent frequency is set to.

#### Scenario: A later tuning raises the land
- **WHEN** a change moves the land fraction above the configured band
- **THEN** a test fails, naming the land fraction and the largest mass's share
