# Design: a placed block keeps its material

## 1. Where the rule lives

In `pbd-core`, on the column, because the column is the one authoritative
description of what stands at every metre:

- `Column::place(layer, material)`: an edit's write. Marks the layer placed.
- `Column::set(layer, material)`: a rule's write. Clears the mark.
- `Column::placed(layer)`: marked and not air (a dug cell is a hole, not a
  block).
- `Column::drawn_material(layer)`: a placed layer is its own material even
  with water over it; only generated ground is dressed as a bed.
- `Column::drawn_sheet(layer, ground)`: `home_biome(material)` for a placed
  layer, `ground` (the cell's biome) for generated ground.
- `column::home_biome(material)`: the one table of which sheet a block comes
  from. Sand a beach's, snow the tundra's, jungle sward the jungle's, rock and
  the forge's bricks the mountains', water the ocean's, the rest (grass, dry
  grass, soil, dirt, stone, ore, tilled soil, the workbench) the meadow's;
  `None` for air and the lights. It is the table the pack's block icons used
  already, moved to core so the icon and the ground read one copy.

The mark is a 504-bit mask beside the layers (64 bytes a column). It is
derived state like the rest of the column: the save keeps `(cell, layer,
material)` as it always has, and replaying the save through `place` rebuilds
the mask. Nothing is written to disk that was not before.

## 2. Why the sheet, and not a new code

The column pass's per-layer word is a four-bit render code, and all sixteen
are taken or reserved. And the code was never wrong: `DIRT` is dirt. What
varied was the sheet, chosen per cell by its biome. So the fix carries a sheet
for exactly the layers that need one:

- **Column pass**: a second nibble buffer, the same layout as the codes, laid
  after every slot the material buffer holds (`PLACED_SHEETS_BASE =
  COLUMN_CAPACITY * MATERIAL_WORDS`). Nought for generated ground; the home
  sheet plus one for a placed layer. `gpu_placed_sheets` skips every column
  with nothing placed in it, so its CPU cost is a scan of each column's mask.
- **Cap**: the terrain pass draws a cell's top from its record, not from the
  column. Bits 26..29 of the record's material word carry the top's placed
  sheet plus one (`PLACED_SHEET_SHIFT`), set and cleared by
  `reconcile_surface`, which every edit and every rebuild of an edited cell
  already runs. The vertex shader passes them through in `out.material`.
- **Heightfield wall** (a cell with no column): its top metre takes the
  record's placed sheet; under that it is generated ground.

The biome byte is not touched: it still drives the foliage, tufts, tree
height and fallen leaves of the cell, and a column's generated layers under a
placed block still draw from it.

## 3. Saves

The save stores the material as placed and always has: no migration. A world
saved before this change opens with every placed block drawn as itself,
because the column is rebuilt from the same edits through `place`.

## 4. Cost

The material buffer doubles (4.1 MB to 8.3 MB of GPU memory at the tier's
capacity), and every fine-set upload writes one more `MATERIAL_WORDS` per
resident slot: about 252 bytes a slot, the same again as the codes. The frame
cost was not measured in this change: another agent was building on the same
machine, and the perf suite was not run. `tools/perf_suite.py` on the next
quiet sitting is the check.

## 5. Decisions

- Which sheet a placed block takes is the pack icon's, unchanged: the icon is
  what the player picked up, so the block put down looks like it.
  Recommendation taken (ask only with screenshots).
- A placed grass block still dries and frosts with the season
  (`ground_tint`): the season is the world's, not a change of material.
