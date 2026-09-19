# Design: a column per finest cell

## The column

```rust
/// One cell's material stack over a fixed radial span, one entry a metre.
pub struct Column { layers: [Material; LAYERS] }
```

`LAYERS` covers the measured relief with room either side: the deepest basin is
-125 m and the highest summit +158 m, so a span of **-145 to +175** is 320
entries. A fixed span rather than a per-cell window, because a window needs an
anchor stored beside every column and a rule for what happens when an edit
leaves it; 320 bytes is cheap enough that the simpler thing wins.

**The bottom entry is bedrock and is never mineable.** Without it a player digs
through the floor of the world and sees the inside of the planet, which is the
one hole that cannot be fixed by drawing more faces. It is the reference's own
`col[0] = Bedrock` rule.

## Generation, from the one generator

A column is built from what already decides the surface, not from a second
description of it:

1. `planet_gen::surface_altitude` gives the top, exactly as it does today, so
   the heightfield tiers and the column tier cannot disagree about where the
   ground is.
2. Every layer below it takes the material the existing `top_material` rule and
   its depth give it: the surface material on top, soil under that, stone below.
3. **The carve.** A 3D field sampled at the layer's own position decides air.
   `gnoise3d_seed` already takes a point rather than a direction, which is what
   makes this possible without new noise: the terrain's own basis, sampled down
   the column instead of across the sphere.

A cave is worth having only if a player can be inside one, so the carve is
shaped rather than uniform: thresholded ridged noise gives connected tunnels
where plain fBm gives isolated bubbles, and the threshold tightens toward the
surface so the ground is not lace.

## Collision: from a radius to a run

`SurfaceContact` answers one radius today. It grows to the **run** structure
around a point:

```rust
pub struct ColumnContact {
    /// Top of the solid run at or below the sample: what you stand on.
    pub floor: f32,
    /// Bottom of the solid run above it: what you hit your head on.
    pub ceiling: Option<f32>,
}
```

That is the whole of walking into a cave. The walker already resolves against a
floor radius; it gains a ceiling, and the swept resolution that currently
rejects a step up taller than a stride keeps working because a cave mouth is a
step down into a run, not a cliff.

**The heightfield path does not disappear.** Outside the finest tier there is no
column, and `SurfaceContact` answers as it does now. One function decides which,
so nothing else in the walker learns that two worlds exist.

## Rendering the runs

The vertex budget is where a column tier is won or lost. Today a cell draws a
cap and six walls in 60 vertices. A column draws that **per solid run**.

Most columns are one run: solid from bedrock to the surface. A cave adds a
second. So the budget is a cap on runs rather than on layers - **four runs, 240
vertices** - and a column with more runs than that draws its four largest, which
is a bounded and visible failure rather than a buffer overrun.

240 vertices over the finest band's 40,670 cells is about **9.8 M vertices**
against the frame's present 19.6 M, and only in the band: the coarse tiers are
untouched. That is the same shape the clutter change took, and the same reason
it was affordable.

The exposed faces themselves are the mesher's ordinary question - a face wherever
a solid layer meets air or a neighbour's air - and the existing wall construction
already knows how to build one between two radii.

## Mining and placing

- A hit is a ray from the eye down the view, marched layer by layer through the
  columns it crosses, stopping at the first solid one. The clutter change
  already marches a ray for its shot; this is the same walk at a different
  scale.
- Breaking sets that layer to air and **gives the block to the slots**, which is
  what the ten slots are for and what removes the starting kit's reason to
  exist.
- Placing takes from the selected slot and sets the air layer the ray last
  crossed, refusing where the player stands.
- **Every edit is written to durable storage on the same input frame.** Not
  batched, not on a timer, not on exit. That is the repo's own rule, stated as
  non-negotiable, and a dug cave that does not survive a reload is the failure
  it exists to prevent.
- An edited column is re-uploaded as one record; nothing else re-generates.

## What this leaves for later, and says so

- **Baked light.** A cave lit by sky occlusion computed for a surface will be
  wrong inside. It wants the light pass the standing design describes, and until
  then a cave is lit as its mouth is.
- **Streaming beyond the band.** Walk far enough and a column leaves the finest
  tier; its edits must outlive that, which is a store keyed by cell rather than
  by record slot. The edit store is designed for that from the start even though
  the band is all that draws.
