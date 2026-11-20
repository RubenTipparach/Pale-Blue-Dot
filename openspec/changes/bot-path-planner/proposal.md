# Bot path planner: plan in three dimensions, digging included

## Why

The owner, 2026-11-20, after the bot spent most of an hour boxed in 40 m from
a river it was sent to fish: offered a 3D path planner, a fresh world or
stopping, chose "Write up a path planner". Earlier: "is there anyway to
improve/plan pathing?", "your dude just walks into a wall", and the stuck
rule, "point towards where you want to go, and equip the right tool, shovel
or pick axe to mine your way".

Today's route (`river_walk::HexRoute::plan_where`) is a 2D search over hex
cells with one ground height each. It has no step limit, so a two-block pit
wall or a bluff reads as walkable, and it knows nothing of what is solid
above the ground (overhangs, the bot's own tunnels). When the walker is held,
`dig_through` guesses a block from the heading. Measured on `tree-run-5`,
2026-11-20: from a stone pit the bot dug itself, the guess named a grass
block while the look met stone, then a soil block above the head while the
look met stone in front; each fix moved the stall a few metres. The
remaining missions (fresh water, three crops, reef, town, boat) all need
walks of 40 m to kilometres over that kind of ground.

## What changes

- **A planner over cells and layers.** States are a cell and the layer the
  feet stand on, read from the live ground (the columns and the player's
  edits). Moves to a neighbour: walk (a rise of at most one block, the auto
  step, or a drop of at most three); tunnel (the feet's layer kept, blocks
  at the feet and head of the neighbour dug); stair up (one layer up, the
  blocks over it dug and the head-room over the walker's own cell). Each dig
  costs by its material, with the tool that digs it. Stations and lamps are
  never dug. Water deeper than a wade is not entered on a dry walk.
- **A plan is its steps.** Each step names the cell to stand in and the
  blocks to dig first. The bot does them in order: aim at each block, the
  right tool in hand, dig, then walk in (jump for a step up). Nothing is
  guessed on the spot.
- **Replanned as it goes.** 60 m at a time as today, and again when a step
  fails (a block will not break, the walker is held for 4 s).
- `dig_through` and its guesses go; `HexRoute` stays for the walker's own
  river walk.

- **Building its way out (owner, 2026-11-20).** "the bot neeeds good
  understanding of pathing or building its path out of being stuck", after
  it was boxed in underground by its own tunnels and flooded holes. The
  planner places blocks as a player does, carried soil or stone, besides
  digging. It can build:
  - a pillar up out of a hole: jump, a block set under the feet;
  - a step up a bank too tall to stair-cut;
  - a block over a gap or shallow water to cross it.
  Each placed block costs like a dig. A walk with no plan swims or builds up
  before it gives up.

## Not in this change

- Swimming routes (under water the bot only swims up and on).
- Travel by map beyond what the ground near the bot holds (towns km away
  are walked 60 m at a time, each leg planned when reached).
