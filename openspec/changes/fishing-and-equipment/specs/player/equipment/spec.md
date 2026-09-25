# Equipment Specification

## ADDED Requirements

### Requirement: Breaking a block takes time, set by the tool in hand
Taking a layer SHALL require holding the use button on the same layer for the
break time of its material. That time SHALL be the material's base time with
the right tool, and a fixed multiple of it with any other tool. The fishing
rod SHALL NOT break any layer. Releasing the button or moving the aim SHALL
reset the progress. The base times and the multiple SHALL be validated data
with units.

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

### Requirement: The block being broken shows cracks that grow with progress
While a layer is being broken, its visible faces SHALL show a crack overlay
chosen from ten stages by the fraction of the break time elapsed. Each stage
SHALL contain every crack pixel of the stage before it. The overlay's pixels
SHALL use the terrain's own face UV mapping and texel size, so a crack pixel
covers exactly one block pixel. Nothing SHALL be drawn when nothing is being
broken.

#### Scenario: Halfway through stone
- **WHEN** the pickaxe has been held on stone for half its break time
- **THEN** stage 5 of 0 to 9 is drawn over that block's faces

#### Scenario: The stages grow
- **WHEN** the shipped stage images are compared in order
- **THEN** every crack pixel of stage `k` is a crack pixel of stage `k + 1`

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
