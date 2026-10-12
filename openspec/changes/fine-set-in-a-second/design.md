# Design: the fine set in under a second, and the dig that does not wait

## Where the sixteen seconds go

`record` builds one `GpuCell`. Per cell it samples the height field at its own
centre, at each neighbour's centre (memoised, so mostly shared), and then for
EACH side calls `fine_floor`, which samples seventeen points along the edge.
Those points are not shared with the neighbour's own call for the same edge
(it walks the edge from the other end, so the floats differ and the memo
misses). About a hundred noise evaluations a cell, at roughly a microsecond
each, is the 0.1 ms per cell the instrument reports.

## Where the shader reads a floor

`floor_of` has one caller, `planet_surface.wgsl`'s wall branch:

```wgsl
if level < finest_level() {
    let mid = normalized(corner[side] + corner[side+1]);
    let neighbor = normalized(2.0*mid - axis);
    if covered_by_finer(neighbor, level) {
        lower_height = min(lower_height, floor_of(cell, side));
    }
}
```

`covered_by_finer` is `dot(neighbor, anchor) > band_cos(level + 1)`, and
`band_cos` is the cosine of the set's `complete` radius for that level
(`LodParams::of`). `planet_visibility.wgsl` declares `floor_of` and never calls
it. So a floor is read iff:

- the cell is on level 8, 9 or 10, AND
- the neighbour across that side is within `complete[k+1]` of the anchor.

A band's inner edge sits at `BAND_M[k+1] - margin` and `complete[k+1]` is at
most `BAND_M[k+1]`, so the sides that qualify are a ring one margin wide at the
inner edge of each coarse band: measured at the spawn by the test below, not
assumed.

### The rule the CPU applies

`record` takes a `FloorRule`: `None` on the finest level, `Every` on the base
(whose finer neighbour band moves with every anchor), and on levels 8 to 10
`Within { anchor, cos: cos((complete[k+1] + 2 * tile_width_m(level)) / R) }`.
A side computes `fine_floor` iff the rule reads its neighbour's direction;
otherwise it carries the neighbour's height.

The two-tile guard covers the one approximation: the shader reflects the
centre through the edge midpoint to find the neighbour, the CPU uses the
neighbour's real centre. They differ by a fraction of a tile. The carried
neighbour height is chosen so that even a read the guard missed would be
`min(corner.w, corner.w)`: the same wall a finest-level side draws.

Building level `k`'s records needs `complete[k+1]`, which is known once level
`k+1`'s band has been laid and (if over capacity) truncated. So the build lays
all four bands FIRST, settles `complete`, and only then records.

### The test that holds it

`a_floor_is_computed_wherever_the_shader_reads_one`: for the real set at the
spawn, for every record on levels 8 to 10 and every side, run the shader's own
test (reflected neighbour, `LodParams::of(set).bands`) and, where it would
read the floor, assert the record's floor equals `fine_floor` recomputed in
full. It also counts the sides, which is the measured size of the ring.

## Every core

After the floors, the build is lattice (about 130 ms a level) plus cheap
records. Both parallelise without touching determinism:

- **Lattices**: one `Lattice` per level, the four laid on four scoped threads.
  A `Lattice` is a memo of pure positions, so four of them lay the same cells
  one would.
- **Records**: every level's cells cut into chunks, one scoped thread per
  chunk up to `available_parallelism`, each with its own `Heights` memo, and
  the outputs concatenated in chunk order. Record `i` is the same bytes
  whichever thread built it.

`std::thread::scope` rather than a new dependency: the task already runs on
Bevy's compute pool and blocks for its whole duration anyway, and scoped
threads borrow the cells without an `Arc`. The column tier stays after the
records and single-threaded: it is 95 ms and its passes depend on each other.

A test holds the parallel build to a single-threaded build of the same anchor,
record for record, byte for byte.

## A dig never waits

With the rebuild under a second the tier keeps up with any walker, and this
part is what makes the claim unconditional rather than a matter of speed.

**The sampler.** `sample_at` today returns `None` where the finest record has
no column, and the march reads `None` as air: the ray passes into the hill and
the click does nothing. It answers instead from the record: solid iff the
layer's floor is below the record's height. That is exactly what the rim rule
generates for a column off the tier (`generate_edited_solid` fills to the
surface) and exactly what the heightfield draws there, so the ray stops on the
cap the player sees. A column with edits is not reachable this way: an edit
always lands on a resident column, and a rebuild regenerates it with them.

**The edit.** `apply_edit` finds the record and, where it has no column, calls
`ColumnTier::adopt(record, ...)`:

- takes the next slot (the buffers are sized to `COLUMN_CAPACITY`; a full tier
  refuses, logged as the error it is);
- generates the column with `generate_edited_solid` and the save's edits for
  that cell, 5 us measured;
- names its resident neighbours' slots, and writes its own slot into each of
  their side words, so their flanks clip against it;
- is flagged RIM, which it is by construction (it is outside the tier's disc),
  and stamps the finest record's slot word;
- extends the light array with its slot and lets the edit's whole-tier
  relight, which runs anyway, light it.

Then the edit proceeds on the resident column exactly as today. The next
rebuild regenerates the cell as an ordinary member of the tier, with its edit.

**What stays an error.** A click whose ray enters ground no finest record
covers (`OffTheFineSet`) is still logged: the finest level is not resident
there at all. With the rebuild under a second that state lasts under a second
after an arrival and never happens while walking, and the log line is the
instrument that says so if it ever does.

## Measured after

`streaming_cost`, release, the owner's desktop (i7-9700F, eight threads), the
default spawn:

| | Before | After, one thread | After, eight threads |
| --- | ---: | ---: | ---: |
| `generate_fine`, whole | 16,089 ms | 1,807 ms | **535 ms** |

The bands lay in 127 to 211 ms each (in parallel, so the slowest is what
counts), the tier is 85 ms, and one column alone is 5 us. The floor test
counts 41,988 of 665,063 coarse sides the shader reads a floor on (6.3%);
level 11's 48,631 cells carry none.

The base level cannot use the rule (the finer band it meets moves with every
anchor, so every side is read somewhere), so it is built on every core in
chunks instead. Startup (`Planet ready`, base plus the first fine set) went
from 31.16 s before this change, through 17.03 s with only the fine set fixed,
to **4.40 s**.

In the game, the same desktop, a scripted session of real input (synthetic
keyboard and mouse into the release build, logs and screenshots kept outside
the tree): forty seconds sprinting while clicking dig twice a second, a
flight, stepping out, digging on landing, walking on.

| | Before (`44d1d6d`) | After |
| --- | ---: | ---: |
| Fine set landing time | 16.6 s | 0.5 - 0.8 s |
| Digs taken | 19 | 149 |
| `edit BLOCKED` lines | 78 | **0** |
| Startup (`Planet ready`) | 31.2 s | 4.4 s |
| Stepping out of the ship to the set landing on the player | | 0.84 s |
| Stepping out of the ship to the first dig | | 1.0 s |

The baseline's 78 errors are every one `NoColumn`, starting six seconds into
the sprint at 91 m from the anchor, which is the tier's 90 m edge: the failure
the proposal predicted, reproduced on the owner's machine.

Adoption never fired in the session: at half a second a rebuild, the tier was
always there before the click. It is the guarantee rather than the mechanism,
and `a_dig_past_the_tiers_edge_adopts_the_column_and_lands` is what holds it.

## What is not settled

- Whether the record chunks should leave a core for the main thread. The
  session above showed no hitch worth a number, but it was not measured as
  frame time; `performance-harness` is where that belongs.
- Whether `near-field-streaming`'s growth is still wanted once the rebuild is
  fast. It still removes the whole-set upload every 40 m; that is a frame-time
  question, not an arrival one, and is left to that change.
