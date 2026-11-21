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
the trash, close. Stone is kept: recipes take it.
