# Surface Rendering Specification

## ADDED Requirements

### Requirement: A tree is built from the cell's own hexagon
Cosmetic foliage SHALL be drawn as hexagonal prisms on the cell's corner rays,
shrunk toward the cell centre by a per-part width factor, in Tenebris's own
terms: **0.20** of the cell for wood and **0.65 to 1.00**, hashed per layer, for
a leaf. It SHALL NOT be drawn as axis-aligned boxes sized independently of the
cell, and a crown SHALL NOT overhang its own cell.

#### Scenario: A tree on the finest tier
- **WHEN** a cell of the finest level carries a tree
- **THEN** its trunk is a hexagonal prism about a fifth of the cell across
- **AND** each leaf layer is a hexagonal prism between 0.65 and 1.0 of the cell
- **AND** no part of it extends beyond the cell's own hexagon

#### Scenario: Two leaf layers step rather than stamp
- **WHEN** a broadleaf tree's canopy is drawn
- **THEN** its two layers take different widths from a hash of the cell and the
  layer
- **AND** the same cell draws the same widths on every frame and every run

### Requirement: Tree height follows the material, in whole metres
A tree's trunk SHALL be a whole number of one-metre cells, `3 + a hashed bit +
a per-material extension`, matching Tenebris's per-biome table, so that a tree
is measured in the same unit as the terrain it stands on.

#### Scenario: A forest tree and a grass tree
- **WHEN** trees are drawn on the forest material and on grass
- **THEN** the forest tree stands 8 to 9 metres and the grass tree 5 to 6
- **AND** both are whole numbers of terrain cells
