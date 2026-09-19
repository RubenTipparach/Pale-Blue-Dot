# Tasks

## 1. The inventory, in the core

- [x] `pbd_core::inventory`: `Item` (a block of a material, or a tool), `Stack`
      (an item and a count), and `Slots<N>` holding ten of them. `give` fills a
      matching stack before an empty slot, `take` empties a slot when it hits
      zero, and a stack never exceeds its item's limit.
- [x] It goes in the core because what a slot holds and how a stack merges is a
      rule a future multiplayer has to agree about, and it depends on nothing
      but the material enum that is already there.
- [x] Tests: give merges then spills, take empties, a full store refuses, the
      selection wraps both ways.

## 2. The HUD

- [x] Delete the title, the survey-flight subtitle, the flavour line and the
      two-line keybinding wall.
- [x] The slot row: ten bordered squares on the bottom bar, the selected one
      lifted and outlined, a stack count when above one.
- [x] Thumbnails as UV rects into `tilesets/fields.png` on the shader's own 4x4
      grid, so a block's icon is the texture the ground is drawn with and no new
      art is needed. One table maps material to tile, shared with the shader's
      mapping rather than written twice.
- [x] The readouts move onto the bar; one context line replaces the wall, with
      the full binding list behind a key.

## 3. Selection

- [x] `1`-`9` and `0` select a slot; the wheel steps it and wraps. The wheel is
      already read by the camera zoom, so the two must not both act on one
      frame - the slot takes it while walking, the camera while flying.

## 4. Prove it

- [x] Captures: the HUD at the spawn, and a slot row with several block kinds in
      it so the thumbnails can be judged against the ground they came from.
- [x] A test that every material the terrain can show has a thumbnail tile, so a
      new material cannot ship as a blank slot.

## 5. Held for the columns change

- [ ] Digging and placing. The heightfield has no block to remove.
- [ ] The first tool, which is the first equipment worth a slot.
- [ ] A bag behind the ten slots, when there is more to hold than ten stacks.
