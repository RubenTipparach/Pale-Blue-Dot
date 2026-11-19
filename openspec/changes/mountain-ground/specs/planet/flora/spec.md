## ADDED Requirements

### Requirement: Mountains grow alpine plants in the clutter slots

On a Mountains cell, the ground clutter SHALL draw alpine pieces in its
existing slots (alpine flowers, cushion plants, dwarf juniper, heather,
bilberry, sedge, and a boulder on a scree cell), with no new clutter vertices,
and the rolls that decide whether a piece grows and what a hand takes SHALL
be the same as on any other cell. Every piece SHALL be lit as the terrain is
and drawn with no outline.

#### Scenario: The same rolls

- **WHEN** a grassy Mountains cell rolls a bush
- **THEN** the visibility pass, the CPU's rule and the gather rule agree it
  grows, and it is drawn as a juniper, heather or bilberry

#### Scenario: No new vertices

- **WHEN** the clutter draw's vertex count is read
- **THEN** it is the same as before the alpine pieces

### Requirement: Mountain rock carries lichen and scree, drawn

A Mountains rock face below the snow line SHALL show lichen in its own texels,
and a rock cell whose uphill neighbour stands at least 3 m above it SHALL draw
scree on its top. Both SHALL be decided at the draw from the cell and its
neighbours, so every world's mountains show them and no cell's material
changes.

#### Scenario: The foot of a cliff

- **WHEN** a Mountains stone cell has a neighbour 4 m above it
- **THEN** its top draws the scree tile, and its material is still stone

### Requirement: Snow lingers on shaded flats below the line

A flat Mountains cell within 14 m below the snow line, in shade, SHALL draw
ragged snow over its top without changing its material.

#### Scenario: Dig a patch

- **WHEN** the player digs a cell drawn with a snow patch
- **THEN** the block taken is the cell's own material, grass or stone
