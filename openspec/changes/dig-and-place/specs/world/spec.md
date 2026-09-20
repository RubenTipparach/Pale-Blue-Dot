# World: digging and placing

## ADDED Requirements

### Requirement: A block under the reticle can be taken

The player SHALL be able to remove the solid cell their view ray first meets,
within a bounded reach, and receive it as a stack in their inventory.

#### Scenario: The nearest solid cell is the one taken

- **WHEN** the view ray from the eye crosses air and then a solid layer
- **THEN** the layer taken is the FIRST solid one along the ray
- **AND** the material removed is added to the player's slots
- **AND** the column's own layer reads as air afterwards

#### Scenario: Nothing in reach is nothing taken

- **WHEN** the view ray meets no solid layer inside the reach
- **THEN** no layer changes and the slots are unchanged

#### Scenario: Bedrock is not a block

- **WHEN** the first solid layer along the ray is the bedrock floor
- **THEN** nothing is taken, because the world has a bottom

### Requirement: A block can be put back

The player SHALL be able to place the block in their selected slot into the
empty cell the ray passed through before the solid one it stopped on.

#### Scenario: The block lands in the air the ray last crossed

- **WHEN** the player places while a solid cell is targeted
- **THEN** the placed layer is the last air layer along the ray before it
- **AND** one is removed from the selected stack
- **AND** the column's own layer reads as that material afterwards

#### Scenario: An empty hand places nothing

- **WHEN** the selected slot holds no block
- **THEN** no layer changes

#### Scenario: A block cannot be placed inside the player

- **WHEN** the target air cell is one the player's own body occupies
- **THEN** no layer changes, because a player sealed into rock cannot move

### Requirement: An edit survives the tier being rebuilt

An edit SHALL be stored apart from the generated column, so that a column
regenerated after the player walks away and back carries the same edit.

#### Scenario: The same column generates the same way twice

- **WHEN** a column is generated, edited, and generated again from the same
  direction with the same edits
- **THEN** the second column equals the first after the edit

#### Scenario: An unedited column is untouched

- **WHEN** no edit exists for a cell
- **THEN** its column is exactly what the generator alone produces

### Requirement: An edit is durable on the frame it is made

Every accepted edit SHALL reach durable storage before the frame that made it
ends, and SHALL be replayed when the world is loaded again.

#### Scenario: An edit is on disk immediately

- **WHEN** a dig or a place is accepted
- **THEN** the edit is written and flushed to the save before the frame ends

#### Scenario: A reloaded world keeps its edits

- **WHEN** the world is loaded with a save that holds edits
- **THEN** every edited column generates with its edit applied
