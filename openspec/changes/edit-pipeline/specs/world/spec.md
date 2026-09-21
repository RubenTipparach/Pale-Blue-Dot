# World: the edit pipeline

## ADDED Requirements

### Requirement: An edit costs what it changed

Applying one world edit SHALL NOT copy or re-derive state the edit could not
have reached. Specifically it SHALL NOT duplicate the resident cell set, and it
SHALL NOT re-derive light outside the region light can travel to from the
changed cell.

#### Scenario: Digging one block does not copy the world

- **WHEN** a single layer of a single cell is changed
- **THEN** the records rewritten are the edited cell's, its face neighbours' and
  its surface record, and no other record is written or copied

#### Scenario: A bounded re-bake agrees with a full one

- **WHEN** light is re-baked over the region within the propagation range of a
  changed cell
- **THEN** every cell of the tier holds the value a full bake of the same tier
  would give it

#### Scenario: Holding the dig button keeps the frame

- **WHEN** edits are applied on consecutive frames
- **THEN** the per-edit cost on the frame the player clicked is a small fraction
  of a frame at 60 Hz

### Requirement: An edit is accepted and sequenced before it is applied

An edit SHALL be validated, recorded in the durable ordered log and given a
sequence before any derived state is changed, and the code that changes derived
state SHALL be one implementation that does not know whether the edit came from
this player's input or from elsewhere.

#### Scenario: A local click and a remote edit take one path

- **WHEN** a sequenced edit is applied
- **THEN** the columns, the surface record, the contact and the light are
  changed by the same code whatever the edit's origin

#### Scenario: Replaying the log rebuilds the same world

- **WHEN** a set of edits is applied in log order from a freshly generated tier
- **THEN** the resulting tier is identical to the one that had those edits
  applied one at a time as they were made

#### Scenario: Two edits to one cell resolve by order

- **WHEN** two edits name the same cell and layer
- **THEN** the one later in the log decides what stands there

### Requirement: Only the changed records reach the GPU

An edit SHALL upload the cell records, column words and light words for the
slots it changed, at their own offsets, rather than rewriting the resident set.

#### Scenario: An edit's upload is bounded

- **WHEN** one block is broken
- **THEN** the bytes written to the cell, column and light buffers are those of
  the changed slots and not of the whole resident set

#### Scenario: A new tier still uploads whole

- **WHEN** a tier is rebuilt at a new anchor
- **THEN** every level's records are written, by the same upload function
