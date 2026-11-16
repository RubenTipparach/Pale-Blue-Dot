# Design

## Context

- **Craft today** (`crates/pbd-core/src/vehicle/`): each craft is one
  `RigidBody` (`body.rs`) integrated by the core at the substep rate
  (`SUBSTEPS = 4` per fixed tick in `pbd-app/src/vehicles.rs`), because
  buoyancy is stiff; the app places the Avian body where the core says.
  Hulls float on the sea table (`hull.rs`), and `Surroundings` hands in the
  sea, its state, the tide, the wind, gravity and the depth under the craft.
  There is no water velocity in it.
- **Lines today**: `Mooring { at, length, anchored }` ties a craft's bow to
  a fixed point. `Craft::moor` is slack until taut, then
  `k = min(40 m, 4000)` N/m with damping `1.6 sqrt(k m)`, pulling only
  (`vehicles` design: "A mooring line has to scale with what it holds"). An
  anchor is a mooring whose point is where it was dropped, at most
  `ANCHOR_DEPTH_M = 35` m deep, rode 3 x depth + 2 m. It never drags.
- **Flowing water today** (`water-flow`; `pbd_core::liquid`,
  `pbd-app/src/water_flow.rs`): cells with levels 0 to 7 under Luanti's rule,
  one tick a second. The mesher gives each flowing top a flow direction (the
  lane the shader pans the ripples along, `FlowingWater`), and the player can
  show flow arrows (`FlowArrows`, the setting or F5). There is a direction
  but no speed.

## Goals / Non-Goals

**Goals**
- A river's current moves what floats on it, in every craft, with one rule.
- Lines between two craft, and anchors that can drag.
- Rafts, logs, booms, bundles and nets as things on the water.
- The mockup first, the port on a request.

**Non-Goals**
- Rapids or waterfalls as a separate water simulation: the current is read
  off the cells the game already has.
- Swamping and capsizing on a river (the sea's bilge and heel rules stand).
- Multiplayer authority for lines between two players' craft (one rule,
  written once, as for every craft).

## Decisions

### 1. The current is derived from the cells, never saved

A flowing cell already has a direction. The port gives it a speed from the
water's own slope: the drop in surface height (layer plus level eighths) to
the downstream neighbours over the cell spacing (2.833 m), through a
Manning-like rule `v = k sqrt(slope) * depth^(2/3)`, clamped. A source, a
still pool and the sea give zero (the sea has its own tide and swell). A
falling cell adds a strong downward-and-outward push. The value at a point
blends the three nearest cells, so a hull crossing a cell boundary feels no
step.

- Why not a velocity field simulated alongside the cells: the cells are
  authoritative and saved; a second field would be a second source of truth
  to keep in step. A derived value is free to change between versions.
- Why speed from the slope: it needs no new state, it is zero where the
  water is level, and it is largest at a fall, which is what the owner plays.

The mockup's river is one smooth field (a centre line, a cross-profile,
cylinder flow round rocks with an eddy in their lee) as a stand-in for the
field the cells would give. Eddies behind rocks are not in Luanti's rule;
recommendation taken (ask only with screenshots): in the port an eddy is a
cell-scale term added behind any solid cell that stands in a current, the
same shape as the mockup's.

### 2. Drag toward the current at the float points

`Surroundings` gains `current(p) -> DVec3` (horizontal). Each hull's float
points (the probes `hull.rs` already sums buoyancy over) feel drag toward
the current, split along and across the hull:
`F = c (u - v) (|u - v| + 0.25)`, with `c` per craft (a raft is blunt both
ways, a barge and a canoe slip along and resist sideways). Different
currents at different probes give the spin from shear and the grip of an
eddy with no extra term. The water's slope pushes the hull downhill
(`- m g grad(h)`), which is what carries it over a fall.

The mockup steps at 1/120 s with semi-implicit Euler and a handful of points
per body; the game keeps its substep rate and its probes.

### 3. Lines between craft

`Mooring` becomes a line with two ends, each a craft's local point or a
fixed point. The pull rule is the existing one, with `m` the reduced mass of
the two ends, and the pull applied equal and opposite. Each line has a
length the player sets (Long, Short) and can be cast off and made fast again
within reach. The drawn rope is derived state: a chain of points that sags,
floats or sinks and is dragged by the current, never read back.

### 4. Anchors hold by bed, set and lift

An anchor on the bed holds up to `hold(bed) x set x lift`:
- `hold`: by the bed cell's material: sand 600 N, mud 2600 N, rock 4200 N
  (the mockup's numbers, for a skiff; scaled by anchor size in the port).
- `set`: grows with the distance dragged: mud reaches 1 in about 1.2 m, sand
  stops at 0.25 (it ploughs), rock stays near 0 until a fluke catches (a
  chance per metre), then 1.
- `lift`: `1 - (a / 0.5 rad)^2`, a the rode's angle at the anchor. A short
  rode breaks an anchor out; that is how it is weighed.

When the line's pull exceeds the hold, the anchor drags toward the craft at
`(T - hold) / 320` m/s and leaves a furrow. Paying out (to five times the
depth by default) and shortening change the line's length. A second anchor
at the stern makes a two-point mooring. The existing anchor's depth limit
and default rode stand.

### 5. Rafts, logs, booms and bundles

- A **raft** is a craft built from logs (the player's own, `tool-crafting`),
  poled where the bed is within 2.7 m and paddled elsewhere.
- A **log** dropped in water is a floating body (a capsule) until it beaches
  or is picked up.
- A **boom** is a chain of logs between two bank fixings: a few points with
  drag toward the current, held to its length, that logs and craft push
  against. Opening one end swings it aside.
- A **bundle** binds the logs held at a boom into one body (their mass, a
  wider beam) that a craft can tow on a line.
- A **jam** is emergent: logs crosswise in a gap one cell wide. Prying the
  key log (the furthest downstream that holds the rest) turns it end-on and
  shoves it through.

### 6. Nets

A net is a cloth of points (13 by 3 in the mockup: corks that float, a lead
line that sinks, twine that bunches but does not stretch), made fast to two
spars at a boat's stern. Each point is dragged toward the current; the sum
is the pull on the boat. A fish the net sweeps round (inside the cork line)
or swims into (the twine within 0.6 m) is caught and rides in the bag, adding
drag. Hauling draws the net to the spars and lands the catch. In the port
the schools are `fishing-and-equipment`'s, and the net is a tool beside the
rod with its own catch rule.

### 7. Saves

Every craft, line, anchor (its bed point, set and rode), boom and bundle is
a record on the durable path, named with its author. A world saved before
this change has none of them and opens unchanged. The current is derived and
never saved.

### 8. The look

The mockup draws everything in the game's world style: hex cells with
blocky pixel textures (16 px a tile, sampled nearest), no outlines on world
objects, lit like the ground. The water is a cut-down stand-in for
`water.wgsl` (its deep and shallow colours, a ripple carried along the flow
in two phases, foam where it runs fast, glints from every lamp). At night
each boat carries a lantern (the anchored skiff's is white, an anchor light)
and a lantern post stands on the bank; every lamp lights what is near it,
and the moon is dim.

## Risks / Trade-offs

- **Stiff lines on light craft.** The mooring rule's cap was found the hard
  way on a 30 kg canoe; tow lines between heavy barges use the same cap and
  stretch about 0.3 m at 1.3 kN, which reads as rope, not as a bug.
- **Emergent jams can be permanent.** Logs that beach in slow water at a
  bank never move again. The mockup's Break jam also frees stranded logs;
  the port needs the same (a pike pole), or a log drive stalls.
- **The derived current depends on the water rule's tick.** Levels change
  once a second; the speed is blended over the tick so a hull does not feel
  the step.

## Open questions

None for the owner yet: the mockup is the question. If its look is
approved, the in-game captures go to the survey beside the mockup's shots.

## The inner tube (owner, 2026-11-16: "also what about innertubes?")

A sixth tab, Tube (`#tube`): one person in an inner tube on a slow, shallow
reach. The ring is a hull with no keel (equal drag along and across, 115 kg
with its rider), so the current carries it; a torque from the difference in
current across the ring turns it slowly where one side runs faster (a bank,
a rock, a riffle's edge), damped so it never spins up. Hands give 75 N of
push and a small turn. **Get off** (E) wades or swims to the nearer bank
and picks the tube up, carried on the back; **Get on** walks in to waist
depth and sits back into the ring.

Made from what the game has (it has no rubber or hide), proposed for the
port: **pine pitch** (3 pine cones on the campfire) and an **inner tube**
(6 wool, 2 rope, 2 pine pitch at the workbench), a pack item placed on water
like a boat. A tube left on water drifts; one on the bank stays.
