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
by its material (soil, sand, grass, dirt 6 m; stone, rock, ore 10 m;
others 7.5 m: at 2 and 4 the bot tunnelled after a hen, 2026-11-20). Stations, lamps and water are never dug. Leaves and plants are not
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

## 6. A block between the walker and the planned dig (2026-11-22)

Seen on `tree-run-6`, shot 13 (fresh water), 14 m from the river: the plan's
first dig was 4.4 m off, past the 3.5 m reach, so the follower walked toward
it, but a stone block stood between and the walk went nowhere. Each 6 s the
step went stale and the same plan was made again, for over 20 minutes; not
one dig was tried. The watchdog's up-and-out found open sky and handed back.

Fix: while walking toward an out-of-reach dig, held short (no headway for
1.5 s), the bot looks level along the way and digs the block the look meets,
with that block's tool, as a player does. Not a dig the plan made, so the
plan's cell is avoided and planned round when that block will not come out
in 12 s (a tree's ground, a station). The step's clock is reset while it
digs, so the plan does not go stale under it.

## 7. Up and out from a cell's rim (2026-11-22)

Seen on `tree-run-6`, the last branch: in a pit, the bot aimed a block under
its feet every half second for four minutes, each refused ("is where the
player stands"): it stood on its cell's rim, and `place-underfoot` lifts the
player only over their own cell. Fix: before the block goes down, the bot
walks to within 0.45 m of its cell's middle; where it cannot get there
(seen next: four more minutes standing still), it pillars up as a player does,
looking straight down, jumping, and setting the block on the ground it left
at the top of the jump, which `place-underfoot` does not need to allow.
Measured next: over the middle, aimed at the middle of its own cell's top,
the look still met a neighbour's top from the eye's height and angle, and the
place was refused; the bot never jumped. So the middle is dropped: the bot
always pillars, looking straight down, which meets its own column's top.
Measured next: the height did not move in four minutes of jumps, and with W
held and a new heading every six seconds, not a millimetre: the walker is held
fast, its feet on cells' rims over a gap, where no jump starts. So five
seconds in every twelve it digs the block its downward look meets, with that
block's tool, and drops onto footing it can jump from; then pillars again.

## 8. Fields are not dug through (2026-11-22)

The first chili sown on `tree-run-6` was uprooted three seconds later: the
walk to the next cell planned a dig through the field's tilled soil, and a
dig under a crop uproots it. Tilled soil is now `Never` to the planner, a
floor it walks on and never digs.

## 9. Long walks over the surface (2026-11-22)

On `tree-run-6`, the walk to a desert 190 m off stayed under ground for a
quarter of an hour: each 60 m leg of the planner dug through where the way
round ran longer than the leg, and the walker never came up. A walk of more
than 40 m now follows the surface route (`HexRoute`, the older heuristic)
while the walker is not below the ground's surface; the planner takes the
last 40 m, a walker under ground, and 20 s after the surface route is held
(no headway for 4 s).

Measured next (design 7): in a tunnel of its own, four climbs and the height
fixed to the centimetre: the jump was a tap of one frame. It is now Space held
for a fifth of a second.

## 10. Out by a stair, with the right tool (owner, 2026-11-22)

The owner: "the only reason youd be stuck is if you used the wrong tool".
The pinned bot was aiming at stone with nothing in hand. Up and out no longer
sets blocks under the feet or jumps: it digs a stair. The block over the head
first (as before), then the next cell along the heading, its two layers over
the feet dug open with the block's own tool (pickaxe for stone, rock and ore,
shovel for earth), highest first, then walked into; the walker's auto step
takes the block up. Repeated until open sky is over the head. A neighbour is
used only when its floor is solid and its two layers are open or diggable
(never a station, water or a tree's ground).

Then the owner got the bot's walker out by hand, in the game, from the spot
the bot was pinned in (the log: the iron pickaxe in hand; the column over the
head dug to the grass, layers 215 to 221; then stone set under the feet once
a second, each place "ok"). The bot's two mistakes against that: it dug only
the one block over the head, and it looked down at 83 degrees, which from a
cell's rim meets the neighbour's top (refused). So up and out is now the
owner's way: the whole column over the head dug to the sky, lowest first,
each block with its own tool; then the look at 88 degrees, straight down, and
a carried block set under the feet every 0.6 s, the game lifting the walker.
The stair above is kept for when no block is carried, or after 20 s of
pillaring without getting out.

## 11. A leg done as soon as it is planned (2026-11-22)

Sowing the turnips, the walk to a field cell planned the same one-step leg
every frame for five minutes: the step was done on planning (the walker
already within 0.6 m of it) and the leg did not arrive (the goal a tilled
cell, `Never` since design 8). A leg whose steps run out within a second of
its making, without arriving, now hands the walk to the straight walk, as
when no plan is found.
Measured next, in a shaft by the turnip field: "No room above", 400 places
refused; the block over the head was solid and never dug. The column over
the head is now dug from the head's own layer up, not one over it.

## 12. Out of a pit on a walk; mining still digs down (owner, 2026-11-22)

The owner, on "never dig below the feet": "dig below the feet is effective
when getting ore". Mining keeps its stair down. What changes is the walk: in
a pit of the bot's own digging, open to the sky but deeper than a step (the
feet more than 1.5 m under the ground's surface), held for 6 s, the bot
pillars up to the surface as the owner did (design 10), then walks on.
Seen: five minutes in a pit by the turnip field, facing a two-block wall, the
same plan made every six seconds.
Then in the turnip field's pit: "No room above", the feet's column clear. The
game asks the head room of the cell under the eye, a neighbour of the feet's
from a cell's rim. That cell's two layers over the feet's ground are dug too.
Measured next (the refusal now logs what is over the cell): Air, then
Tilled, a field's edge over the bot in a hollow under it. The guard of 5.5
(never dig tilled soil) held it there. Tilled soil at or under the feet is
still never dug; over the head it is dug on the way up.
Then, in the field's pit with a full pack (no block to set): "no cell round
to stair up into", the cells round being the field's tilled soil a layer up,
`Never` to the stair's test. The stair digs tilled soil over the feet too.

## 13. The planner held the frame (measured 2026-11-22)

The owner: "game is lagging", "fps is super laggy" (F3 at 4 to 23 fps during
bot runs).

**Measured** (`tree14`, the reef shot on `tree-run-6`, the desktop, `--profile
fast`, F3 on, `--frame-log`):
- the first 5 minutes ran at 31 fps, frame p50 20.6 ms, p99 744 ms, worst
  2371 ms, with 286 frames over 50 ms;
- the main thread's p99 was 529 ms, the GPU's 34 ms;
- `PLAN_TIME` reached 1414 ms and 1853 ms for a single `path::plan`, run on
  the main thread inside the bot's frame;
- 197 plans or routes took over 8 ms, the routes (`HexRoute::plan_where`)
  50 to 100 ms each.

A plan may expand 20,000 states, at about 90 us each (every state reads the
ground's blocks, and the route's `avoid` asks `under_standing_tree`). On a
walk over rough ground near water, most searches run to the cap.

**Fix:** a wall-clock budget of 15 ms on the search. Past it, the search stops
and returns the nearest state it reached, as it already does at the
expansion cap (`arrives: false`, a leg planned on from its end). The bot walks
the part it has and plans on. The cap stays as a bound that does not depend
on the machine. Measured again on the same shot after the change.

## 14. The searches off the main thread (owner, 2026-11-23)

The owner, watching the bot at 13 to 17 fps: "why cant bot be on another
thread".

**Measured** (`tree-run-7`, the farming and fishing shots, the desktop, `--profile
fast`, F3 on, `--frame-log`):
- the 15 ms budget of section 13 was checked every 64 expansions; by the cove an
  expansion cost about 1 ms, so a plan took 66 ms at the median and 119 ms at
  worst (checked every 8 now);
- a failed surface route cleared the route and its half-second wait, and the bot
  routed every frame: 298 routes in 65 s at about 32 ms each, 222 of 978 frames
  over 50 ms (a failed route now waits a second);
- the farming shot, 300 s, four runs: the run whose bot planned 264 times and
  dug 40 blocks had 450 frames over 50 ms and the main thread at 22.7 ms; the
  runs whose bot hardly moved had 25 to 51 and 14 to 16 ms. Sound on or off made
  no difference once the bot's activity was matched (25 against 32);
- the same world with no bot, stood in for 240 s, holds the main thread at 10.4
  to 10.7 ms.

So the bot's searches are what holds the frame in a bot run, not the game.

**Design.** The two searches (`path::plan` and `HexRoute::plan_where`) run on a
worker thread from a snapshot:
- the bot copies what the search reads into an owned `GroundSnapshot` when it
  asks: the finest records within the leg's reach plus margin (their keys,
  directions, heights and neighbours), the edits for those cells, the
  stations, the bot's holes and blocks, and the tree test's answers for the
  cells in reach;
- one search in flight at a time; the bot keeps walking its current leg (or
  stands) until the answer comes back, and a stale answer (the goal moved more
  than 1.5 m, or the fine set's version changed under a plan that digs) is
  dropped and asked again;
- the inputs the bot presses each frame (`walk_to`, `aim_at`, the taps) stay on
  the main thread: they are cheap and must see this frame's state;
- no budget is needed on the thread for the frame's sake; the 15 ms budget
  stays as a bound on how stale an answer may be.

Measured before and after on the same shot, 300 s, the bot's plans counted per
run, so a quiet bot is never read as a faster game.

**Found on the way: a search of every record per lookup, in the game.**
`digging::record_of`, under `material_of` and the standing-tree test, found a
cell's record by walking all ~3900 finest records and comparing keys, on every
call; `fruit`, `tree_fx`, `tree_shake` and `beaver` did the same. The game calls
`material_of` on every aim and dig, and a bot's plan calls the tree test
thousands of times. The fine set now carries a key-to-record index, built with
the set on its own task (`FineSet::finest_index_of`), and every one of those
lookups uses it. A player sees nothing different; it is only less work.
Measured on the farming shot, the same 5 minutes before and after.
