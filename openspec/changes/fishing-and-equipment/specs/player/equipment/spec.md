# Equipment Specification

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
