# Proposal: light that knows what is above it

## Why

**The owner's words: "tenbris-rs had an advanced lighting model for ground
contact between blocks stacked on top of each other and caves, we need to
implement that here", and then: "lighting was calculated via voxel lighting
algorithm combined with baked vertex colors".**

Measured, with `tools/frame_mean.py`:

| | mean of 255 |
| --- | ---: |
| Standing in the open meadow at midday | 92.5 |
| Twenty metres inside a cave | **74.2** |

A tunnel underground reads at four fifths of an open field. That is the whole
report: there is no dark in this light model, because there is no light model -
there is a sun term and one scalar per cell.

**This repository already wrote the diagnosis down.**
`docs/tenebris-comparison.md` says it in as many words: light here "is sampled
per **cell**, flat across a 19 m tile" against Tenebris's per **vertex**,
interpolated; block and torch light is "**absent**"; and it names the cost -
"the single biggest reason the ground reads as faceted plates instead of
terrain". What is running is `planet_lod.rs`'s `skylight`, which is
`1 - occlusion/degree * 0.55` where `occlusion` is the mean uphill slope to
each neighbour. It is computed off the HEIGHTFIELD, so:

- It cannot see a cave, because a heightfield has no inside.
- It cannot see an overhang, for the same reason.
- It cannot see one block sitting on another, because the column tier - the
  voxel part, the part with the blocks in it - contributes nothing to light at
  all.
- It is `@interpolate(flat)`, so a 2.8 m cell is one brightness with a step at
  every edge.

**And half of the answer is already in the tree, unbound.**
`assets/shaders/voxel_light.wgsl` is a GPU Jacobi form of Tenebris's
`world_light.rs` seed-and-propagate rule, with RGB and sky nibbles, written
during the shader port and never dispatched. `docs/shader-port.md` says
plainly: "The desktop prototype currently uses the separate CPU scalar
sky-occlusion value described above."

## What

**A voxel light field over the column tier, and a per-vertex sample of it.**

- **The field.** One byte per (column, layer): a sky nibble and a block nibble,
  which is Tenebris's own `(sky << 4) | block`. Seeded at the top of each
  column's open air and propagated down and sideways, losing a level per metre
  and stopping at solid rock.
- **Contact darkening.** A face's corner samples the cells that MEET at that
  corner, so where a block sits on another the crease goes dark by
  construction rather than by a term added to it. This is the baked vertex
  colour half of what the owner named: the value varies across a face because
  its corners were sampled separately, which is also what ends the flat-tile
  faceting the comparison document blames for the ground reading as plates.
- **A cave is dark because nothing reached it**, not because a depth term was
  subtracted. The mouth stays bright, the light falls off over a few metres of
  tunnel, and a torch - when there is one - would fill the same field from the
  other channel.

## Scope

**The column tier only**, which is where blocks and caves are. Outside it the
heightfield's `skylight` stands, because a heightfield cell has no inside to
light and nothing to stack. The two meet at the tier edge, where a column's
open-sky top is level 15 and the heightfield value is already near one.

**No torches or lamps ship in this change.** The block channel is carried,
propagated and consumed, and nothing emits into it yet, because a torch is an
item and there are no items to place. What that buys now is that the channel
exists and is proven by a test rather than added later to a field that has to
be re-laid out for it.

## What this is NOT

**Not the GPU relaxation.** `voxel_light.wgsl` solves the same rule by Jacobi
iteration on the GPU, which is the right answer for a field that changes every
frame. Ours changes when the tier is rebuilt or a block moves, the tier is
already built on a background task, and a BFS is exact where a fixed iteration
count is approximate. The CPU bake goes in the same task as the columns; the
shader stays where it is, and the design says what would make it worth binding.

**Not per-frame relighting of the whole tier.** A dig relights the cells a
change can reach - a bounded region around the edit - and nothing else. The
whole tier is baked once when it is built.

**Not coloured light.** Tenebris carries a torch TINT and the unbound port
extends that to RGB nibbles. One block channel is what a white lamp needs and
is what the field carries; RGB is a widening of the same nibbles and is named
in the design as what it would cost.
