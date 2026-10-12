# Proposal: the ground around the player loads first, and never stops loading

## Why

The owner reports blocks popping in around the player, and that nothing can be
edited "until it finally loads". Both are one mechanism, and it is the shape of
the streaming rather than its speed.

Everything fine around the player is ONE artifact built by ONE task. The four
LOD bands (about 160,000 records out to 2.4 km) and the column tier (about
3,100 columns within 90 m, the only cells that hold voxels and therefore the
only cells that can be dug or built on) are generated together in
`lod::generate_fine`, from scratch, on one compute-pool task, whenever the
player has walked `REGEN_DISTANCE_M` (40 m) from the set's anchor, and the
whole set is swapped in on the frame the task lands.

So the near field waits for the far field. Measured, the tier is 85 ms of a
16 s rebuild on this container, half of one percent, and it cannot arrive
before the other 99.5% does.
While the task runs the player keeps walking, and the tier they are walking
through is the OLD one, anchored 40 m or more behind them: at eight metres a
second a player reaches the old tier's edge fifty metres on, which is six
seconds after the rebuild started. Past that edge there is no column, so a dig
or a place finds nothing and silently does nothing (`sample_at` returns `None`
where `tier.slots` has no entry) - which is "can't edit until it loads". And
the edge is a ring of SOLID rock by the rim rule, so what a fast walker sees
ahead is a wall that vanishes when the set lands - which is "blocks popping
in". If the task takes longer than the player takes to walk the next 40 m, the
next rebuild starts the moment this one lands, always anchored behind the
player, and the tier never catches up until they stop.

Tenebris does not have this problem because it does not stream: the main body
is 300 m in radius and every column of it is resident, so an edit anywhere is
an edit on a column that exists. Its lesson for us is the shape of its remesh
loop instead: dirty chunks are rebuilt under a WALL-CLOCK budget, at least one
a frame, the rest rolling over, so no burst of work can stall a frame and
progress is always made.

## What

1. **Split the artifact.** The column tier stops being built inside the band
   rebuild. It becomes its own thing with its own anchor, its own cadence and
   its own upload, so the ground under the player's feet never waits on the
   horizon.
2. **Grow the tier instead of rebuilding it.** A column is already a pure
   function of its direction, the worms and the player's edits; the design
   says so and the rim rule depends on it. So the tier keeps every column it
   already has, generates the ring of columns the player has walked toward,
   drops the ring they have walked away from, and never generates a column
   twice. The rim rule needs no change: the rim is wherever the tier ends this
   frame, and it moves out one ring at a time.
3. **Nearest first, under a budget, every frame.** Missing columns are
   generated in order of distance from the player, a few milliseconds' worth a
   frame on the main thread (the reference's remesh budget, ported), and each
   frame's result is a COMPLETE tier with its rim wherever generation has
   reached, so nothing partial is ever drawn. The ring ahead of a walking
   player is a few dozen columns and is closed long before they reach it.
4. **An edit never waits.** The aimed cell is inside the tier by construction
   once the tier keeps up; and where it is not (a load, a teleport, a sprint
   into fresh ground on a slow machine) the edit generates that one column
   synchronously and adds it, because one column costs a fraction of a
   millisecond. There is no state in which pointing at the ground does nothing.
5. **The bands keep their task**, rebuilt off the pool every 40 m as now, and
   gain the same growth later: this change moves the near field only, because
   the near field is what the owner is standing on.

## Measured before anything moves

The write-up needs three numbers this repository has never recorded: what one
`generate_fine` costs split by level and by the tier, what ONE column costs, and
what the worm gather (the tier's regional pre-pass) costs on its own. An
ignored test in `planet_lod.rs` measures them off the real generator at the
default spawn and prints them; the design carries the results. They are this
container's numbers, which are slower than the owner's machine, and the
argument rests on their ratios rather than their sizes.
