# Design: dig and place

## Where an edit lives, and why not in the column

A `Column` is generated, not stored: `planet_column::build` makes one per cell
within `reach_m` of the tier anchor, and the tier is rebuilt whenever the
player walks far enough for the anchor to move. An edit written into a
generated column would live exactly as long as the player stayed put.

So an edit is a sparse OVERRIDE, keyed by the cell's stable ID (`metadata[3]`,
which the record has carried since the LOD set was built) and holding only the
layers that differ:

```rust
pub struct Edits { cells: HashMap<u32, Vec<(u16, Material)>> }
```

A `Vec` of pairs rather than a map of maps, because an edited column has a
handful of changed layers and the order they are applied in must be stable:
a `HashMap`'s iteration order is not, and `CLAUDE.md` is explicit that
randomized order must never decide a simulation outcome. The outer map is a
keyed lookup and nothing iterates it.

`column::generate` takes `&Edits` and applies them last, after the worms have
carved. That keeps the property the tier rests on: a column is a pure function
of its direction, the worm field and the edits. Generate it twice, from
anywhere, and it is the same column.

## Picking the target: Tenebris's march, against our lattice

`interact.rs` marches the eye ray in 0.25 m steps, resolves each sample to a
(tile, depth) with a warm hint, and stops at the first solid block; the last
non-solid sample is where a placed block goes. Ported:

- the hint is `Tier::walk(start, direction)`, which already steps cell to cell
  across whichever edge the ray falls outside of, so each sample costs a short
  walk from the last one rather than a search;
- the layer is `column::layer_at(altitude)`, the inverse of `layer_altitude`;
- solid is `Column::solid(layer)`.

A fixed step rather than a DDA because the lattice is not a grid: a hex column
has five or six neighbours and no axis to step along. At 0.25 m against a
1 m layer and a 2.833 m tile, a step cannot cross a whole cell, which is the
only property the march needs.

Reach is `DIG_REACH_M`, five metres, which is Tenebris's `INTERACT_MAX_DIST`
rounded to our scale.

## What a dig and a place may not do

- **Bedrock stays.** Layer zero is the floor the whole column stands on and
  `generate_solid` writes it unconditionally; taking it would open a hole to
  the centre of the planet.
- **A place may not seal the player in.** The candidate cell is rejected when
  it is one the player's own capsule occupies, tested against the same feet
  and eye the walker uses. Tenebris does this with its own body check, and the
  reason is the same: a block placed into your own head is a world you cannot
  move in.
- **A place needs a block.** `Slots::take_held` answers with the selected
  stack's item or nothing, and `Item::Tool` is not a block.

## What has to be rebuilt, and what does not

An edit changes one column, and what reads a column is:

1. **its own runs** - regenerate that column and repack its record;
2. **its neighbours' flanks** - a flank is clipped against this column's air
   gaps, so each neighbour's record is repacked too, though its own column is
   unchanged;
3. **the terrain record**, when the edit moved the column's top: that is the
   mouth rule already written, and it runs again for the edited cell.

Nothing else in the tier is touched. The GPU sees a handful of 48-byte records
rewritten at their own offsets rather than a rebuild of three thousand
columns, which at the measured 14.7 us a column would be a 46 ms hitch on
every click.

`PlanetContact` holds an `Arc<FineSet>` for collision, so it is refreshed from
the same edited set: a shelf you dig out is a shelf you can then stand in on
the very next frame.

## Durable on the frame, as the rule demands

`CLAUDE.md`: "Every accepted world mutation enters the durable transaction
path immediately. A queued write alone is not a durable save; acknowledge
commitment only after the storage backend succeeds."

So an edit is appended to `saves/<world>/edits.ron` and the file is flushed
before the system returns, and the accept is reported only if the write
succeeded. It is an append rather than a rewrite because the cost has to be
bounded by the edit rather than by the size of the world, and it is replayed
at startup into the `Edits` resource. The format is one line per edit - cell
ID, layer, material - so a partial write at the end of a file costs the last
edit rather than the world.
