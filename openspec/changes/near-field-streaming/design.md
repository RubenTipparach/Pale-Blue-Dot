# Design: the near field is its own artifact, and it grows

## What the code does today

`lod::generate_fine(anchor, settings, edits)` is one function and one task. It
lays the four fine bands (levels 8 to 11, out to 2.4 km), records every cell,
and then calls `column::build`, which gathers the worms once for the 90 m
region, generates every column inside `reach_m`, stamps each finest record
with its slot, lowers the records a mouth broke through, and bakes the light.
`refresh_lod` spawns that task when the camera has moved 40 m from the set's
anchor, one in flight at a time, and swaps the whole `FineSet` into
`PlanetFine` (a new version) and into `PlanetContact` when it lands;
`upload_fine` then rewrites all four band regions, the column records, the
light and the materials.

Three consequences, and the owner reported two of them:

- **The tier waits for the bands.** They are one task. The tier is the part
  under the player's feet and the only part they can edit; the bands are the
  horizon.
- **The tier is rebuilt from nothing every 40 m**, including the 90 m of
  columns that were already there and have not changed. A column is a pure
  function of its direction, the worms and the edits (the `dig-and-place`
  design rests on that), so the rebuild computes the same answer again.
- **The player outruns it.** The tier reaches 90 m; the rebuild starts at
  40 m; a walker covers the remaining 50 m in six seconds at walking speed and
  under four at a sprint. Past the edge `sample_at` finds no slot and an edit
  does nothing; and the edge is the rim, a ring generated SOLID, so what is
  ahead is a wall until the set lands. If the rebuild takes longer than the
  next 40 m of walking, every landing is already stale and the next rebuild
  starts at once, anchored behind the player.

## Measured

`streaming_cost::what_the_near_field_costs_to_build`, ignored, release, on this
container (software rasteriser, slow cores; the owner's machine is faster and
the ratios are what matter):

| Part of one rebuild | Cells | Cost | Per cell |
| --- | ---: | ---: | ---: |
| Level 8 band (lattice 121 ms + records) | 35,931 | 3,487 ms | 0.094 ms |
| Level 9 band (lattice 111 ms + records) | 35,878 | 3,381 ms | 0.091 ms |
| Level 10 band (lattice 125 ms + records) | 39,035 | 4,035 ms | 0.100 ms |
| Level 11 band (lattice 130 ms + records) | 48,631 | 4,576 ms | 0.091 ms |
| Column tier: worm gather | | 16.0 ms | |
| Column tier: build (gather, columns, reconcile, relight) | 3,105 | 85.3 ms | 0.027 ms |
| `generate_fine`, whole | 159,475 + 3,105 | 15,935 ms | |
| One column generated alone, gather amortised (mean of 500) | 1 | 0.006 ms | |

The tier is **0.5%** of the rebuild. Ninety-seven percent of the rebuild is
`record`: the seventeen floor samples of the height field each band cell
takes, at about a tenth of a millisecond per cell across all four levels.
That cost is the bands' own and is what a ring-wise band rebuild would pay
only for its new ring; it is noted for `hexagon-lod` and not touched here.

Two things follow. The tier on its own is 85 ms for all 3,105 columns, so
even rebuilt whole it lands two hundred times sooner than it does today, and
grown by the ring it is a few dozen columns at six microseconds each: a
walker's whole frontier costs well under a millisecond a frame. And one
column at six microseconds is cheap enough to generate on the frame an edit
asks for it, which is what makes "an edit never waits" a rule rather than a
hope.

## The tier becomes its own artifact

`ColumnTier` today is a `Vec<Column>` in slot order, `slots: Vec<usize>`
indexed by FINEST RECORD INDEX, the packed `GpuColumn` records, and the light.
Two of those tie it to the band it was built in: `slots` is keyed by the band's
own record order, and each finest `GpuCell` carries its slot in `metadata[2]`.

The tier gets its own key and its own life:

- **Keyed by stable cell ID**, the `metadata[3]` the edits are already keyed
  by. `slots` becomes a map from ID to slot, and the record-index view the
  digging march uses today (`tier.slots.get(record)`) becomes a lookup through
  the record's ID. `record_of` already resolves an ID by scanning the finest
  records; the same scan runs the other way, once per band swap, to restamp
  every finest record's slot word.
- **A slot allocator with a free list.** A column that leaves the tier frees
  its slot; a column that joins takes the lowest free one. The GPU column
  buffer, the light and the materials are already fixed at `COLUMN_CAPACITY`
  slots, so nothing moves when a slot is reused, and an upload is the changed
  slots rather than the buffer.
- **The anchor is the player, continuously.** No 40 m rule for the tier. Each
  frame the tier knows which IDs are within `reach_m` of the camera (the
  finest band's records give the candidates; a lattice walk from the last
  centre gives them cheaply) and which resident columns are past
  `reach_m * UNLOAD` (1.5, Tenebris's `DETAIL_UNLOAD_FACTOR`, so a column does
  not thrash at the edge).

## It grows nearest first, under a budget, and every frame is complete

Missing columns are generated in order of distance from the player, on the
main thread, under a wall-clock budget per frame (`tier_budget_ms`, a
`ColumnSettings` field, default a few milliseconds), at least one a frame so
progress is guaranteed and the rest rolling over: this is `remesh_budget_allows`
from Tenebris's `renderer.rs`, the rule it uses to keep a burst of dirty chunks
from stalling a frame, ported for the same reason.

Every frame's tier is a COMPLETE generation. That is the standing rule against
drawing partial data, and the rim rule is what makes it cheap to honour: a
column whose neighbour is not resident is a rim column and is generated
solid, so the tier is closed wherever generation has reached and the rim is
simply the ring that has not been reached yet. When the ring behind it is
generated, the rim columns are regenerated carved (they keep their edits, as
the rim already does today) and the new ring becomes the rim. Nothing is ever
drawn with a hole in it.

Nearest first means the ring AHEAD of a walking player closes first: the
columns the player will reach next are the ones being generated now, and the
ones behind them, which nobody is looking at, come last. At walking speed the
front moves 8 m a second, which is under three rings of columns a second, a
few dozen columns, well inside a few milliseconds of budget at the measured
per-column cost.

The worm gather stays regional: gathered once for a region around the anchor,
re-gathered when the player has moved a fraction of its reach, on the pool.
That is the one thing on this path that is not per column, and it already
exists as the regional pre-pass `column::build` runs.

## Light, records and the surface, incrementally

- **Light** is a whole-tier bake at a measured 9 ms. It stays whole: rebaking
  the whole tier once per frame that added columns is the same cost an edit
  pays today, and cheaper than the reasoning an incremental bake would need.
  It goes up with the records that changed, as `upload_fine` already insists.
- **Records** upload per changed slot range. A generated ring is contiguous in
  slot order (the allocator hands out slots in generation order), so a frame's
  upload is one range for the ring plus the rim columns it regenerated.
- **The finest records** need two per-record writes when a column joins: the
  slot word and, where a mouth broke the surface, the lowered height and the
  neighbours' wall corners (`reconcile_surface`). Those are writes into the
  finest band's region of the cell buffer at known indices, the same kind of
  write an edit already makes, so the band's region gains a per-record dirty
  list beside the whole-band upload it has today.
- **The contact** already reads the tier through `FineTier` and `column(id)`;
  it keeps reading the live tier, which is now a resource that changes every
  frame rather than every 40 m, so `PlanetContact` holds the tier by `Arc`
  swap as it does the set.

## An edit never waits

When the digging march resolves a cell whose column is not resident, it does
not return `None`. It generates that one column synchronously with the
regional worms, adds it to the tier as a rim column of a one-column island
if need be, and proceeds. At the measured per-column cost this is invisible.
The cases it covers are the ones the budgeted growth cannot: the first frame
after a load or a teleport, and a sprint into fresh ground on a machine slower
than the budget assumes. It also makes the gate explicit: today an edit past
the tier's edge fails silently, which is a bug wearing a design's clothes.

## What stays

The bands keep `generate_fine` minus the tier, off the pool, every 40 m. They
are the horizon and the horizon can lag. Their own growth by ring, and the
band hysteresis `hexagon-lod` already names, are that change's work; this one
moves only what the player is standing on. `FOLIAGE_DRAW_CUTOFF_ALTITUDE` and
the altitude task on `hexagon-lod` are untouched.

## The instruments that came first

The owner asked for two tools before the change: the level under the player
on the screen, and a hard error whenever a click that should have changed the
world changed nothing. Both are built (`lod::NearField`, `hud::near_field`,
the `error!` lines in `digging.rs`) and both are how this change is judged:
when it lands, the readout should never say NO COLUMN on foot and the error
should never fire while walking. The owner's own definition of blocked is the
one the error implements: a click with no visual change and no collision
change. Flying is exempt only because edits are not offered off foot yet.

## What is not settled

- **The budget's default.** A few milliseconds is a guess until the owner's
  machine measures a ring; the field is data so it is tuned there, not here.
- **Whether the bands should follow.** The same ring growth would remove the
  40 m pop of band boundaries at 300, 600 and 1,200 m. Proposed on
  `hexagon-lod`, not here.
- **Teleport and load.** A teleport empties the tier; the first frames after
  one are a growing disc under the player. The edit rule covers the ground
  they point at; the walker's contact falls back to the heightfield where no
  column is resident, as it does at the tier's edge today.
