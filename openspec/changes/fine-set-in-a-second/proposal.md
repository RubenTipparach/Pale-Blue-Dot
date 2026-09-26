# Proposal: the finest ground is there when you arrive, and a dig never waits

## Why

The owner asks for two things, in their words: "the highest detailed level of
terrain loads when I get there and I can interact with it immediately", and
"there should never be that blocking error when I try to mine the terrain".

Both are one number, measured on the owner's own desktop (Intel i7-9700F,
eight cores) with the existing instrument
(`streaming_cost::what_the_near_field_costs_to_build`, release):

| Part of one fine-set rebuild | Cells | Cost |
| --- | ---: | ---: |
| Level 8 band (lattice + records) | 35,931 | 4,084 ms |
| Level 9 band | 35,878 | 3,782 ms |
| Level 10 band | 39,035 | 4,447 ms |
| Level 11 band | 48,631 | 5,031 ms |
| Column tier (worms, columns, reconcile, light) | 3,105 | 95 ms |
| `generate_fine`, whole | | **16,089 ms** |

The desktop is no faster than the container the `near-field-streaming` design
measured on, because the whole rebuild runs on ONE thread; seven of the eight
cores are idle while the player waits. So:

- **Arriving anywhere** (landing, a load, stepping out of the ship) shows the
  base level and has no column to dig for sixteen seconds.
- **Walking**, the rebuild starts 40 m from the anchor and the tier ends at
  90 m, so a walker at 8 m/s reaches the tier's edge six seconds into a
  sixteen-second rebuild. From there to the landing every click into the
  ground logs `edit BLOCKED: ... NoColumn` - the error the owner reports.
- **Startup** builds the same set synchronously in `create_planet`, so the
  game also opens sixteen seconds later than it needs to.

Ninety-seven percent of the rebuild is `record`, and almost all of `record` is
`fine_floor`: seventeen noise samples on every side of every cell. The shader
reads a side's fine floor in exactly one place (`planet_surface.wgsl`, the
wall branch): on a level COARSER than the finest, and only where the neighbour
across that side lies inside the next finer band. So level 11 computes
48,631 x ~6 x 17 samples nobody reads, and levels 8 to 10 compute them for the
whole band when only the ring against the next finer band is ever read.

## What

1. **Compute a fine floor only where the shader reads one.** Level 11 carries
   none. A coarser side carries one where its neighbour lies within two tiles
   of the next finer level's complete radius; every other side carries the
   neighbour's own height, which the shader never reads there and which is a
   no-op under its `min` if it ever did. The drawn picture is unchanged by
   construction, and a test holds every side the shader would read to the
   full seventeen-sample floor.
2. **Build the set on every core.** The four levels' lattices are laid in
   parallel, and the records are cut into chunks generated in parallel, each
   with its own height memo, and joined in order. The output is bit-identical
   whatever the thread count: a height is a pure function of its direction.
3. **A dig never waits for the streaming.** Where the eye ray enters ground
   whose finest record has no column, the sampler answers from the record's
   own surface (solid below its cap, the assumption the rim rule already
   makes), and the edit generates that one column on the spot (5 us, measured),
   adopts it into the tier as a solid rim column carrying the save's edits,
   and applies the click to it. The `NoColumn` error becomes unreachable on a
   cell the fine set holds.

Together these take the rebuild from sixteen seconds to (the design sets the
target, the implementation measures it) well under one, which makes arrival,
walking and startup all one-second events, and makes the dig independent of
the streaming entirely wherever the finest level exists.

## What this is not

- It does not change the bands' radii, the partition, the tier's reach, the
  40 m regeneration rule, or any shader.
- It does not do the ring-by-ring tier growth `near-field-streaming` designs.
  That change remains the plan for a tier that never rebuilds; this one makes
  the rebuild cheap enough that the growth is no longer what stands between
  the owner and a working dig, and takes that change's section 4 ("an edit
  never waits") as its own item 3.
