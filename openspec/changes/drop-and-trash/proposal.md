# Drop and trash: getting rid of items

## Why

On `tree-run-6`, 2026-11-20, the showcase bot mined for two hours toward
"The iron age" and gained no coal. After a day of digging, its 30-slot pack
was full of 99-stacks of sand, stone, soil and dirt, and the game had no way
to get rid of any of it. A player mining a while ends up the same.
Offered a drop or a trash slot, the owner chose "Both", and added: "You
should be able to drag stuff off the pack into the background".

## What changes

- **Q drops the held stack.** The stack in the selected hotbar slot drops
  whole, as a pickup floating 3.2 m in front of the player. That is past
  the 2.6 m magnet, so it is not pulled straight back. It is picked up
  again by walking to it, as any drop is, and it expires as any drop does.
- **Dragging a stack off the pack drops it.** With a stack on the pointer
  in the pack, a click on the background (no panel, no slot, no button
  under it) drops that stack the same way.
- **A trash slot.** One slot beside the hotbar line in the pack: a stack put
  into it is destroyed. A held stack clicked onto it is gone.
- Each is one durable save line: a drop with the pack after it (a new
  `toss` line), or the pack alone (a `pack` line) for the trash. An older
  build reading a newer save skips the `toss` line as damaged; nothing
  else changes.
- The bot clears junk (sand, soil, dirt, grass) into the trash when its
  pack has fewer than two free slots, before it digs or mines.
