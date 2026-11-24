# Proposal: furniture

## Why

**The owner, 2026-11-23:** "Add furniture recipes to carpenter as a seperate
branch and make, add furniture to the town,add furniture to town. Make sure
NPCs and player can sit in chairs, beds. Allow storage in drawers, closest,
various chests. The player should be able to find things in chests and
storage stuff. Players can just take items for now. In the future we might
implement stealing mechanics later".

And in the same round: "Carpenter missions / And yes we need a carpentry
station for furniture".

Town houses are empty inside but for a hearth and the stairs.
`player-building` lists furniture in its scope and its "furniture in a wall"
check, with no tasks; no station makes furniture, nothing in the world can be
sat on, and nothing stores what a player does not carry. "Closest" is read as
**closet** (a wardrobe).

## What Changes

- **A carpenter's bench**, a station block made at the workbench. Every piece
  of furniture is made there and nowhere else: its recipes are their own
  branch ("Furniture") on the bench's panel, apart from the workbench's tools.
- **Ten pieces**: chair, stool, bench, table, bed, drawers, closet, and three
  containers (wooden chest, iron-banded chest, crate). Each is a good with an
  icon, a recipe and a model built as one shaped form (bevelled edges, tapered
  legs, a stuffed mattress), lit as the terrain is, with no outline.
- **Placed in build mode** with a Furniture tool: a piece stands in a cell,
  its back to one of the cell's sides (or in its middle, a table), checked by
  the building rules: not into a wall, not across a doorway, not on a stair,
  headroom over it, not overlapping another piece. Placing takes the piece
  from the pack; taking it away gives it back (a container only when empty).
- **Sit and lie down**: F on a chair, stool or bench sits the player; F on a
  bed lies them down. Moving or F again gets up. The eye drops to the seat's
  or the pillow's height. There is no sleep rule in the game, so a bed does
  not pass the night (design 6).
- **Townsfolk use them**: by night the townsfolk lie in the beds of their
  town; by day the ones at a sitting post, and a share of the idle, sit at
  the seats near their post. One rule of occupancy holds a seat or a bed for
  one body: two people never share one, and the player cannot sit where
  someone sits (or the other way round).
- **Storage**: F on drawers, a closet or a chest opens a two-sided panel like
  the pack's: the container's wells beside the carried ones; a click moves a
  stack across. The contents are a `store` record, written in the same line
  as the slots after the move, through the durable path.
- **Furnished towns**: every town house gains its furniture, laid by a rule
  from the house's own stored record (cells, doors, stair, hearth, storeys):
  derived, never saved, and it changes no wall, floor, door or anything a
  player built, so an old world's towns are furnished on load and nothing in
  them moves.
- **Town loot**: a town container holds seeded goods that fit the household's
  trade (a farmer's seeds and crops, a smith's coal and iron, a fisher's rope
  and bait, a trader's coins). The player may take it all, with no penalty;
  what is taken is remembered (the container's own `store` record) and a town
  container does not refill (design 9). Stealing is future work, not built.
- **A Carpentry branch in the town tree**: build the carpenter's bench, make a
  chair, a table, a bed and a chest, furnish a room, sit down, lie in a bed,
  and find something in a town chest; paying coins like the rest of the tree,
  counted on the game's own events (design 12, and a delta to
  `town-missions-tree`).
- **The mockups first**: a Furniture scene in M15 (`towns.html#furniture`)
  and the Carpentry branch on M33's Town tab.

## Capabilities

### New Capabilities

- `player/furniture`: the carpenter's bench and its recipes, the pieces,
  their placement, sitting and lying, storage and its save.
- `world/town-furniture`: furnished town houses, townsfolk in seats and beds,
  town loot and its memory.

### Modified Capabilities

- `town-missions-tree` (a change, not yet in the main specs): the Carpentry
  branch, as its own delta in that change.

## Impact

- **`pbd-core`:** `furniture` (kinds, models, footprints, occupancy, the
  `store` record and its moves, loot), `building::Shape::Furniture` and its
  rules, `settlement::pieces::furnish` (the town layout), a `Doing::Sitting`
  and `Doing::Lying` in the townsfolk motion, the carpenter's bench material,
  ten goods, the town tree's new kinds.
- **`pbd-app`:** the carpenter's panel, the Furniture tool in build mode, the
  sit and lie state, the townsfolk's seats and beds, the storage panel, the
  carpentry missions' events.
- **Data:** `recipes.ron` (the carpenter's station and eleven recipes),
  `town_missions.ron` (the Carpentry branch), `locale_en.yaml`.
- **Saves:** new record kinds only (`store`), and a new `piece` shape;
  nothing existing changes. An old world loads unchanged and its town houses
  are furnished from their own records.
- **Performance:** a town's furniture is cut into the house's room meshes it
  already draws, so no draw call is added per piece. Not measured in a cloud
  session (CLAUDE.md).
