# Design

## 1. The toss line

`toss d<id>:<stack>:<made_s>:<x>,<y>,<z> s0 .. s39`: the drop (the same text
as an edit line's drop) and the slots after it. Replayed as a drop added and
the slots set. `WorldSave::record_toss(drop, carried)`.

## 2. Q

A walking-mode key, `drop`, bound to Q in the controls list. The selected
hotbar stack is taken whole, a drop of it made at the chest plus the look's
level heading times 3.2 m, recorded, and added to the live drops.

## 3. The pack

- **Background:** while a stack is on the pointer, a left click with no
  pack slot, button or panel under the pointer tosses it, as Q does.
- **The trash cell:** a slot cell with the index `TRASH` (past every real
  slot), at the end of the hand-and-HOTBAR line. A click there with a stack
  held takes it out of the slots and records the pack.

## 4. The bot

Before a Dig or Mine act works, with fewer than 2 free slots carried: open
the pack, pick up a junk stack (sand, soil, dirt, grass of any kind), click
the trash, close. Stone is kept: recipes take it. Only a full stack (its
item's stack limit, 99) is trashed, never a handful: the owner, "you cant be
trashing stuff at 2, you should way at least till 64 or 99 whatever full
stacks are" (2026-11-20), after the bot trashed each freshly dug block.

## 5. A dropped item that is not a block shows its sprite (owner, 2026-11-22)

The owner, playtesting: "when dropping items, if the item is not a block
show sprite instead".

What a drop looks like today (`desktop/drops_view.rs`, `dress`):
- a terrain block: a small hex prism of its ground texture;
- a lamp and a good: a card that faces the camera, wearing its pack icon;
- a tool, a fish, and the station blocks (forge, workbench, chest): no
  look at all, so the drop floats there undrawn. A missing picture, but
  one the player meets when they drop an axe or a fish with Q.

Change: one rule for both the pack and a drop. An item with a picture of its
own in the pack (a fish, a tool, a lamp, a good, the forge, the workbench,
the chest) drops as a card wearing that picture. Only a terrain block, drawn
in the pack from the ground's atlas, drops as a prism.
- `slots::own_icon(item, icons)` is the one table. `paint_icon` uses it for
  slots and `dress` for drops, so a new item with an icon is never left
  undrawn on the ground.
- `dress` makes a card for every item `own_icon` answers: each block
  material, every good, every tool (`Tool::ALL`), and every fish in the
  roster (by index, as the pack draws them).
- A card is lit as the terrain is (`LitByField`, `LitLikeTerrain`), like a
  lamp's card today. It keeps its 1 px outline, since it is an icon
  (CLAUDE.md, "No outlines in the world").

Check: drop an axe, a fish, a seed, a workbench and a dirt block with Q, and
take a screenshot of each lying on the ground, in
`docs/screenshots/drop-and-trash/`.
