# Equipment Specification

## ADDED Requirements

### Requirement: The tool in hand is drawn as a hex-pixel model of its icon
The tool in hand SHALL be drawn in first person as a model built from its own
16 px icon: each opaque pixel a hexagonal prism one pixel deep on an offset
hex grid, in that pixel's colour, with no face drawn between two hexels. The
model SHALL be built from the icon file itself, so the two cannot differ. It
SHALL swing while a block is being broken. The fishing line SHALL leave from
the rod model's tip.

#### Scenario: Changing tools
- **WHEN** the player equips the pickaxe
- **THEN** the pickaxe's hex model is in hand and no other tool's is

#### Scenario: A thin handle
- **WHEN** an icon has a one-pixel diagonal handle
- **THEN** its model's handle is one connected piece

#### Scenario: The line and the rod
- **WHEN** a line is cast
- **THEN** it starts at the tip of the rod model being drawn

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
