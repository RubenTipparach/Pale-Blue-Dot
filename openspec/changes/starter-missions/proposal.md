# Starter missions

## Why

The owner, 2026-11-14: "yup, try to do all of step 1 over the next 24 hours,
when you think you are in a good spot, start scripting a sereis of missions
to accomplish like "start a fcrafting table", make mining tools, make fishing
rood, make axe, make fire... etc". They added, the same day: "go fishing,
catch x fish, cook x fish", "farm x plants" and "finsd x seeds (seeds drop
from hitting grass or finding plants int he wild)".

The first game loop is crafting tools, fishing, farming, herding and cooking
(CLAUDE.md, "The first game loop"). A new player is not told any of it. A
chain of small missions walks them through the loop in the order that each
part unlocks the next.

## What Changes

- **A mission chain in data** (`assets/config/missions.ron`). Each mission
  has a short title, one line of help, and one or more objectives counted
  from things the game already records: an item made, a block placed, a fish
  caught, a crop harvested, an animal tamed, a dish cooked. A mission opens
  when the missions before it are done.
- **The first chain** (the owner's list, filled out along the loop):
  1. Gather by hand: dirt ×4, fibre ×3, a branch log.
  2. Make planks and rope.
  3. Make a crafting table and place it.
  4. Make mining tools: a stone pickaxe and a stone shovel.
  5. Make an axe, and fell a tree.
  6. Make a fishing rod.
  7. Make a fire: craft a campfire and light it.
  8. Go fishing: cast at water.
  9. Catch fish ×3.
  10. Cook fish ×3 at the campfire.
  11. Find seeds ×4: from hitting grass, or by picking wild plants
      ("finsd x seeds (seeds drop from hitting grass or finding plants int he
      wild)").
  12. Till farmland, and plant seeds ×4.
  13. Farm plants: harvest crops ×4.
  14. Tame an animal.
  15. Build a pen and keep an animal in it.
  16. Collect a product (an egg, milk or wool).
  17. Build a furnace and smelt ore into ingots ×3.
  18. Make a bronze or iron tool.
  19. Cook a meal from the farm and the herd.

  The counts (the owner's "x") are data, and are picked so that each mission
  takes one to three minutes.
- **A missions panel** in the pack, and a one-line tracker on the HUD for
  the current mission, in the stone-and-wood kit. A finished mission gets a
  short chime (the Sounds page's `ui_*`) and a line in the log.
- **Saved with the world** as a player record: which missions are done and
  the counts so far. These are durable, like every other player change.

## Capabilities

- `player/missions` (new): the chain, objectives, the record, the panel and
  the tracker.

## Impact

- `pbd-core`: `missions` (the chain, objective matching on game events,
  progress).
- `pbd-app`: events from crafting, dig and place, fishing, farming,
  husbandry and cooking fed to the missions system; the panel; the HUD line.
- **Saves:** a new `missions` record. An old world has none and starts the
  chain. Missions whose state already holds are completed on open, so a
  world that owns tools is not told to make them. These are: an item in the
  pack or owned, or a block placed, checked against the world's current
  state.
- **Order:** this is written now and built last in phase 1, once crafting,
  farming, herding and cooking exist to count.
