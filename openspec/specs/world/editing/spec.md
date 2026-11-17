# World Editing Specification

## Purpose
What a player's change to the ground is once it is made: the block placed
stays the block placed. Built by `placed-blocks-keep-material`; digging and
placing themselves are designed in `openspec/changes/dig-and-place`.

## Requirements

### Requirement: A placed block keeps its material

A block the player places SHALL be stored, drawn, dug, dropped, heard and
saved as the material they placed, on every biome and every planet, whatever
stands around or under it. Generation's surface rules (the biome's tileset,
desert sand, snow, a river's bed, a town's top) SHALL apply only to generated
ground, never to a placed block. A placed block SHALL be drawn from its own
sheet, the one its pack icon shows. Ground nobody touched SHALL be drawn as it
was.

#### Scenario: Any block on any ground
- **WHEN** any placeable block is placed on desert sand, on snow, in a river's
  bed, on a beach under the sea, or on a lane
- **THEN** the column holds it, knows it is placed, draws it as that material
  and from its own sheet, never the biome's
  (`column::placed_tests::a_placed_block_keeps_its_material_on_any_ground`)

#### Scenario: A block on a placed block
- **WHEN** a second block is placed on top of a placed block
- **THEN** both keep their own material and sheet
  (`column::placed_tests::a_placed_block_keeps_its_material_on_any_ground`)

#### Scenario: Under water
- **WHEN** a block is placed under a river's water
- **THEN** it is drawn as itself, not as a bed, while generated sward under
  the same water is still drawn as the bed
  (`column::placed_tests::a_placed_block_under_water_is_not_a_bed`)

#### Scenario: Saved, reopened and drawn
- **WHEN** blocks are placed, the save is written and the world reopened
- **THEN** the save holds each as placed, the tier's column holds it placed,
  the column pass draws its code from its own sheet, the cap's record carries
  its code and its own sheet, and the cell's biome is unchanged
  (`planet::column::placed_block_tests::a_placed_block_is_drawn_as_itself_after_a_reload`)

#### Scenario: Untouched ground
- **WHEN** a generated column and its record are drawn
- **THEN** no layer and no cap carries a placed sheet
  (`planet::column::placed_block_tests::untouched_ground_takes_its_biomes_sheet`,
  `column::placed_tests::only_an_edit_places`)

#### Scenario: Dug back up
- **WHEN** a block placed on desert sand is dug
- **THEN** it yields that block (tilled soil its dirt, ore its ore), and the
  sand under it is still sand
  (`desktop::digging::placed_tests::a_placed_block_on_desert_sand_digs_as_itself`)

#### Scenario: The slot and the block are one picture
- **WHEN** any block's pack slot and the same block placed are drawn
- **THEN** both come from the same sheet
  (`desktop::slots::tests::a_slot_and_a_placed_block_share_a_sheet`)
