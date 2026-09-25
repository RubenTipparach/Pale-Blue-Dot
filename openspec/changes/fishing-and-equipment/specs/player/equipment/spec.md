# Equipment Specification

## MODIFIED Requirements

### Requirement: Breaking a block takes time, set by the tool in hand
Taking a layer SHALL require holding the use button on the same layer for the
break time of its material with the tool in hand. The break times SHALL be a
matrix of one time per tool (shovel, pickaxe, axe) for each class of material
(dirt, stone, rock, ore, wood, placed), and the fastest tool for dirt SHALL be
the shovel, for stone, rock and ore the pickaxe, and for wood the axe. The
fishing rod SHALL NOT break any layer. Releasing the button or moving the aim
SHALL reset the progress. With the button still held, the next layer SHALL
start after a short pause. The matrix and the pause SHALL be validated data
with units (`assets/config/dig.ron`).

#### Scenario: Dirt with the shovel and with the pickaxe
- **WHEN** the player holds the use button on dirt with the shovel, then with
  the pickaxe, then with the axe
- **THEN** each takes that tool's time from the dirt row, and the shovel's is
  the shortest

#### Scenario: Each row's best tool
- **WHEN** the shipped matrix is read
- **THEN** the shovel is fastest on dirt, the pickaxe on stone, rock and ore,
  and the axe on wood

#### Scenario: Letting go early
- **WHEN** the button is released, or the aim leaves the layer, before the
  break time
- **THEN** nothing is taken and the progress starts again from nought

#### Scenario: The rod
- **WHEN** the rod is in hand and the use button is held on stone
- **THEN** no layer changes

#### Scenario: Holding on
- **WHEN** a layer breaks with the button still held
- **THEN** the next one starts after the pause, and a fresh press starts at
  once

## ADDED Requirements

### Requirement: The axe fells a tree, and the tree stays felled
Whether a cell carries a tree SHALL be decided by one rule in the core, and
the shader's selection SHALL be validated against it rather than written
separately. Felling a tree with the axe SHALL remove it, give its wood, and be
written to the save on the frame it happens.

#### Scenario: Felling and reloading
- **WHEN** a tree is felled and the world is reloaded
- **THEN** the cell draws no tree and the wood is in the slots

#### Scenario: The rule and the shader agree
- **WHEN** the core rule and the shader's selection are evaluated over the
  same cell IDs, top materials and biomes
- **THEN** they agree on every cell
