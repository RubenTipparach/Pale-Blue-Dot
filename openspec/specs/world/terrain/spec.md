# Terrain Generation Specification

## Purpose
The surface is what a player walks on, digs into and looks across, and the
reference's generator is the look the owner asked for. Every height and
threshold is a field of one config with units, so a second body is a second
value and never a second function. A field is either planet-scale, whose size
is an angle, or land-scale, whose size is metres and whose frequency follows
the body's radius - which is what keeps a bigger world from being a smoother
one.

## Requirements

### Requirement: The terrain generator is Tenebris's
The surface altitude SHALL be Tenebris's generator ported term for term, with
each noise field declared as either planet-scale, whose size is an angle and
does not change with the body, or land-scale, whose size is a number of metres
and whose unit-sphere scale is derived from the body's radius. A land-scale
term SHALL carry its amplitude in metres rather than as a share of the relief
budget, so the summit and the roughness underfoot move independently.

#### Scenario: Mountains stand on land
- **WHEN** a cell is above the mountain elevation
- **THEN** no cell within three cells of it is below sea level

#### Scenario: Rivers reach the sea
- **WHEN** a cell is a river channel
- **THEN** a downhill walk from it ends below sea level

#### Scenario: The budget holds
- **WHEN** the sphere is sampled
- **THEN** the summit is between 130 and 180 m, the floor between 60 and
  110 m down, and land is between 40 and 60 percent of it

#### Scenario: The ground is as rough as the reference's
- **WHEN** adjacent land cells are compared over the sphere
- **THEN** at least a third of them differ by a block or more
- **AND** the finest feature in the height field is under twenty metres, which
  is under seven cells

### Requirement: The biome is a classification the surface reads
A cell's biome SHALL be one pure function of its direction (ocean, beach,
tundra, mountains, desert, swamp, jungle, fields), and the top block and the
tree density SHALL both read it. The moisture field that separates the
temperate biomes SHALL be land-scale, so a walk of a few hundred metres
crosses more than one of them.

#### Scenario: A beach is sand
- **WHEN** a cell classifies as beach
- **THEN** its top block is sand

#### Scenario: Every biome occurs
- **WHEN** the sphere is sampled
- **THEN** every biome and at least six top materials occur on it

#### Scenario: A walk crosses biomes
- **WHEN** a kilometre of land is walked in a straight line
- **THEN** it passes through more than one biome

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
