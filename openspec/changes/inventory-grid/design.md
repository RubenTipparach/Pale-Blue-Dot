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
- **Dropped items on the ground**, and throwing a stack away. There is no
  item entity yet. A stack can be moved, never destroyed, until one exists.
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

**2. `Carried` in the core, and the give order is one function.** The hotbar
and the pack are two arrays in one struct, because the order a give fills
them in is a rule a future multiplayer must agree on. The order:
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

**4. A full pack refuses the dig (survey I3).** The edit is refused before
it is saved, as a full tier refuses one today, and the HUD says "your pack is
full". *Alternative:* dig anyway and lose the block, as today. Rejected by the
inventory spec's own rule. *Alternative:* drop it on the ground. There is no
item entity (non-goal).

**5. Mouse moves, the way every block game does it.** Click picks up, click
puts down or swaps, shift-click sends between the hotbar and the pack,
right-click takes half. While a stack is held on the pointer and the pack is
closed, it goes back where it came from, so closing never loses a stack.

**6. I opens it.** I is free on foot and in flight. It is added to
`BINDINGS` for on foot. The pack does not open in a ship's seat, where the
hotbar is not drawn either.

## Risks / Trade-offs

- [A 40-field log line is longer to write on every dig] → A dig writes about
  400 bytes, where it wrote 100. The log is appended and flushed per edit
  already; it is not measured in the cloud session (CLAUDE.md), and the owner
  runs the performance suite on real hardware.
- [Refusing a dig with a full pack could surprise a player] → The HUD says
  why, and the pack is forty kinds of thing. It is survey I3.

## Migration Plan

- An old log reads unchanged: ten slot fields mean an empty pack.
- The kit gains nothing. A kit grant that does not fit the hotbar lands in
  the pack from now on.
- Rollback is the previous build, which cannot read a 40-field line.
