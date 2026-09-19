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

## Collision: walking into a cave

### What the two walkers already do

**Tenebris** (`tenebris-client/src/player_ctrl.rs`, `tenebris-core/src/world.rs`)
walks a tile-centred column stack with four primitives:

| primitive | answers |
| --- | --- |
| `walkable_floor_near(tile, r, body_h)` | the nearest floor at or below `r` with `body_h` of passable cells over it; a floor within step height ABOVE wins (the bridge tiebreak) |
| `ceiling_above(tile, head_r)` | the bottom of the first solid at or above the head |
| `try_horizontal_step` | the candidate tile's floor must be within `step_height_m` (0.6) of the feet AND have headroom, else the move is a wall and slides |
| the containment resolve | feet inside solid: lift to the top of that solid run, capped at step + body height |

Around those sit a stack of rescues - the tree-ground override, the straddle
probe, the neighbour-floor borrow, the topmost-solid fallback, four overshoot
gates - and Tenebris's own `CLAUDE.md` records the fall-through bug they were
written for as **still open**. The shape of that history is the lesson: a
walker that reads ONE tile under its feet flips tiles at every boundary, and
each flip is a new way to read the wrong column.

**Ours** (`walking.rs`, `planet_contact.rs`) reads a five-point footprint - the
centre and four points at `BODY_RADIUS` - and stands on the MAX floor of the
five, swept in 0.2 m segments from the last accepted position. A rise over
`step_height` (1.05 m) while grounded is a wall; tangential motion is dropped
and vertical motion kept, which is what lets a jump beside a wall reach full
height. That footprint is what keeps an edge stable: straddling a boundary
takes the higher of two answers rather than whichever one the tile lookup
landed on. Keep it. What it lacks is any notion of a ceiling, because
`SurfaceContact` has one radius per direction and cannot express one.

### Zero mouths, measured

`cave_mouths` (an ignored report in `planet_column.rs`) counts the tier's caves
whose air gap stands above a neighbour's cap, which is the only way a walker on
open ground can enter one without digging. On the shipped tier:

> 3,105 columns, 2,530 with a cave, **0 open to the surface**.

That is by construction: `hollow` damps the carve to nothing over the top
`roof_m` (9 m) of every column, so no gap ever reaches the ground. The rule was
written to stop the surface being lace, and it works; the cost is that every
cave is sealed. **Collision alone can therefore never let a player walk into a
cave** - the most it buys is falling into one through a sinkhole, and there are
none of those either.

So "walk into caves like Minecraft" is three pieces, not one, and the write-up
says which is which:

1. a walker that understands floor AND ceiling (this section);
2. a way in: either a **mouth rule** in the carve, or **digging** - and
   Minecraft has both, since its caves open onto hillsides constantly and a
   player digs the rest;
3. the mining change already planned, which is what makes an opening anywhere.

### The contact answer: a Stand, from one function

`SurfaceContact` stays what it is - the surface flight, rain and the spawn
read - and the walker gets a second query alongside it:

```rust
/// What a body at `position` stands between.
pub struct Stand {
    /// Top of the solid run at or below the feet: what you stand on.
    pub floor_radius: f32,
    /// Bottom of the solid run above the feet, if there is one.
    pub ceiling_radius: Option<f32>,
    pub water_depth: f32,
}
impl PlanetContact { pub fn stand(&self, position: Vec3) -> Stand }
```

Inside `PlanetContact`, because that is the only place that knows which tier
answered: `FineTier::locate` already yields the finest RECORD index, and that
index is what `ColumnTier::slots` is keyed by. `set_fine` already receives the
whole `FineSet`, which carries the tier. Outside the column tier - and in the
coarse tier, and off the finest band - `stand` is the heightfield's floor and
no ceiling, so nothing in the walker learns that two representations exist.
A footprint sample that lands off the tier answers with the surface, which is
above any cave floor, so a cave meeting the tier's rim is a wall to the walker
exactly as the solid rim is to the eye.

**The core's `Column::contact` answers the wrong thing today and this is the
first fix.** It returns the top of the first solid LAYER at or below the point.
A point 0.3 m inside a two-metre wall is therefore told its floor is one metre
up, which the step rule accepts, and the walker steps into the middle of the
wall. The floor is the top of the solid RUN, which is what
`walkable_floor_near` returns and why it insists on a passable cell above.
`contact` gains that rule and a test with a point inside a three-layer wall.

### The walker: three additions, no new path

- **A footprint takes the MIN ceiling as it takes the MAX floor.** Five
  `stand` samples instead of five `sample`s; the floor logic is untouched.
- **Headroom is a wall.** A candidate whose ceiling is under `floor + 2 x
  HALF_HEIGHT + CONTACT_SKIN` is rejected in the sweep exactly as a tall rise
  is: tangential motion dropped, vertical kept. This is Tenebris's headroom
  check in `try_horizontal_step`, and it is what stops a walker forcing their
  head into a low passage.
- **The head clamps to the ceiling.** After the sweep, if the head is above
  the ceiling, the position drops to `ceiling - height` and the upward velocity
  is zeroed, tangential kept. Tenebris does this after integration and skips it
  underwater, where it only zeroes the rise; ours does the same, since a
  swimmer against rock stops rather than teleports.

Not ported: the containment resolve and the rescue stack. The swept footprint
is why they are not needed, and adding them would be adding the failure mode
they were written against.

### A way in, so there is something to walk into

The carve's surface damping stays: lace was the right thing to prevent. What it
needs is an exception that is a feature rather than a leak. A **mouth** is a
place where the damping is lifted, chosen by the same seeded noise the carve
uses so it is deterministic and rare: where a low-frequency mouth field crosses
a threshold AND the surface slope is steep enough to be a hillside, `roof_m` is
taken to zero over that patch, and the tunnel underneath is allowed to break
the surface. Measured by `cave_mouths` before and after, with a target of a few
per tier rather than one per hill.

That is worldgen and gets its own write-up before its code. Digging needs no
new rule at all and is task 4; between them, the recommended order is:

1. `Column::contact` answers run tops. Core, one test.
2. `PlanetContact::stand`, column-aware inside the tier. Tests: a cave floor,
   a cave roof, and the band edge answering as before.
3. The walker's three additions. Tests: standing on a cave floor, a jump under
   a roof stopping at the roof, a 1.5 m gap being impassable, and the terrace
   scenarios unchanged.
4. Digging (task 4), which is the first way in and the first proof.
5. The mouth rule, measured, as its own change.

The owner's in-game check closes it, not a headless test: Tenebris's walker
passed every test it had while falling through the world at trees.

## Rendering the runs

### The tier is a sub-band, because a column costs 22.3 us to build

Measured (`column_cost`, an ignored report in `pbd_core::column`): one column is
**22.3 us** and carries **2.30 solid runs** on average. So the level-11 band's
40,670 cells would be **0.91 s** of generation and 12.4 MiB of layers, and that
whole cost lands on the async task that already rebuilds the fine set every 40 m
the player walks. It is survivable and it is not worth paying: what a column
buys is a cave a player can be INSIDE, and 300 m of that is 300 m of rock nobody
is standing in.

So the columns are a **sub-band inside level 11**: `COLUMN_M` metres of great
circle around the same anchor, about 3,700 cells at 90 m, **82 ms** to generate
and 393 KiB on the GPU. The same partition shape the LOD bands already use, one
tier further in.

At the sub-band's edge a cell has no column and the heightfield answer stands,
which is exactly what the tier below already does - and it leaves no hole,
because a cell whose neighbour has no column assumes that neighbour solid below
its cap, which is the heightfield's own assumption.

### The pass is ADDITIVE, so the surface pass is untouched

The terrain draw already puts a cap at the surface and a wall from it down to the
neighbour's cap, and it carries the LOD partition, the fine floors and the cut
wall. Rewriting that to be run-driven would risk every seam already paid for.

So the column pass draws only **what is below what the terrain pass draws**:

| face | where |
| --- | --- |
| a cave ceiling | the bottom of every run but the lowest |
| a cave floor | the top of every run but the highest |
| a run flank | from the neighbour's cap down to the run's bottom |

Nothing is coincident: the terrain wall spans from our cap down to the
neighbour's cap, and a column flank starts where that wall stops. The top run's
own top cap stays the terrain pass's.

### A flank must be clipped against EVERY air gap the neighbour has

This is the one thing that cannot be approximated away. A flank wall drawn down
the full height of its run would be correct wherever the neighbour is rock -
invisible, buried - and **would close the passage** wherever the neighbour is
air, which is precisely the case the whole change exists to serve: a tunnel is a
run of air crossing many cells, and a wall at every cell boundary is a tunnel
made of sealed rooms.

So a column record carries **its six neighbours' column slots**, and a flank is
drawn per (my run, neighbour's air gap) pair. A column of `MAX_RUNS` runs has
`MAX_RUNS + 1` stretches of air, so that is four runs against five gaps, twenty
quads a side.

**Not the largest gap only.** The first cut drew one quad a side over whichever
stretch of the neighbour's air was widest, on the reasoning that the rest is a
sliver and the exact answer costs 720 vertices. It is not a sliver: a neighbour
with two gaps had the second drawn as nothing, and from inside a cave nothing is
a window. 720 vertices is affordable on a tier of a few thousand cells and is
the exact answer rather than most of one.

### The tier's rim is generated SOLID

A cell outside the tier answers from the heightfield, whose one assumption is
that the ground under a cap is rock. That assumption is only safe while nobody
can BE under a cap, and a cave is exactly being under one: an off-tier cell
draws no face below its cap, so a cave reaching the boundary is a hole a player
looks out of.

A FACE cannot close it, and that was tried: a flank drawn on the rim's outward
side points away from everyone inside the tier, and the pipeline culls back
faces, so it draws nothing anybody can see. ROCK closes it, because rock is what
the assumption says is there. `column::generate_solid` is the same generator
with no carve; one cell thick is enough to occlude; and the ring moves out with
the tier every `REGEN_DISTANCE_M`, so a player walking toward it never arrives.

### The budget

Per column cell: `MAX_RUNS` x (18 ceiling + 18 floor) for the caps, plus 6 sides
x `MAX_RUNS` runs x `MAX_RUNS + 1` gaps x 6 = **864 vertices**, against the
terrain pass's 60 that it adds to. Over the tier's ~3,100 cells that is 2.7 M
vertices. A run a column does not have, and a gap a neighbour does not have,
collapse to degenerate vertices, which is what the tree branch already does past
its cutoff.

**Measured** on this container's software rasteriser (lavapipe, which is for
A/B and correctness and never for a frame rate): the surface preset is 549 ms a
frame with the tier and 424 ms with `reach_m: 0`, so the tier is about 30% of
that frame - and every vertex of it is invisible from above ground. A cheap
bound exists and is not built: a column whose air gaps all lie below every
neighbour's cap is SEALED and can only be seen from inside, so the visibility
pass could skip it whenever the camera is above the local surface. Computing
that flag is a tier-build-time question about data the build already has.

### The record

```wgsl
struct ColumnRec {
    runs: vec4<u32>,      // 4 runs: from | to << 9 | code << 18 | body << 22
    neighbors: vec4<u32>, // sides 0..3, 0xffffffff off the tier
    more: vec4<u32>,      // sides 4, 5, the degree, then spare
}
```

48 bytes. **Two materials per run**, not one: a surface run is a metre of turf
over tens of metres of rock, and a flank drawn in the material at its top is
forty metres of wall painted like a meadow, which is what the first capture from
inside a cave came back as.

The cell record carries its column slot plus one in the **high half of
`metadata.z`**, whose low half is the baked sky occlusion. An integer field
rather than the record's spare `f32`, which is where this started: a slot
written as `f32::from_bits` is a DENORMAL for every slot the tier can hold, and
a driver may flush a denormal to zero on load, which would read as no column at
all. It survives on lavapipe, which is exactly the kind of thing that survives
every test and fails on somebody's machine.

### Lighting is a stand-in, and says so

A cell's skylight was baked for its SURFACE, so a face thirty metres inside a
hill is handed the same sky as the hillside over it and a cave comes out lit
like a meadow. `cave_dark` and `cave_dark_depth_m` in `column.ron` darken a face
with burial. That is a stand-in for the baked voxel light this change defers,
named as one so it can be turned off rather than hunted for.

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
