# Proposal: a placed block keeps its material

## Why

The owner, playtesting on 2026-11-16: "I placed dirt on sand and it became
sand, dirt block should remain dirt block. this rule should remain universally
true anywhere on any planet", then "not just for dirt block but any block in
inventory". The screenshot shows a block put down from the pack on desert sand,
beside grass terraces, drawn with a sand top and sandy-striped sides.

## What was wrong (the finding)

The block was stored right and drawn wrong.

- **Stored right.** The edit is `(cell, layer, Material::Dirt)` in the save
  (`saves/format.rs`, `code_of`/`material_of` round-trip every material), and
  the column the tier builds from the save holds `Dirt` at that layer
  (`pbd_core::column::generate_edited`). Digging it reads that column
  (`desktop/digging.rs::material_of`) and yields `Dirt` (`yield_of`); the
  footstep reads the same column (`audio/hooks.rs::material_under`). So
  digging, the drop, the sound and the save all had the block placed. No save
  holds anything wrong, and no migration is needed.
- **Drawn wrong.** The record's cap code and the column pass's per-layer code
  were right too: `render_code(Dirt)` is `DIRT` (10). The wrong part was the
  tileset that code was drawn from. `planet_surface.wgsl` picks the sheet from
  the cell's BIOME for every face (`var slot = tileset_slot((input.material >>
  8u) & 0xffu)`), and every sheet keeps the same layout, so `DIRT` is tile
  (2,0) of whatever biome the block stands in. In the desert that tile is the
  desert sheet's "earth", which is sandy orange speckle, and stone's (3,0) is
  the desert's striped red sandstone. Every placed block took the sheet of the
  ground it landed on: sand on a meadow would draw the meadow's grass tile,
  stone on a beach the beach's.

  A second, smaller dressing applied to placed blocks too: a top with water
  standing on it is drawn as a bed (`Column::drawn_material`, `flora-v2`
  12.10), so a grass block placed under a river was drawn as dirt, and sand
  over the sea level under a river's water as the seabed.

## What changes

- A column knows which of its layers an EDIT wrote: `Column::place` writes a
  layer and marks it placed; `Column::set` (a rule's write: the water's flow,
  a test) clears the mark. `generate_edited` and `generate_edited_solid` apply
  a save's edits with `place`, and the tier's `set_layer` (the edit path and a
  landing's replay) does too, so a block is placed whether the tier was built
  from the save or the edit was made on this frame.
- A placed block is drawn from its own HOME sheet, `pbd_core::column::
  home_biome`, never the biome it stands in: the same table the pack's block
  icons read (`desktop/slots.rs::block_art`), so the slot and the block on the
  ground are one picture. Generated ground keeps its biome's sheet.
- A placed block is never dressed as a river bed.
- The sheet reaches the GPU two ways, for the two passes that draw a block:
  - the column pass reads a placed-sheet nibble per layer, laid in the
    material buffer after every slot's render codes
    (`ColumnTier::gpu_placed_sheets`, `PLACED_SHEETS_BASE`);
  - the terrain pass's cap reads four bits of its record's material word
    (`PLACED_SHEET_SHIFT`, 26..29), which `reconcile_surface` sets from the
    column's top.

Nothing about generated ground changes: the generator, its versions and their
digests are untouched, and a record or layer nobody placed carries nought and
is drawn exactly as before.

## Scope

- In: every placeable block (every material with a picture but the sea), on
  every biome and planet the tier draws, in the column pass and the cap.
- Out: the coarse levels beyond the column tier. They are generated
  heightfield records that do not carry edits at all today, so a placed block
  is not drawn there in any material; that is the same before and after.
- Out: ore. Generated ore is drawn as stone (its render code is stone's), and
  a placed ore block keeps that look, now from its own sheet. Giving ore a
  face of its own is a look change for the owner.
