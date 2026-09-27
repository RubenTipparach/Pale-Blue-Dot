# Tasks

Asked for by the owner in survey L2 (2026-09-27). When it is built is survey
I1. Nothing here starts before the owner schedules it.

## 1. The pack in the core

- [ ] 1.1 `inventory::Carried`: the hotbar's ten slots and a 30-slot pack, with `give` in the order decision 2 gives, `move_stack`, `split` and `take`. Verify: core tests of the give order, a give refused when all forty are full, a swap, a half split of an odd stack, and a stack never lost by a move.

## 2. The pack on the disk

- [ ] 2.1 The log line carries forty slot fields, and reads ten as an empty pack (decision 3). Verify: format tests that a 40-field line round-trips, that a 10-field line from before reads with an empty pack, and that a torn line is refused.
- [ ] 2.2 `Hotbar` holds `Carried`; the kit deals through it and a grant that does not fit the hotbar lands in the pack. Verify: the kit tests extended with a full hotbar whose grant lands in the pack.

## 3. Nothing dug is lost

- [ ] 3.1 `apply_edit` refuses a dig whose block cannot be carried, before the save, and the HUD says "your pack is full" (decision 4). Verify: an app test that a dig with forty full slots leaves the cell and the log unchanged.

## 4. The grid

- [ ] 4.1 `desktop/pack.rs`: the grid under the hotbar, drawn with `slots.rs`'s square and thumbnails, opened with I and closed with I or Escape. `MenuOpen` holds the pointer while it is open. I is in `BINDINGS`. Verify: app tests that I opens and closes it, that walking input is ignored while it is open, and that the controls list finds I.
- [ ] 4.2 Click, shift-click and right-click moves (decision 5), each through the durable path. Verify: app tests of each move and of a held stack going home when the pack closes.
- [ ] 4.3 Captures: the pack open with the kit in it, and after a few moves. Verify: in `docs/screenshots/inventory-grid/`.

## 5. The owner's check

- [ ] 5.1 The gate video (`step-videos`, in the owner's batch): the pack opened, stacks moved, a dig into the pack, and a dig refused with everything full.
- [ ] 5.2 The owner approves. Verify: the quote is in `proposal.md`. Then sync `player/inventory` and archive.
