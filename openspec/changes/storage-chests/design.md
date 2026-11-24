# Design

## 1. The block and its recipe

`Material::Chest`, a station row `chest` (no jobs: `max_jobs: 0`), placed and
taken up as a workbench is. Recipe at the workbench: 8 planks, 4 s, one
chest. Its look: a small wooden box block from the workbench's own planks
texture with an iron hasp, drawn as a tile in the block atlas and its icon
in the item sheet (art spec, pixel-art, no outline in the world, lit as the
terrain). Recommendation taken (ask only with screenshots): the look is
shown to the owner in screenshots once it is in the game, and changed on
their word.

## 2. What it holds

Twenty slots (two rows of ten, twice a raft's hold), the hold's own type
(`vehicle::hold::Hold` widened to a length, or a `Store` with the same moves),
so the pack-and-hold panel and its stack moves serve both. A chest's
contents are a durable record, kind `chest`, id `station::record_id(cell,
layer)`, written on every move with the pack in the same transaction, as the
hold's moves are (the hold's `store_with_hands`).

## 3. Opening it

F aimed at a chest within the hold's reach (3.5 m) opens the pack with the
chest's slots in the hold panel's place, titled with the chest's name from
the locale file. Walking out of reach closes it, as the hold does.

## 4. Breaking it

Digging a chest drops each stored stack as a pickup at its cell, then the
chest's own item, in one transaction (the drops written before the record is
cleared, so a crash keeps the items).

## 5. No trash

The trash slot and its save path are removed from the pack (`drop-and-trash`
decision 3 reversed by the owner). A save holding no trash state needs no
migration: the trash kept nothing.

## 6. The bot

`make_room` (the bot's pack-full step) no longer trashes. When fewer than
two pack slots are free before a dig, mine, chop or harvest:
1. the nearest chest the bot has set down within 60 m, else
2. a chest made (planks from logs if needed, at the nearest workbench) and
   set down where it stands;
3. walk to it, F, move each full junk stack (sand, soil, dirt, grasses,
   desert sand, stone, rock) into it, close.
The `select` step's own trash path (a full pack and hotbar) does the same.

## 7. As built (tasks 1.1-1.5, 2026-11-22)

Where the build differs from, or settles, the plan above:

- **The look is not a new atlas tile.** The column's per-layer material word
  is four bits and full (`planet_terrain::render_code`), so a chest is drawn
  as the workbench is: the local trees' wood (`WOOD`), with an iron strap
  across the lid and a dark iron hasp at its edge stood on it
  (`farm::show`), lit by the field as the ground is and with no outline. A
  floating chest drop wears the workbench's boards; its slot shows its own
  icon (`assets/items/stations/chest.png`, drawn by
  `tools/gen_item_icons.py`, provenance in `assets/items/goods/PROVENANCE.md`).
  Recommendation taken (ask only with screenshots): shown in task 2.2.
- **Codes.** `Material::Chest` is 30 (after desert sand's 29), in the save's
  material table too; the station row `chest` has `max_jobs: 0`; the recipe
  `chest` is 8 planks, 4 s, at the workbench, giving `block:chest`.
- **The store** is the hold's own type widened to carry its length:
  `vehicle::hold::Hold.slots` is a `Vec` (a raft's `Hold::default()` is still
  ten), and a chest is `Hold::with_len(pbd_core::chest::CHEST)`, twenty.
  `shift` and `send` bound a move by the store's own length. The record
  (`pbd_app::chests`) is kind `chest`, schema 1, id
  `station::record_id(cell, layer)`, its body the pack line's stack texts, as
  the hold's.
- **Opening.** F on a chest is the farm's `Open` act (the prompt reads "Open
  chest"); it opens the pack with the chest's panel (two rows of ten, under a
  CHEST lintel, on the stone frame) where a raft's HOLD would be, when the
  walker's centre is within 3.5 m of the chest's (`pack::chest_distance`);
  further, the note says "Too far". F again, I, Esc, the chest being dug, or
  the walker leaving the 3.5 m all close it. A chest open hides a raft's hold.
- **Breaking.** `digging::apply_edit` spills a chest before its own line:
  each stack a drop of its whole count at the chest's centre, each written as
  a toss line with the slots unchanged, then the record written empty (records
  are never removed), then the dig's line drops the chest itself. The spilled
  drops float and expire as any drop does (five minutes).
- **No trash.** The plaque, `TrashCell`, its press path and the `pack.trash`
  key are gone. The trash wrote only the pack line it shared with every move,
  so an old save has nothing to skip. The showcase's `Click::Trash` is gone
  too; its two steps do nothing until task 2.1.

## 8. The bot, as built (2026-11-22)

`make_room` stores: the nearest chest the bot set down within 60 m, walked
to, opened with F, each junk stack (the ground's blocks, any count, the
biggest first) clicked out of the pack and into the first chest slot that is
empty or holds the same block short of a stack, until three slots are free;
then Escape. With no chest near: a junk stack dropped with Q for room, then
the acts Craft("planks", 2), Bench("chest", 1), Hand("hand"),
Place("block:chest"), and the store after. `select` with pack and hotbar
both full makes room the same way before its swap. Chop joins dig, mine and
harvest in making room first.

## 9. The storage mission (owner, 2026-11-22)

The owner: "we need a storage mission in the intro section of the game". A
trunk mission after the crafting bench, `storage` ("Somewhere to keep
things"): make a chest, set it down, and put a stack in it. A new mission
kind, `store`, counts a stack moved from the pack into a chest (`Did(Store,
<item>, count)` written as the move is stored, one per item the chest gained).
The objective names the ground's blocks and the trunk's goods, any of which
counts. Old worlds past the bench see it open at once; nothing else in the
chain moves (it unlocks nothing the chain needs).
