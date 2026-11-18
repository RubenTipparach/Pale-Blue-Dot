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
