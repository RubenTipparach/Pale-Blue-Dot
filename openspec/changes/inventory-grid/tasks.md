# Tasks

Asked for by the owner in survey L2 (2026-09-27). Survey I1: built now, while
the climate runs compute. I2: thirty slots. I3: a dig with a full pack drops
the block into the world as a floating block.

## 1. The pack in the core

- [x] 1.1 (Built as `put`, `take_all`, `take_half` and `send`, the four mouse moves of decision 5, beside `take`.) `inventory::Slots` grows to the hotbar's ten slots and a 30-slot pack (`PACK`), with `give` in the order decision 2 gives, `move_stack`, `split` and `take`. Verify: core tests of the give order, a give refused when all forty are full, a swap, a half split of an odd stack, and a stack never lost by a move.

## 2. The pack on the disk

- [x] 2.1 (`saves/format.rs`; the `pack` and `pick` lines and the drop field of 3.2 are in the same format and its tests.) The log line carries forty slot fields, and reads ten as an empty pack (decision 3). Verify: format tests that a 40-field line round-trips, that a 10-field line from before reads with an empty pack, and that a torn line is refused.
- [x] 2.2 (`Hotbar` still wraps `Slots`, which is now all forty, so no second type was needed.) `Hotbar` holds `Carried`; the kit deals through it and a grant that does not fit the hotbar lands in the pack. Verify: the kit tests extended with a full hotbar whose grant lands in the pack.

## 3. Drops (decision 4)

- [x] 3.1 (`pbd_core::drops`, the type named `ItemDrop` so it does not shadow Rust's `Drop`.) `pbd_core::drops`: a drop (item, count, where, the world time it was made), the magnet's pull, the pickup radius and the 300 s life, as pure functions. Verify: core tests that a drop inside 2.6 m moves toward the player and one outside does not, that one inside 1.2 m is picked up by the give order, that what does not fit stays on the drop, and that a drop past 300 s is gone.
- [x] 3.2 (`pbd_app::drops` and `digging::apply_edit`. The pickup, the full pack and the reload are tests (`drops::tests`, `saves::tests::drops_pickups_and_pack_moves_come_back`); a dig making one drop and no give is shown by the noon capture, the hotbar still at 64 grass with the block floating, since `apply_edit` needs a resident planet no unit test builds.) Every dig drops its block at the cell's centre instead of giving it; the dig's log line records the drop, and a pickup is its own line with the slots after it. A load restores the drops still inside their time. Verify: app tests that a dig makes one drop and no give, that a drop is picked up when the walker stands by it, that a full pack leaves it floating, and that a save and load keeps a floating drop.
- [x] 3.3 (`docs/screenshots/inventory-grid/`: `drops-noon.jpg`, `drops-midnight.jpg`, `drop-in-its-hole.jpg`. A light's card is built and tested but not captured, since no scripted dig takes up a lamp.) Drops drawn as small bobbing, turning hex prisms of their material, and a light as its icon on a card. Verify: a `--capture` of a few drops beside the walker by day and at night, in `docs/screenshots/inventory-grid/`.

## 4. The grid

- [x] 4.1 (`desktop/pack.rs`; the grid is the pack's thirty with the hotbar's ten under them, and `--menu pack` opens it for a capture.) `desktop/pack.rs`: the grid under the hotbar, drawn with `slots.rs`'s square and thumbnails, opened with I and closed with I or Escape. `MenuOpen` holds the pointer while it is open. I is in `BINDINGS`. Verify: app tests that I opens and closes it, that walking input is ignored while it is open, and that the controls list finds I.
- [x] 4.2 (`pack::click` and `Slots::shift`: a stack stays in its slot until the second click, which is the whole move and one `pack` line, so closing with a stack in hand moves nothing.) Click, shift-click and right-click moves (decision 5), each through the durable path. Verify: app tests of each move and of a held stack going home when the pack closes.
- [x] 4.3 (`pack-kit.jpg`, and `pack-after-dig.jpg`, which shows a dig pulled into the pack rather than a mouse move: a capture has no pointer.) Captures: the pack open with the kit in it, and after a few moves. Verify: in `docs/screenshots/inventory-grid/`.

## 4b. Blocks as the ground draws them (decision 7, survey I4)

- [x] 4.4 (`slots::block_art`, `drops::PrismFaces`, one prism mesh per block kind on one untinted atlas material; `a_block_is_shown_as_the_ground_draws_it` and `a_grass_drop_wears_its_top_side_and_underside`; `block-sides-*.jpg`.) A slot's picture is the block's side tile, untinted; a drop's prism wears top, side and underside tiles. Verify: an app test that grass's slot tile is the sheet's transition tile, and captures of the hotbar and a drop.

## 4c. Bare hands (decision 8)

- [ ] 4.5 `Equipment` holds a tool or nothing; a new world's kit holds nothing; the roll-out starts with "Bare hands"; bare hands dig nothing and draw nothing in hand; the `hand` line writes `-` for them. Verify: the tests and the capture decision 8 names.

## 5. The owner's check

- [ ] 5.1 The gate video (`step-videos`, in the owner's batch): the pack opened, stacks moved, a dig pulled into the pack, and a dig with everything full left floating.
- [ ] 5.2 The owner approves. Verify: the quote is in `proposal.md`. Then sync `player/inventory` and archive.
