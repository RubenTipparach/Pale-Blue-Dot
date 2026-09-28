# Proposal: an inventory grid (the pack)

## Why

**The owner (survey L2, 2026-09-27): "need to give the player an inventory
grid, just drop something, dont need those blocks in the inventory
really".** The player carries ten hotbar slots and nothing else. On
2026-09-27 the starting kit filled eight of them, so the five new lights
(`lamps-and-lanterns`) had room for only two in an old save. A new world's kit
now drops snow, rock and ore to make room (`lamps-and-lanterns` decision 12).
Something has to give whenever the game hands the player more than ten kinds
of thing: a kit grant, a dig, a catch, and later a city's goods and building
pieces.

**The owner's answers (survey, 2026-09-27):** I1 "recommended", so it is built
now, while the climate runs compute; I2 "recommended", thirty slots in three
rows of ten; I3 "they can dig, but see how tenebris drops blocks into the
world like minecraft!", and in chat, "if pack is full, stuff just gets mined
into the world as floating blocks".

There is a second reason. Digging a block with a full hotbar loses the block
silently today: `digging.rs` gives it to the slots and drops whatever did not
fit. The inventory spec says a give that does not fit is refused, never
silently dropped. The core does refuse it; the dig does not listen.

## What Changes

- **A pack** of 30 slots, three rows of ten under the hotbar, so a column
  lines up with the number key above it (survey I2).
- **I opens and closes it**, and Escape closes it. While it is open the
  pointer is free, the world keeps running, and walking input is ignored, as
  the field guide does it.
- **Moving stacks.** Click picks a stack up and click puts it down, swapping
  with what is there. Shift-click sends a stack between the hotbar and the
  pack. Right-click picks up half.
- **Where a give goes:** matching stacks with room first, hotbar then pack,
  then the first empty hotbar slot, then the first empty pack slot.
- **A dug block drops into the world** (survey I3), as Tenebris drops it: a
  small floating block of its material that a magnet draws to the player
  and the pack takes in. A full pack does not stop the dig; the block floats
  where it was cut until there is room, for 300 s of world time. Drops are
  saved, so a quit loses none.
- **The kit** deals into the pack what the hotbar cannot hold.
- **The save:** the pack rides the log line with the hotbar, as the hotbar
  does, so a dig and the block it yields reach the disk in one line. A line
  with ten slot fields, from before the pack, still reads.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `player/inventory`: the player carries a pack as well as ten slots; the
  pack opens as a grid of the same slots; a dug block drops into the world
  and is picked up.

## Impact

- **`pbd-core`:** `inventory::Slots` grows to the hotbar and the pack, with
  `give`, `move_stack`, `split` and the give order; `drops` holds a drop's
  rules (the magnet, the pickup, the life).
- **`pbd-app`:**
  - `hotbar.rs` holds `Carried`, and the kit deals through it;
  - `saves/format.rs` reads and writes 10 or 40 slot fields;
  - `desktop/digging.rs` drops the dug block instead of giving it, and a
    new `drops.rs` draws, pulls in and picks up the drops;
  - a new `desktop/pack.rs` draws the grid with `slots.rs`'s square and
    thumbnails;
  - `controls.rs` gains I in `BINDINGS`.
- **No new dependency.** Frame cost is a grid of 40 squares redrawn while it
  is open, and nothing while it is closed.
- **Save:** the log format gains the pack; an older build cannot read a line
  with 40 slot fields, as with any format added before.
