# Design: furniture

## Context

The owner's request is in the proposal. What exists:

- **Stations** (`pbd_core::station`): a station is a block (`Material`) with
  a row in `STATIONS`; its recipes are the rows of `recipes.ron` naming its
  code; the workbench's panel is a grid of wells over the recipe row.
- **Goods** (`pbd_core::goods`): one append-only table saved by code; a good
  can be placed by a reader of its own (the fence, the incubator).
- **Player building** (`player-building`, built): pieces as `piece` records,
  `Shape` per class, checked by `building::rules` (one function per rule, the
  first broken is the reason) and cut by `building::cut` into the town's
  meshes and solids.
- **Towns** (`cities-in-the-world`, built): each town is a stored record laid
  once from its template; its buildings are cut by
  `settlement::pieces::cut_building` into outside and room meshes, solids, a
  hearth under the chimney, sconces and candles. Every cut is derived.
- **Townsfolk** (`townsfolk`, built): derived, never saved; each at a post or
  on a path, drivable by `Townsfolk::drive` (the lost villager).
- **F** (`interact`): one table of acts, the prompt names the winner.
- **Saves**: `WorldSave::store_with_hands` writes a record and the slots
  after it in one journal line; that is the durable path for a move between
  a container and the pack.
- **No sleep rule.** Nothing in the game passes the night.

Every choice below that the owner did not make is marked **recommendation
taken (ask only with screenshots)**; the mockup (M15's Furniture scene) is
what the owner looks at.

## Goals / Non-Goals

Goals: the carpenter's bench and its Furniture branch; ten pieces; placement
by the building rules; sitting and lying for the player and the townsfolk
with one occupancy rule; storage with a durable save; furnished town houses
with seeded loot the player may take; a Carpentry branch in the town tree.

Non-goals: stealing, ownership and the townsfolk minding what is taken
(future, decision 10); sleeping through the night (decision 6); furniture
that a player moves without taking it up; furniture outside houses in towns
(the inn's yard, the market's stalls stay as they are).

## Decisions

### 1. The carpenter's bench is a station block

`Material::Carpenter` (saved code 40: 30 to 39 are left to the paving
stations of `townsfolk` design 41, which are written up and not built), a
`carpenter` row in `STATIONS` with four jobs, drawn as the workbench's wood
with a plank top, a saw and a square stood on it. It is made at the
workbench (`carpenter-bench`: 8 planks, 2 logs, 1 iron ingot, 6 s) and placed
like the workbench. Its F opens its panel, the workbench's panel with its own
code: the lintel names it "Carpenter's bench" and the grid under it is the
**Furniture** branch, every recipe whose station is `carpenter`. No piece of
furniture is made by hand or at the workbench (owner: "we need a carpentry
station for furniture").

### 2. The pieces

| Code | Use | Footprint (along x deep), height, m | Recipe | Slots |
| --- | --- | --- | --- | ---: |
| `chair` | seat, 1 | 0.48 x 0.48, back 0.95 | 3 planks | |
| `stool` | seat, 1 | 0.40 x 0.40, 0.50 | 2 planks | |
| `bench` | seat, 2 | 1.30 x 0.42, 0.46 | 4 planks | |
| `table` | surface | 1.05 x 1.05, 0.76 | 5 planks | |
| `bed` | bed, 1 | 0.98 x 2.0, 0.55 (head 0.95) | 5 planks, 3 wool | |
| `drawers` | store | 0.90 x 0.48, 0.92 | 6 planks | 12 |
| `closet` | store | 1.04 x 0.58, 1.95 | 8 planks | 20 |
| `chest` | store | 0.88 x 0.50, 0.55 | 5 planks, 1 rope | 15 |
| `banded-chest` | store | 0.98 x 0.56, 0.62 | 5 planks, 2 iron ingots | 24 |
| `crate` | store | 0.62 x 0.62, 0.62 | 3 planks | 8 |

Each is a good (stack 4) with a 16 px icon in the UI's style (a 1 px
outline: icons are UI). Recipes take 3 to 6 world seconds.

**Models, one shaped form each.** A piece is made of bevelled solids (every
edge chamfered, so no face meets another at a hard box corner), legs that
taper from a wide shoulder into the top they hold, a seat with a rounded
front, a mattress and pillow that bulge and overhang the frame, a chest lid
that crowns, iron bands that wrap the lid's curve. Built in `pbd-core`
(`furniture::model`), in the town atlas's own textures (`timber`, `plank`,
`boards`, `linen`, `wool`, `iron`), so they take the town's room material and
are lit by its sun, shadows and cell light field like the walls round them.
No outline, no contrasting edge.

### 3. Where a piece stands

A piece stands in a cell at a layer, **its back to one of the cell's sides**
(`side` 0 to 5, the cell's own side order) or **in its middle** (`side` 6,
facing side 0; the table). Its footprint is a rectangle in a regular hexagon
of the standard cell (2.833 m across the flats): set against side `s`, its
back is the wall's inside face (half the 0.24 m wall in from the edge); in
the middle, its centre is the cell's. Several pieces share a cell, so a room
of one cell holds a bed, a chest and a stool.

`building::Shape::Furniture { cell, layer, side, kind }`, slot
`furniture:<cell>:<side>:<layer>`, kept as a `piece` record like every other
piece (schema 1: a new variant, which a build without it skips, as it skips
any body it cannot read).

### 4. The placement rules

The building rules gain furniture's cases, one function each, the first
broken being the reason (`build.refused.*`):

- **Outside**, **Slot**, **Pentagon**: as every piece.
- **HeldUp**: stands on the ground or a floor at its layer, as a lantern.
- **InWall** ("Into a wall"): its footprint inside the cell's hexagon less
  the wall's half thickness. Every piece fits against any side by its size,
  and a test holds that, so the rule refuses only a piece too big (none
  ships); it is kept so the rule exists for later pieces.
- **Overlap** ("Against another piece"): its footprint does not cross the
  footprint of another piece of furniture in the cell at its layer.
- **Doorway** ("In a doorway"): no footprint crosses the 1.1 m by 0.9 m
  clear space inside a door, of a door in any of the cell's walls on its
  storey.
- **Headroom**: no floor in the cell over it closer than the piece's height,
  and none of a stair's well.
- **Clear**: not on a stair's cells.

A town's furniture is laid by the same footprint and overlap functions on
the town's plan (decision 8), so a town room and a player's room obey one
rule.

### 5. Placing and taking up

Build mode gains a **Furniture** tool after the lantern. While it is chosen,
the wheel steps through the kinds of furniture carried (the HUD names the
piece and how many), R turns it a side, and a click places it; the ghost is
green or red with the reason. Placing takes one from the pack in the same
line as the `piece` record (`store_with_hands`). The Remove tool aimed at a
piece of furniture takes it up into the pack; a container with anything in
it is refused ("Empty it first").

### 6. Sitting and lying (the player)

F on a seat offers **Sit**, on a bed **Lie down** (the prompt names the
piece). Sitting holds the player where they stood and moves the eye to the
seat: 1.15 m over the floor for a chair or bench, 1.2 m for a stool, facing
the way the seat faces, the look still free. Lying puts the eye at the
pillow, 0.75 m over the floor, looking along the bed. Any movement key, jump
or F again gets up and gives the eye back. While seated the walker does not
move; the body is not moved into the furniture, so no solid pushes it out
and nothing about the walker's physics changes.

**There is no sleep rule in the game**, so lying in a bed does not pass the
night. When a sleep rule is written (survival's, phase 4), the bed is where
it goes. Recommendation taken (ask only with screenshots).

### 7. One occupancy rule (the player and the townsfolk)

`furniture::Occupancy` holds who is on each place of each seat or bed: the
player, or a townsperson by id. A bench has two places; everything else one.
`claim` refuses a held place, and a body holds one place at a time (a new
claim lets the last go). The player's Sit asks it first ("Someone is sitting
there"); a townsperson's seat or bed is chosen among the free places. The
places are named by the furniture's id: a player piece's record id, or a
town piece's id from its site, building and number.

### 8. Furnished town houses

`settlement::pieces::furnish` lays a building's furniture from its stored
record: its cells, doors, windows, stair cells, chimney and storeys. Derived
every time the town is cut, never saved; it adds faces to the room meshes and
solids, and changes no wall, floor, roof, door or anything a player built, so
**an old world's towns are furnished on load and nothing else in them
moves**. The rule, by the building's name:

- **A house** (house, cottage, hut, igloo, townhouse): on its ground storey a
  table in the middle of the cell with most neighbours inside the house
  (never the door's cell, a stair's, or the hearth's), chairs or stools
  against its free sides facing it, a bench where the house is wide; on its
  top storey (the ground one if it has one) a bed against an outer wall, a
  chest at the bed's foot, drawers or a closet against another outer wall.
  Every piece is tried by the footprint rules of decision 4 and left out
  where it does not fit.
- **An inn**: tables with benches on the ground storey, beds and chests
  above.
- **A hall** (hall, moot hall, exchange, caravan hall, longhouse): tables
  with benches, a banded chest.
- **A store** (boathouse, fish hut, tower, keep): crates and a chest.

Which kinds, and which of a choice, are drawn from the building's own seed
(its id), so the same house is furnished the same way in every build that
has this rule.

### 9. Town containers and their loot

A town container's contents, until the player first changes them, are its
**loot**: seeded by the container's id (site, building, number) and the
household's trade. A house's trade is a role of its town's people drawn by
the building's id; a hall or inn takes the town's trader if it has one. The
tables (`furniture::loot`): a farmer's seeds, wheat and turnips; a smith's
coal, iron and copper ingots; a fisher's rope, bait and fibre; a trader's or
merchant's coins and planks; anyone's planks, rope, fibre, logs and the odd
tool. Coins go to the purse when taken (`Reason::Found`).

**Taking is free, for now.** Nobody minds. The first move in or out writes
the container's `store` record with what is in it after, in the same line as
the player's slots; from then on the record is what it holds. **A town
container does not refill**: what is taken stays taken across a quit and a
reload. Recommendation taken (ask only with screenshots): a refill rule
(each world week, say) is for when towns trade among themselves, and would
make a free chest an income.

### 10. Stealing (future, not built)

The owner: "In the future we might implement stealing mechanics later".
What this change leaves for it: every town container's record names its
town and building, the player's moves are journaled under the player, and a
taker could be seen by the townsfolk near it. A stealing rule (goodwill lost,
a guard called) reads those; nothing here does.

### 11. Storage panel

F on a container opens **its panel**: the container's wells (its slot count,
eight to a row) above the carried slots, the pack's own wells and icons. A
click on a stack moves it whole to the other side's first room; shift-click
moves half. A town container's lintel names it and the house ("Chest, Timber
house"). Esc or F closes it. Every move is one `store_with_hands` line.

### 12. The Carpentry branch of the town tree

Off the trunk's "Take a job" (`first-job`), nine missions, 115 coins:

| Id | Mission | Counts | Coins | Unlocks |
| --- | --- | --- | ---: | --- |
| `carpenter` | A carpenter's bench | craft and place `block:carpenter` | 10 | |
| `chair` | Make a chair | craft `chair` | 10 | |
| `sit` | Take a seat | `sit` on any seat | 5 | |
| `table` | Make a table | craft `table` | 10 | |
| `bed` | Make a bed | craft `bed` | 15 | |
| `lie` | Lie in a bed | `lie` on a bed | 5 | |
| `chest` | Make a chest | craft `chest` | 15 | `banded-chest` |
| `furnish` | Furnish a room | `build` four pieces of `furniture` | 30 | `closet` |
| `find` | Look in a town chest | `loot` anything from a town container | 15 | |

New kinds: `sit` (a seat's kind or `seat`), `lie` (`bed`) and `loot` (`<site>:<thing>` of the town container's thing; `any:any` here), raised where the game seats, lays down and moves a stack out of a town container; `build` gains the furniture names and the group `furniture`. The banded chest and the closet are locked until their missions by the town tree's unlock rule.

### 13. Text

Every word from the locale file: `item.<code>.name` for the pieces and the
bench, `station.carpenter.name`, `craft.carpenter`, `build.tool.furniture`,
`build.refused.{in_wall,overlap,doorway}`, `build.remove.not_empty`,
`interact.act.{sit,lie,open_store}`, `furniture.*` for the panel and notes,
`town_mission.<id>.*`. Plain and short.

## Risks / Trade-offs

- **The towns' meshes change** (more faces inside houses), so a test that
  counted a town's faces moves; no saved thing does.
- **A town piece is set in a regular hexagon's plan**, while a planet cell
  is up to 9% off it; every piece keeps 0.1 m or more off its walls in the
  standard cell, which covers that.
- **Townsfolk step into their seat or bed** rather than walk to it (they are
  stood there when the hour turns, as the lost villager is stood): walking
  through a house's doors is for later. Recommendation taken.

## Migration Plan

New records and goods only. An old save loads unchanged, gains furnished town
houses on load, and has no `store` record until the player moves a stack.
