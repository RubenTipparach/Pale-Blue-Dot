# Proposal: tools that stop digging, and bait by the water

## Why

Two bugs the owner reported on 2026-11-18:

- "It looks like some tools can get into a state where they no longer dig.
  I'm not sure what's causing this."
- "getting worms is not working for the mission. Worms should be found
  anywhere near rivers or oceans (in dirt)".

## What Changes

- **A felled tree no longer takes the dig.** The left button's tree path
  (`desktop/digging.rs`, `tree_on_ray`) hit-tests every cell that grows a
  tree by the seed's rule, felled or not. A felled tree is not drawn
  (`trees_taken`), but its invisible trunk and crown kept catching the ray:
  every hold near where a tree stood went to the tree path, the tool swung,
  "No more wood" came up once a stroke, and the ground was never dug. That
  is the "state": stand where you felled a tree, and no tool digs there. The
  hit-test now skips a tree whose wood is all taken, by the same rule the
  draw and the rooted ground already use.
- **Bait comes from earth by any water.** Today bait turns up only where the
  dug layer sits from 2 m under to 1 m over the sea's level, which no river
  inland ever is and which even a beach often is not. The new rule: digging
  grass, soil, dirt or sand turns up bait in one cell in `bait_one_in` (3)
  when a water cell, river or sea, stands within `bait_reach_cells` (3) cells
  of the dug cell, from 3 m under its layer to 2 m over it
  (`bait_water_m`). Never from a block a player placed. The nearness is one
  rule in `pbd_core::gather`, read on the column tier's own water cells.
- The mission's counting was checked and is right: a find is raised as
  `Gained { how: Found }`, `item_code` of the bait good is `bait`, and the
  mission's objective is `gather bait 3`. A test now pins three single finds
  closing it.

## Impact

- `pbd_core::gather`: `GatherSettings::bait` takes whether the ground is by
  the water instead of an altitude; `bait_band_m` is replaced by
  `bait_reach_cells` and `bait_water_m` in `gather.ron`. A find is not
  terrain and is not stored: no generator version moves and no save changes.
- `desktop/digging.rs`: `tree_on_ray` skips felled trees; the dig asks the
  core's nearness rule over the fine set's neighbour table.
- Saves: none touched. Tree records, edits and the hotbar read as before.
