# Storage chests, in place of the trash slot

## Why

The owner, 2026-11-22, watching the bot stall at a tree with "No room in
the pack": "dont we have storage chests to craft?", "maybe we should add that
if we dont have them", "so the bot can no longer just trash items", and "I
would remove the trash icon and opt for a storage system instead, since
excess items can be refined in future updates, or traded for money".

The game has no container a player can make. The pack fills with 99-stacks
from mining; today's way out is the trash slot (`drop-and-trash`), which
destroys what a later update would give a use or a price.

## What changes

- **A chest**, made at the workbench from planks, set down as a block like
  any station. F on it opens it beside the pack, as a raft's hold opens
  today: its slots on one side, the pack on the other, stacks moved by click
  and drag with the hold's own moves.
- **A chest keeps what is put in it** across sessions: a durable record per
  chest (its cell and layer), every move one save line, as the hold's.
- **Breaking a chest drops what it holds** at the chest, then the chest
  itself; nothing in it is lost.
- **The trash slot goes.** The pack shows no trash; nothing in the game
  destroys an item. Dropping (Q, dragging off the pack) stays.
- **The bot stores instead of trashing.** When its pack is near full, it
  puts its junk stacks in the nearest chest of its own, making and setting
  one down by its workbench or where it works when it has none.

## Out of scope

- Selling, refining or sorting stored items (later updates, per the owner).
- Chests in towns, locked chests, larger chests.
