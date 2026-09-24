# Equipment Specification

## ADDED Requirements

### Requirement: One tool is in hand, in its own slot
The player SHALL hold exactly one tool (fishing rod, shovel, pickaxe or axe)
in a tool slot that is separate from the ten item slots. The tool slot SHALL
be drawn beside the item slots, with the held tool's own icon. A new world
SHALL start with all four tools owned and the fishing rod in hand.

#### Scenario: A new world
- **WHEN** a world is created
- **THEN** the tool slot holds the fishing rod
- **AND** the four tools are owned, and none of them occupies an item slot

#### Scenario: A saved world from before tools
- **WHEN** a world saved before this change is loaded
- **THEN** it is dealt the four tools with the rod in hand, exactly once

### Requirement: Holding G opens the tool picker; a tap still boards
On foot, holding G past the hold threshold SHALL open a picker beside the tool
slot that lists every owned tool by icon and name, with the held tool
highlighted. While it is open, the mouse wheel SHALL move the highlight and
SHALL NOT change the item slot or the camera zoom. Releasing G SHALL equip the
highlighted tool. A release of G before the threshold SHALL be a tap, and a
tap SHALL board or leave a craft as before.

#### Scenario: Changing to the shovel
- **WHEN** the player holds G for longer than the threshold, turns the wheel
  one step and releases G
- **THEN** the tool after the rod in the picker's order is in hand
- **AND** the selected item slot is the one selected before

#### Scenario: A tap beside the Loon
- **WHEN** the player taps G within reach of a craft
- **THEN** the player boards it and no picker opens

#### Scenario: Aboard
- **WHEN** the player is aboard a craft and holds G
- **THEN** no picker opens

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

### Requirement: A tool change is saved at once
Changing the tool in hand SHALL be written to the durable save on the frame
it happens.

#### Scenario: Equip and quit
- **WHEN** the axe is equipped and the process ends before any other event
- **THEN** the axe is in hand when the world is loaded
