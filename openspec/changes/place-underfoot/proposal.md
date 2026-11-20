# Place a block under your own feet

## Why

The owner, 2026-11-20, after the showcase bot was boxed in underground by
its own digging: "the easiest way to get unstuck underground is to build a
ladder up, or dig straight up and place dirt blocks below feet", then "game
should support placing blocks where the player stands and move the player up
when placing such blocks", "this will help ux and bot", and "there MUST BE
HEADROOM ABOVE TO PLACE A BLOCK ON YOUR FEET".

Today a block aimed into the cell the player stands in is refused ("is where
the player stands", `digging::occupies`). A player in a hole has to step
out, place and step back, or dig a stair.

## What changes

- A block aimed at the layer the feet stand in, in the player's own cell, is
  placed when the two layers over the feet's new height are open: the head
  room the player rises into. The player is lifted one block onto it. The
  eye eases up as on a step (the step ease, `walking.rs`).
- With no head room it is refused as today, with the note "No room above".
- A block aimed at the head's layer in the player's cell stays refused.
- The bot's "up and out" (`bot-path-planner` design 5.1) uses it: dig
  overhead, look down, place, again.
