# Equipment Specification

## ADDED Requirements

### Requirement: Breaking a block takes time, set by the tool in hand
Taking a layer SHALL require holding the use button on the same layer for the
break time of its material. That time SHALL be the material's base time with
the right tool, and a fixed multiple of it with any other tool. The fishing
rod SHALL NOT break any layer. The base times and the multiple SHALL be
validated data with units.

#### Scenario: Dirt with the shovel and with the pickaxe
- **WHEN** the player holds the use button on dirt with the shovel, then on
  dirt with the pickaxe
- **THEN** the second takes the wrong-tool multiple of the first

#### Scenario: Letting go early
- **WHEN** the button is released, or the aim leaves the layer, before the
  break time
- **THEN** nothing is taken and the progress starts again from nought

#### Scenario: The rod
- **WHEN** the rod is in hand and the use button is held on stone
- **THEN** no layer changes

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
