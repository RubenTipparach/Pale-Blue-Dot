# Design: an inventory grid (the pack)

## Context

Observed on the branch, 2026-09-27:

- `pbd_core::inventory::Slots` is ten `Option<Stack>` and a selection.
  `give` fills matching stacks, then empty slots, and returns what did not
  fit. A block stacks to 99, a fish to 16, a tool to 1.
- The hotbar is `hotbar::Hotbar(Slots)`, a Bevy resource, drawn by
  `desktop/slots.rs` as bordered squares with each item's thumbnail.
- Every log line that changes what is carried (an edit, a kit, a catch)
  carries the ten slots whole (`saves/format.rs`), so a replay rebuilds the
  world and the player from the same lines. That is the save's central
  decision and the pack keeps it.
- `digging.rs` builds the slots after the edit, `moved.give(was, 1)`, and
  ignores what `give` returns. With a full hotbar, the dug block is lost.
- A menu that owns the pointer sets `controls::MenuOpen`. The field guide
  (J) is the model: it opens over the running world, and Escape closes it.

## Goals / Non-Goals

**Goals:**
- Room for more than ten kinds of thing, in a grid the player can sort.
- Nothing given to the player is ever lost silently.
- The same durable path: what is carried is on the disk the moment it
  changes.

**Non-Goals:**
- **Throwing a stack away** from the pack. Drops come from digging only.
- **Drops from anything but a dig**: a catch that does not fit still goes
  where `give` puts it, and a mob or a chest drops nothing, because there is
  neither yet.
- **Chests and other containers.** Those are `cities-in-the-world`'s and
  `player-building`'s, and they would reuse `Carried`'s stack moves.
- **Re-dealing the lights an old save could not fit** in kit grant 2. That
  deal is recorded as done. A new world has them all.
- **Crafting.**

## Decisions

**1. Thirty slots, three rows of ten (survey I2).** Ten wide, so the grid
lines up with the hotbar and its number keys. Thirty plus ten is forty kinds
of thing, which covers every block, every light and a creel of fish with room
over. A bigger pack is a constant.

**2. The core's `Slots` grows to forty, and the give order is one function.**
The hotbar is the first ten slots and the pack the next thirty, in one
struct, because the order a give fills them in is a rule a future
multiplayer must agree on. `SLOTS` stays the hotbar's ten, which the number
keys and the selection know; `PACK` is thirty. The order:
1. matching stacks with room, hotbar first;
2. the first empty hotbar slot;
3. the first empty pack slot.

A dug block goes to the hand before the pack, so what the player is building
with stays in reach.

**3. The log line carries forty slot fields, or ten.** A line written after
this change carries the hotbar and the pack; a line from before carries ten
and reads as an empty pack. It keeps the rule that one line holds a dig and
the block it yields. It costs about 300 bytes more per line with a full pack
(an empty slot is one character). *Alternative:* a separate pack line
written when the pack changes. Rejected: a dig that fills the pack would then
be two lines, and a crash between them is the torn state the log exists to
prevent.

**4. A dug block drops into the world, as Tenebris drops it (survey I3).**
The owner: "they can dig, but see how tenebris drops blocks into the world
like minecraft!", and "if pack is full, stuff just gets mined into the world
as floating blocks". Tenebris's drops (`tenebris-client/src/drops.rs`, read
at the pinned commit) are the reference:
- **Every dug block drops**, at the dug cell's centre, scattered 0.22 m so a
  pile does not stack in one spot. There is no gravity: a drop hovers where
  it was cut.
- **It looks like the block**: a small hex prism of its material, 0.16 m
  across the corners and 0.2 m tall, bobbing 5 cm at 2.2 rad/s and turning
  at 1.5 rad/s, each drop at its own phase. A light (a lantern, a candle)
  drops as its own icon on a card that faces the camera, as Tenebris draws
  non-block items.
- **A magnet brings it in.** Within 2.6 m of the player's chest (0.8 m over
  the feet) it is pulled at `7 (1 + 2.6 − d)` m/s. Within 1.2 m it is picked
  up by the give order of decision 2. What does not fit stays on the drop,
  floating, and is tried again every frame: a full pack is the owner's
  "floating blocks".
- **It lasts 300 s of world time**, as Tenebris's does, then goes. World
  time passes only while the world is played (CLAUDE.md), so a drop left at
  a quit is still there, with the same time left, at the next load.
- **It is durable, which Tenebris's are not.** Tenebris keeps drops in
  memory and loses them at a quit or a crash. Here every accepted change to
  the world reaches the disk at once (CLAUDE.md), and a floating block is
  the player's property lying in the world. So the dig's log line records
  the drop it made (its item, count and world time), and a pickup is a line
  of its own carrying the slots after it. A drop past its 300 s is not
  restored at load, so a despawn writes nothing.
- *Alternative:* drop only when the pack is full, and give straight to the
  pack otherwise. The owner pointed at Tenebris, which drops every block and
  lets the magnet bring it in; the magnet makes a dig beside the player
  feel instant anyway. Asked once in chat to be sure.
- *Alternative:* refuse the dig when the pack is full. The owner said to
  dig.

**5. Mouse moves, the way every block game does it.** Click picks up, click
puts down or swaps, shift-click sends between the hotbar and the pack,
right-click takes half. While a stack is held on the pointer and the pack is
closed, it goes back where it came from, so closing never loses a stack.

*As built (2026-09-27):* a stack picked up stays in its slot until the second
click, drawn at the pointer with its slot outlined, and the second click is
the whole move (`Slots::shift`), one `pack` line. So "goes back where it came
from" is simply never having left: a close, a crash or a refused save cannot
strand a stack on the pointer, and no line is needed for picking up. Half a
stack moves or merges and never swaps, since it has nowhere to put what it
would displace.

**6. I opens it.** I is free on foot and in flight. It is added to
`BINDINGS` for on foot. The pack does not open in a ship's seat, where the
hotbar is not drawn either.

## Risks / Trade-offs

- [A 40-field log line is longer to write on every dig] → A dig writes about
  400 bytes, where it wrote 100. The log is appended and flushed per edit
  already; it is not measured in the cloud session (CLAUDE.md), and the owner
  runs the performance suite on real hardware.
- [Hundreds of drops left floating after a long dig with a full pack] →
  Each is one small mesh on one shared material, and each goes after 300 s.
  A dig makes at most one. Not measured in the cloud session.
- [A pickup line per block doubles the log's lines while digging] → A pickup
  line is short (the slots, about 400 bytes with a full pack) and the log is
  appended per edit already.

## Migration Plan

- An old log reads unchanged: ten slot fields mean an empty pack.
- The kit gains nothing. A kit grant that does not fit the hotbar lands in
  the pack from now on.
- Rollback is the previous build, which cannot read a 40-field line.
