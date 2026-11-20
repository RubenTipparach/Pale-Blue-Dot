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
