# Inventory Specification

## ADDED Requirements

### Requirement: The player carries ten slots
The player SHALL have ten inventory slots, each holding either nothing or a
stack of one item kind with a count. A slot SHALL be selectable, exactly one
SHALL be selected at a time, and the selection SHALL wrap in both directions.

#### Scenario: Taking a second stack of the same block
- **WHEN** a block is given and a slot already holds that block below its limit
- **THEN** it joins that stack rather than taking a new slot

#### Scenario: A stack at its limit
- **WHEN** a block is given and every matching stack is at its limit
- **THEN** it takes the first empty slot
- **AND** if there is none, the give is refused rather than silently dropped

#### Scenario: Emptying a slot
- **WHEN** the last item is taken from a stack
- **THEN** the slot holds nothing, rather than a stack of zero

### Requirement: A slot shows its item, never its name
A slot SHALL be drawn as a bordered square carrying the item's own thumbnail,
with a stack count only when the count is above one. An inventory SHALL NOT be
presented as a list of item names.

#### Scenario: A slot holding a block
- **WHEN** a slot holds a block of a material the terrain can draw
- **THEN** it shows that material's own texture as its thumbnail

#### Scenario: A material with no thumbnail
- **WHEN** a material the terrain can show has no thumbnail tile
- **THEN** a test fails, rather than the slot drawing blank
