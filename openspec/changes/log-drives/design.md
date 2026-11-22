# Design (planned, not built)

## Sources

- M34 River Craft (`docs/mockups/river-craft.html`, `#logs`): logs rolled in
  from a rollway, carried by the current, caught by a boom, bundled.
- The flow: `water-flow` (the liquid rule's levels and directions on the hex
  grid) and the river's own surface and flow (`river-cache`).
- The raft and its hold: `small-boats` design 5.

## Decisions (to confirm when built)

1. A log in water is a floating body moved by the cell's flow vector,
   scaled by depth, with a little drag and spin; it bumps along banks.
2. It rests when the flow under it is below a threshold (still water) or the
   water under it is too shallow to float it; it starts again if the water
   moves again.
3. Logs are records (position, frame, state), written when they come to
   rest and at most every few seconds while moving, so a reload finds them
   where they were.
4. Loading a raft: logs move into the raft's hold slots (a log stack), and
   unloading drops them on the bank or into the water.
5. Booms and bundles (M34) follow, after the basic drive.

## As built (2026-11-21)

What there was to build on, found by reading the code:

- **Logs already exist.** The axe (or a bare hand) takes `Good::LOG` from a
  tree into the slots, a log a stroke, saved with the tree's `tree` record
  (`tool-crafting` decision 3). That is where a drive's logs come from.
- **The raft's hold already carries logs.** A raft's ten-slot hold is a
  `hold` record written with the slots in one line and moved through the
  boat's menu (C); `vehicles::hold`'s test already moves twelve logs into
  one. What was missing is taking a log out of the water into it, and
  putting one from it back into the water.
- **The current a craft feels is `vehicles::fresh_water`**: the fresh
  water's surface at a point, its slope as a current, and a river's own
  flow down its course from the drainage map (2 m/s, with a pull toward its
  line). A floating log reads the same function, so a log and a tube in the
  same water drift the same way: one rule, not two.

### L1. A log is a record, stepped by a pure rule

`pbd_core::log_drive`:

- `LogBody { position, heading, state }`: body-frame metres in f64 (a
  stored position, `world-frames` 10.4), its long axis, and its state:
  `Floating`, `Resting` or `Taken`.
- `step(log, water, dt)`, given what the water says at the log (its surface
  radius, its current, its depth under the log, or no water), moves it: its
  velocity eases toward the current's (`DRAG_PER_S`), it rides the surface,
  and its long axis turns toward the flow.
- It **comes to rest** when the current under it stays under `REST_MPS`
  (0.15 m/s) for `REST_S` (1.5 s), when the water is shallower than its
  draught `FLOAT_DEPTH_M` (0.3 m), or when there is no water under it (it
  has grounded on the bank). A resting log in water that flows again, over
  `WAKE_MPS` (0.3 m/s) and deep enough, floats again.

### L2. Saved where it stops, and while it moves

A log is a record of kind `log` (schema 1), its id the next free one in
the world. It is written:

- when it is made (thrown in, or unloaded off a raft);
- when it comes to rest, and when it starts again;
- at most every `SAVE_EVERY_S` (5 s) while it moves;
- when it is taken (state `Taken`, so the journal says where it went).

A reload puts each untaken log back where its last record says, so a log
is never lost; at worst it is a few seconds upstream of where it was.

### L3. Into the water and out of it

- **Q** with logs selected, aimed over fresh water, throws one log in as a
  floating log, written in one line with the slots (`store_with_hands`).
  Over land Q drops a stack as it always did.
- **F** on foot, near a log (the interact prompt names it), takes it back
  into the slots, in one line with the slots.
- **F aboard a raft**, near a log, loads it into the raft's hold.
- **U aboard a raft** puts one log from its hold over the bow, across it.

### L4. Drawn like everything else in the world

A log is a bark cylinder with cut ends (the `bark` and `timber` town
textures), lit by the field (`LitByField`, `LitLikeTerrain`).

### Risks

- A log loaded onto a raft is two lines: the hold with the slots, then the
  log's `Taken` record. The hold goes first, so a crash between them leaves
  a log in the hold and still in the water: one too many, never one lost.
- A log away from the column tier has no water to read and is not stepped;
  it waits where it is until the player comes back.

Not built here: booms and bundles (decision 5), jams, and the bot video
(task 2.4, left for the batch).
