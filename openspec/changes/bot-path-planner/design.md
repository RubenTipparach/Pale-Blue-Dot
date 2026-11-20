# Design

## 1. The search

`showcase::path::plan(from, goal, ground) -> Option<Plan>` (the bot's own
module; nothing in the game's rules uses it). `ground` answers, for a cell
key and a layer, the material there as the player would find it
(`digging::material_of` over the fine set and the save's edits), and a
cell's neighbours and centre (the finest set). The search is A* over states
`(cell, layer)`, the layer the feet stand in, with the straight distance to
the goal as the heuristic. The region is the cells within the leg's span
plus 18 m, the leg at most 60 m (as `HexRoute`).

A state is standable when the block under the feet (`layer - 1`) is solid
and not water-topped deeper than a wade on a dry walk.

Moves from `(a, la)` to a neighbour `b`:

| Move | To | Needs | Digs |
| --- | --- | --- | --- |
| walk | `(b, lb)` with `lb - la` in `-3..=1` | `(b, lb)` standable, `lb` and `lb+1` clear | none |
| tunnel | `(b, la)` | the block under it solid | solid blocks at `la`, `la+1` in `b` |
| stair up | `(b, la+1)` | the block under it solid | `la+1`, `la+2` in `b`, and `la+2` over `a` |
| stair down | `(b, la-1)` | the block under it solid | `la-1`, `la`, `la+1` in `b` |

Cost: metres walked, plus 0.5 m a step up, plus for each block dug a cost
by its material (soil, sand, grass, dirt 2 m; stone, rock, ore 4 m; others
3 m). Stations, lamps and water are never dug. Leaves and plants are not
blocks here and are walked through.

## 2. The plan

`Plan { steps: Vec<Step> }`, `Step { cell: u32, layer: usize, at: Vec3,
digs: Vec<(u32, usize, Material)> }`, the start state left out. The search
caps at 20 000 states expanded; a leg with no plan inside it falls back to
the nearest state reached (the bot replans from there).

## 3. Following it

The bot's walk (`Bot::walk_to`) takes the next step:

1. While the step has a block to dig that is still solid: the tool for it
   (`fetch_tool`, pickaxe for stone, rock and ore, else shovel), the look
   on the block's centre, the button held while the look meets it.
2. Then walk to the step's centre, jumping if the step is a stair up.
3. A step reached (the feet in its cell at its layer, give or take a half
   block) is dropped; a step not reached in 6 s, or a block not broken in
   8 s, replans from where the walker is.

`dig_through`, `Through` and `blocked_here` go. The fishing, field and
station acts keep calling `walk_to` as they do.

## 4. Checking it

- Unit tests on a made-up ground (a closure): a flat walk, a pit two deep
  (stair up out of it), a wall (tunnel), a three-block drop, a station in
  the way (round it), deep water on a dry walk (round it).
- In the game: the rest-of-tree run from the paused fresh-water shot on
  `tree-run-5` reaches the river and fishes it.

## 5. Building its way out (owner, 2026-11-20)

New moves from `(a, la)`, each needing a carried block (soil, dirt, grass,
stone, sand; the select path puts it in hand):

| Move | To | Needs | Places |
| --- | --- | --- | --- |
| pillar | `(a, la+1)` | `la+1`, `la+2` open over `a` | a block at `la` in `a`, jumped onto |
| step | `(b, lb)` with `lb - la` = 2 | `(b, lb)` standable | a block at `la` in `b`, then up on it and on |
| bridge | `(b, la)` | `(b, la-1)` open or water | a block at `la-1` in `b` |

Each place costs 2 m, like a dig of earth. A step's `places` list sits
beside its `digs`. The follower aims at the face the block goes on and
right-clicks, as `Place` does. The planner counts the blocks carried and
plans no more places than that.

Seen 2026-11-20 on `tree-run-6`: after the bait and fresh-water work, the
bot was boxed in underground among its own tunnels, a flooded pit and a
standing tree's ground. Digging alone had no way up.

### 5.1 Up and out first (owner, 2026-11-20)

The owner: "the easiest way to get unstuck underground is to build a ladder
up, or dig straight up and place dirt blocks below feet". So when the bot is
underground (open sky not over it) or no plan is found, it first goes up and
out:
- dig the block over the head;
- jump, and at the top of the jump place a carried block under the feet;
- again, until the head is in open air;
- or set a ladder against a wall and climb it, when one is carried.
Then the bot plans again from the surface. This comes before the pillar,
step and bridge moves of the plan above.
