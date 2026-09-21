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

### A way in: the damping IS the knob, measured

The owner asked why the caves could not simply be raised. A height offset on
its own does nothing - the tunnels move up into the top nine metres and the
damping there erases them exactly as before - but the question was the right
one, because the damping was a GUESS: it was written to stop the surface being
lace, and lace was never measured. `damping_sweep` measures it:

| `roof_m` | land columns open at the top | underground hollow | columns losing their surface layer |
| ---: | ---: | ---: | ---: |
| 9.0 | 0.0% | 3.4% | 0.0% |
| 4.0 | 0.0% | 3.5% | 0.0% |
| **2.0** | **3.7%** | 3.5% | **0.2%** |
| 0.5 | 4.7% | 3.5% | 3.8% |

Two metres opens one land column in twenty-seven with nothing worth calling
lace; half a metre is where the ground starts going. So `roof_m` ships at 2.0,
and most of what that opens is a HOLE - the tunnel sheet crossing the ground -
that a walker drops into and follows down, which is how most Minecraft caves
begin. On the default spawn's tier that is about ninety columns.

The mouth patch below stays as a second knob on the same function: it is what
makes a few WALK-IN openings, a tunnel entering a hillside at ground level,
which the sweep's holes mostly are not (one in the default tier by the
stricter `cave_mouths` measure, seven in a patch's).

### Perlin worms: tubes, and openings where a worm starts at the surface

The owner's call, and the right one: the sheet carve is replaced by WORMS.

**A worm** is a seed, a start point and a walk. From its seed it takes a
length, a starting radius and a heading; each step it moves `step_m` along
its heading, turns its yaw and pitch by 3D noise sampled at where it is, and
its radius wanders by the same noise, so a slow bend widens into a room. Pitch
is held inside a band about the tangent plane so a worm tunnels rather than
dives, it is held under the ground by at least its own radius plus a layer,
and it never reaches the bedrock floor. What it carves is a chain of capsules.

**Some worms start at the surface.** A share of them (`surface_share`) begin a
layer under the ground heading down, and the first capsule of such a worm cuts
the ground open: that is the opening, and it leads somewhere by construction,
because the rest of the worm is behind it. The mouth patch, the flare and the
damping sweep all go with the sheet; a worm that starts at the surface is what
they were approximating.

**A column stays a pure function of its direction, which is the property the
whole tier rests on.** Worms are seeded on a fixed lattice - a cube-sphere grid
whose cells are about a base tile across - and a cell's worms are a function of
the cell's index and the world seed alone. To generate a column, gather every
worm whose seed cell lies within the longest worm plus the widest radius of
the column's direction, walk each (deterministic, so the same worm every time
from anywhere), and test the column's layers against the capsules that pass
near it. The tier build gathers and walks the region's worms ONCE and hands the
paths to every column, which is the regional pre-pass the write-up said worms
would need, done at the one place columns are built in bulk.

**What replaced what.** `CaveField` became `WormField`: density per seed
cell, length and radius ranges, step, turn rate, pitch band, surface share,
start depth, steer scale, all in `column.ron` with units. `hollow`, the ridged
salt, the mouth patch, the flare and the damping knob are gone; `generate`
takes the gathered worms. The instruments stayed and were the acceptance.

**Measured, at the default spawn, release build, one core of this container:**

| Quantity | Sheet carve | Worms |
| --- | ---: | ---: |
| `sight_lines`, median across the cave | 2 m | 3 m |
| `sight_lines`, longest ray | 12 m | **54 m**, along the tube |
| hollow share of the underground | 3.5% | 0.50% |
| land columns crossed by a cave | - | 23.4% |
| `cave_mouths`, openings in the 90 m tier | 0 (1 with the damping at 2 m) | **12**, 11 walkable |
| gather and walk, 90 m tier | - | 12.8 ms, 150 worms, 7,864 capsules |
| one column | 22.3 us | 14.7 us, 1.26 runs mean |

A tenth of the sheet's hollow volume and a quarter of the land crossed: the
worms gather the air into tunnels a player can see down instead of spreading
it into slabs, which is the whole difference. `worm_turn` is 0.12 rather than
the 0.30 first tried, because at 0.30 a tunnel doubled back inside thirty
metres and the longest sight line was 32 m; a bend radius of about seventeen
metres at a two-metre step is a tunnel that reads as one.

**Both ends of a capsule are held under the ground.** The first cut clamped a
worm's position at the top of each step and pushed the capsule to wherever the
step would land, so the far end was clamped only when it became the next
step's start, and the last capsule of every worm never was: the suite caught a
buried worm 0.1 m proud of the surface where the ground fell away. `hold`
clamps the far end at the step it will be, before the capsule is recorded.

**The cross-section changed shape.** `cave_cross_section` at the same column
draws a 10 m room over a 3 m floor of rock over a second 6 m gallery below,
where the sheet drew pancakes with pillars; the rooms are where a worm's
radius wandered wide or two worms crossed.

**A mouth is a gap a walker can STEP into, and the first rule counted holes.**
The opening test was "the gap's roof stands above the neighbour's cap and its
floor is below that cap", which is true of a tunnel running five metres under
the meadow next door: a hole in its roof, not a doorway. The `--view mouth`
frame showed it plainly - a camera on the grass aimed down into a slot at its
feet. The floor must be within a STEP of the ground outside, and the step is
`planet::column::STEP_M`, which `WalkingConfig::step_height` now reads too:
what a walker can climb and what counts as a doorway are one fact, so a mouth
the count offers is a mouth the walker can take. Counted that way the default
tier has 9 openings and 8 walkable, against 12 and 11 under the loose rule and
0 under the sheet.

**The capture picks by SIGHT LINE, because burial was the slab's question.**
`--view cave` took the chamber with the most rock over it, which is the only
thing that distinguishes one two-metre slab pocket from another. With worms
the frames differ by whether the tunnel carries on, so the pick walks cell to
cell at eye height in each of six headings and takes the longest open run,
then aims along it: 16 cells of open tunnel in the default tier, and the frame
is a passage running into the dark with a gallery off it. `--view overhang`
aims at the roof eight metres down that passage rather than the metre of
ceiling over the lens, which was a grey wash at a grazing angle.

### The carve WAS a sheet, and that was its real weakness

The owner asked whether this is Perlin worms. It is not: `hollow` is one
thresholded ridged field, air where `ridged(point / 46 m) > 0.88`. The crest of
ridged noise is a surface, and a threshold near the top keeps a thin shell
around it, so every cave is a slab. That one fact is behind three things this
change measured and worked around rather than fixed:

- `sight_lines` finds a median of 2 m inside a chamber and nothing past 12 m:
  a slab seen edge-on is a wall.
- the cross-sections are pancakes with pillars, and the 3.5% hollow figure is
  spread thin rather than gathered into anything a player would call a tunnel;
- a slab meeting the ground is a LINE of single-cell holes, which is why the
  damping sweep opens 3.7% of columns and `cave_mouths` counts one of them as
  a walk-in.

**The next carve should be tubes, and the cheapest tube stays a pure function
of position:** intersect two independent ridged fields, `ridge_a > t &&
ridge_b > t`. The intersection of two sheets is a curve, and the threshold band
around it is a tube - Minecraft's "spaghetti caves" since 1.18. It is a few
lines in `hollow` with a second seed salt, and the instruments already here
say whether it worked: `sight_lines` should go from metres to tens of metres,
`carve_report` should hold the hollow share, and `cave_mouths` should count
round openings where a tube meets the ground.

**Perlin worms** are the other family: agents that walk a noise-steered path
and carve capsules along it, which gives chosen radii, rooms where a worm slows
or two cross, and mouths for free by starting a worm at the surface. What they
cost is the property everything here rests on: a column stops being a function
of its own direction, because a worm crosses cells. That means a regional
pre-pass - worms seeded per region, carved into every column they touch, and
the tier rebuild reading that region rather than generating per cell. It is
the right tool for caves with intent, and it is its own change; the
intersection comes first because it is an afternoon and it measures.

### The mouth rule, built and measured

The carve's surface damping stays; lace was the right thing to prevent. What is
added is a **mouth**: a rare, seeded patch (`mouth`, on its own noise stream and
never below the shore) inside which two things change in `hollow`:

- the damping is lifted, and
- the carve threshold is LOWERED toward the ground (`mouth_relax`, 0.15 at the
  surface easing to nothing `roof_m` down), so the tunnel sheet flares open
  where it meets the surface.

The second is what made it work. Lifting the damping alone was measured first
(`mouth_sweep`): four columns in a hundred of a patch opened, each a
single-cell hole where the thin sheet crossed the ground. With the flare a
quarter of a patch's columns open.

**The field is fBm remapped to a unit range and rarely reaches its ends**, so
the threshold reads lower than it sounds: 0.70 is 2.2% of the land in a patch
and 0.60 is 17%. Shipped at 0.65, about 7%, which is roughly one patch per
ninety-metre tier - the Minecraft cadence of an entrance every few hundred
metres. The default spawn sits between patches; `--spawn mouth` moves the spawn,
and so the tier, to the nearest one (92 m away, measured).

**A mouth is a change to the RECORD, not only to the column.** Where the carve
broke the ground, the column's top is below the height the record was built
with, and the record is what the terrain pass caps, what the neighbours' walls
go down to, and what the walker stands on outside a cave. Left alone it drew a
meadow over the hole. So `column::build` lowers the record to the column's top,
gives its cap the material that is actually there, and sets every neighbour's
wall height for that side to match; the "one source" test keeps its rule
everywhere the carve did not touch and learns the exception where it did.

**And the wall between two column cells is the column pass's.** The terrain
wall from cap to cap assumes rock all the way down, which across a mouth is a
wall drawn over the opening. Between two cells that both have columns the
terrain pass draws no wall, the column flank runs to its run's own top rather
than stopping at the neighbour's cap, and the surface capture shows the
terraces unchanged.

What it looks like: at a 120 m patch scale the first mouth rendered as a
**crater** thirty-five metres across and fifteen deep - a basin whose floor is
the tunnel floor, with the tunnels leading off its walls (`cave_mouths` counts
thirteen such openings in that tier, every one tall enough to walk into). That
is a way in, and a crude one; the patch scale is 48 m now so a mouth is a pit
with tunnels off it rather than a basin.

### The order, as built

1. `Column::contact` answers run tops. Done, with the wall test.
2. `PlanetContact::stand`, column-aware through the finest record index; the
   top run's floor is the DRAWN cap rather than the layer boundary over it,
   or a walker floats a fraction of a metre over the ground it can see. Done.
3. The walker: min ceiling over the footprint, no headroom is a wall, the head
   clamps to the ceiling after the sweep. Done; the terrace scenarios are
   unchanged and the ECS test stands the walker on a real cave floor.
4. The mouth rule, above. Done.
5. Digging (task 4), which opens a cave anywhere.

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
| a run flank | wherever the run's rock meets the neighbour's air, bedrock to our cap |

Between two cells that both carry columns the terrain pass draws NO wall: the
side is the column pass's, whole. That is the third rule this side has had, and
"One drawer per side" below says why the first two - a flank that started at
the neighbour's cap, then a terrain wall that yielded only where the rock did
not fill the step - each left a band that was nobody's. The top run's own top
cap stays the terrain pass's.

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

### The wall the column pass was trusted to draw, and did not

The terrain pass draws a cap and a wall from it down to the neighbour's cap.
Between two cells that BOTH carry columns it drew no wall at all and left the
side to the column pass, which draws a flank per run against each of the
neighbour's air gaps. That yield was too wide, and the gap it left is the
"holes in the terrain" the owner reported.

**A flank is (this run) against (one of the neighbour's gaps), and between two
ordinary cells there is no such pair spanning the step.** This cell's top run
ends at its own surface and the neighbour's top gap begins at the neighbour's
surface, so the band between the two caps belongs to no pair, and nobody drew
it. Underground the flanks meet exactly, which is why the caves looked right
and the meadow did not.

**Measured, with an instrument built for it.** `PBD_NO_SKY` leaves the
atmosphere shell unspawned and paints the clear colour magenta, so a pixel that
is not terrain is a pixel with no world behind it - which a blue sky over a
ridge can never be told apart from by eye. On the seam view: 407,484 background
pixels with the tier on, 404,936 with `reach_m: 0.0`, and 404,936 again with
the tier on and the terrain wall forced back. So the tier was adding 2,548
pixels of nothing, and the wall it suppressed was the thing that had closed
them. The meadow view closed 4,084.

**The fix is a narrower yield, not a wider wall.** `run_covers` asks whether
this cell's rock fills the step in one run; where it does, the heightfield wall
stands as it always did, and where it does not - a mouth, a tunnel crossing the
edge - the column pass keeps the side and draws it exactly. The cave captures
and the mouth count are unchanged by it.

**The lesson is the one this file keeps: a pass that yields must be able to say
what the other pass will draw.** "The column pass has this side" was true of
the cells it was tested on, underground, and false of every terrace on the
surface, and nothing in the build could tell the difference because a hole in
the ground with sky behind it looks exactly like sky.

### And the flank stops at the neighbour's CAP, always

The other half of the same mistake, found the same day and by the same owner:
a flank was allowed to run to its own run's top on a shared side, on the
reasoning that the terrain pass drew nothing there. Once the terrain pass draws
that step again, a flank that also draws it is a SECOND wall over the first -
and where the two disagreed, the second one stood in the air with no floor
under it and no cap over it. The owner's picture is a meadow with slabs of
grass-topped earth standing in it.

It was also why a single trench wall came out with a stack of grass bands down
its face rather than one at the top. Each flank quad paints its own top metre
with the sod's side (`if top >= hi-1.001`), so a duplicate flank drew a second
transition partway down the wall the terrain pass had already drawn correctly.
One rule, one wall, one transition.

**And the run's top is not the cap.** `planet_column.rs` says it plainly where
it decides whether a record is a mouth: "outside a mouth the column top is the
height rounded up by less than a layer and the record is left exactly as it
was". A cell's surface is a float; its topmost solid LAYER ends at the next
whole metre above it. So an unclamped flank ran up to a metre above the cap the
terrain pass draws - one cell of wall standing proud of the ground, wearing the
sod's side because that is what a flank paints on its top metre. That is the
owner's "extra layer of wall where there should just be air", named exactly.

Measured by bisection, with each drawer disabled in turn: with the flank off
the slab is gone and with the terrain wall off it remains, which is what names
the flank as the one drawing it. After the clamp the seam view's background is
404,936 pixels, equal to the tier being off, and `cave_mouths` still counts 9
openings with 8 walkable.

### One drawer per side, and the metre of rock the cap was drawn inside of

The owner's picture, the third on this seam: a pit dug at a cave, its walls
part stone, part earth, part sky. Reviewed against Tenebris again, whose
`build_tile` has ONE rule for a side face - a quad wherever a solid voxel meets
a neighbour that does not cover it, at the same depth - and nothing else.

**Two defects, and the first one made the second one necessary.**

**1. Every column stood one layer proud of its cap.** `column::generate_solid`
filled every layer whose altitude was under the generator's UNFLOORED altitude
(74.6 m: solid through the layer at 74), while the record was drawn at the
floored one (74.0). So every cell in the tier carried a metre of rock its cap
sat a metre inside of, and the tree kept saying so in its own words: "outside a
mouth the column top is the height rounded up by less than a layer". Three
things followed, and each had been worked around rather than named:

- **The aim ray dug the invisible layer first.** `sample_at` answers solid off
  the COLUMN, so a click on the ground took the rock over the cap, which
  changed nothing anybody could see, and a slanted dig into a wall entered the
  neighbour under its top layer and left a roofed pocket with no way in.
- **The first edit to any cell raised its cap a metre.** `reconcile_surface`
  sets the record to the column's top, which was one above it, so the ground
  popped up under the player on the first dig.
- **Both wall rules had to dodge it.** The flank was clamped to the
  neighbour's cap because its run top stood proud of the cap (the slab in the
  meadow); the terrain wall yielded only where the rock filled the step in one
  run because a flank could not be trusted above the cap. Each rule answered
  half of the side, and on a side where the rock did NOT fill the step - a
  cave mouth breaking the surface run, or a pit dug sideways into a wall -
  neither drew the band. From inside the pit that band is the sky.

`column::surface_m` is the one function now: the generator's altitude,
floored to the layer, read by the column and by the record
(`planet::surface_height`) alike, so a cap and the column top under it are one
number by construction. `every_column_top_is_its_records_height` holds it on
the whole tier, `PlanetContact::stand` lost its special case for the rounded
top, and the mouth test asserts equality rather than "within a layer".

**2. With that, the side is the reference's rule and nothing else.** Between
two tier cells the terrain wall is not drawn at all; a flank is (this run)
against (one of the neighbour's air gaps), `max(lo, gap.lo)` to
`min(hi, gap.hi)`. No clamp to the neighbour's cap, no `run_covers`, no
float-cap adjustment at either end, because there is no float cap. A pass that
yields must be able to say what the other pass draws, and now it says "all of
it".

**Found by an audit, not by a picture.** `every_side_of_the_tier_draws_what_
the_reference_exposes` transcribes the shader's band arithmetic (the run words,
`column_gap`) into Rust and holds it, on every shared side of the real tier,
equal to the reference's rule computed from the columns alone. Under the two
half-rules it failed on the BUILT tier, before any edit: `cell 33816 side 3
draws [] where the reference exposes [(58.0, 60.0)]` - two metres of a natural
mouth's wall drawn by nobody - and again after a layer was dug into a
neighbour's side. `a_dug_or_placed_cell_and_its_neighbours_draw_what_is_
exposed` digs a pit, digs sideways into its wall, and places a block back in
each, and audits the cell and its ring after every step.

**And a flank wears what a wall wears.** The flank was painted in the run's
BOTTOM material, top metre excepted, so the first metre under a pit's grass was
stone beside a terrain wall drawn in earth. A flank now carries the run's top
material and measures its depth from the run's own top, so `face_code` gives
it sod, earth, then stone exactly as it gives the terrain wall, and a cave run
whose top is rock is rock. This is Tenebris's rule as well: a voxel's side
wears the voxel.

**The capture that found the first defect** was `--walk --fixed-dt --pitch -40
--dig 8 --dig-ahead`: the scripted dig logged its three layers as 220, 219 and
218 of a cell whose top was 221 - a pocket under an unbroken roof, lit to 0 of
15 - and the flank of that roof's one-layer run stood in the frame as a wall
of earth. `--pitch` is new for exactly this: a headless walker looked dead
level, and a pit a player digs is a slanted run of cells, each taken from a
different column at a different layer.

**Measured, the same capture on the fixed build.** The three layers are 220,
219 and 218 of a cell whose top IS 220, so the pit is open to the sky and each
is lit to 15 of 15; with the sky off (`PBD_NO_SKY`) the frame has 0 background
pixels; the frame mean is 75.3 of 255 against the roofed pocket's 19.2.
`docs/screenshots/dig-pit-before.png` and `dig-pit-after.png` are the pair,
and `column-mouth.png` is a natural mouth on the same build. The audit passes
on the built tier and after every edit in its sequence.

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
