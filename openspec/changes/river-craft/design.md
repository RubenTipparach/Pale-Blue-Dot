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

### 4. A set anchor holds the boat (owner, 2026-11-19: "anchor in mockup should stop you from moving")

Version 3 of the mockup let a dropped anchor drag in sand, broke it out when
the rode was shortened, and let rowing move the skiff on. The owner, on the
Anchor tab: "anchor in mockup should stop you from moving". The rule is now:

- **Dropped, it sets.** The anchor falls to the bed and sets over `set_s`
  seconds: sand 1.8, rock 1.3, mud 0.8, so under about two seconds on any
  bed. The bed decides only that. While it sets, a pull over `slide_n` (600 N
  on sand, 900 on rock, 1200 on mud) slides it toward the craft at
  `(T - slide) / 320` m/s and leaves a furrow.
- **Set, it holds.** From then on nothing moves it: not the current, not
  rowing, not shortening the rode. The craft swings on the rode, at most the
  rode's length round the anchor, and settles downstream of it. Rowing away
  from the anchor brings the rode taut and the craft stops (the line's pull
  is the existing mooring rule, stiff enough that the stretch is about
  0.1 m at 500 N). The HUD shows "Anchored".
- **The rode stops at the bed.** By default the rode pays out only until
  the anchor touches the bed, then stops: no extra scope is veered (the
  owner, 2026-11-19: "by default anchor rope should stop extending as soon
  as it touches ground"). A longer rope is set in the boat's menu (10),
  for a longer swing.
- **One anchor per boat** (the owner, 2026-11-19: "also only really need
  one anchor per boat"). Every boat that anchors has one, at the bow, and
  swings on its one rode. The stern anchor and the two-point mooring of
  versions 3 to 5 (bow and stern together holding the craft still on its
  heading) are dropped. The game already has one: `Craft::mooring` is a
  single `Mooring`, and nothing changes there.
- **Weighing is the only way to move again.** It hauls the rode in to
  straight up and down and lifts the anchor aboard.

Dropped as proposals: the sand anchor that never sets and drags, the rock
anchor that skips until a fluke catches, and a short rode lifting the shank
and breaking the anchor out. The rope's length changes only how far the
craft can swing: lengthened, the rode is veered at 1.5 m/s and the craft
drops back; shortened, it is hauled in at 1.0 m/s and the craft is pulled
up toward the anchor, never breaking it out. The existing anchor's depth
limit (35 m) stands; its default rode (`place::rode`, 3 x depth + 2 m)
becomes "to the bed" for a new drop, and a rode already saved keeps its
length.

### 5. Rafts, logs, booms and bundles

- A **raft** is a craft built from logs (the player's own, `tool-crafting`),
  rowed with two oars on a rowing frame (9), in any water that floats it.
  No pole (the owner, 2026-11-19: "no pole just oar is best").
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
never saved. The boat's menu adds two settings to a craft's record, the
anchor's rope length and the lantern put out (10.4).

### 8. The look

The mockup draws everything in the game's world style: hex cells with
blocky pixel textures (16 px a tile, sampled nearest), no outlines on world
objects, lit like the ground. The water is a cut-down stand-in for
`water.wgsl` (its deep and shallow colours, a ripple carried along the flow
in two phases, foam where it runs fast, glints from every lamp). At night
each boat carries a lantern (the anchored skiff's is white, an anchor light)
and a lantern post stands on the bank; every lamp lights what is near it,
and the moon is dim.

### 9. Rowing the raft (owner, 2026-11-19: "mockup better rowing on a raft")

Built in M34 as a seventh tab, **Rowing** (`#rowing`), with a Today /
Proposed switch (the raft's own lantern, 9.8, lit only in Proposed), a first- and third-person view, and the tunables on
sliders that print the RON below. The method is the `animation` skill's.

#### 9.1 What the game does today, measured

Read off `crates/pbd-core/src/vehicle/loon.rs` (the stroke and its force),
`crates/pbd-core/src/vehicle/spec.rs` (`LoonSpec::raft`, `vehicles.ron`
`raft:`) and `crates/pbd-app/src/vehicles/draw.rs` (`pole`, `paddle`,
`paddle_blade`, `Moving::Paddle`):

| What | Today |
| --- | --- |
| The beat | A stroke starts when the last has ended and a key is held: power 0.8 s, recovery 0.7 s, so 1.5 s a stroke, 40 a minute. |
| Which side | W alternates sides every stroke; A strokes on the right, D on the left. |
| The force | One "blade" 0.7 m² (Cd 1.25) at x = ±1.35 m, 0.3 m under the waterline mark, dragged aft 1.4 m from z = -0.2; its speed is `stroke_m / power_s * pi/2 * sin(pi u)`, against the ground (`pole: true`), so it pushes off the bed in any depth. |
| The drawn pole | Hangs from a fixed hand at (0.62, 1.45, 0.55) on the right. It leans 0.5 rad while W or S is held, stands at 0.12 otherwise, tilts 0.35 rad sideways while A or D is held. It is set from the keys, not from the stroke: it snaps to the lean on the frame the key goes down and snaps back on release. |
| The water | Nothing: no splash, no swirl, no wake from the pole or the stroke. |
| First person | The seat camera at the poler's eye (0.35, 1.75, 0.7); the crew figure is hidden, so the player sees a pole floating in front of them, no hands. |
| Third person | The blocky poler has no arms; the pole hangs beside him. |

What reads badly, in the order the owner will see it:

1. **The pole does not do the stroke.** The force works 1.35 m out on
   alternate sides at 40 a minute while the drawn pole stays on the right
   and holds one lean. Nothing on screen shows a stroke: no reach, no catch,
   no push, no release, no recovery. The skill's "do not move a tool only by
   pitching it about a fixed fist" is exactly today's pole.
2. **Every change is a snap.** Pressing W tips the pole 0.62 rad in one
   frame; letting go tips it back in one frame.
3. **The raft fishtails without a cause.** W strokes alternate sides 1.35 m
   out, so the raft yaws one way then the other every 1.5 s; the drawn pole
   gives no reason for it.
4. **The surge has no event.** The push rises and falls as a sine through
   the power phase; there is no catch for the surge to start on, and no
   splash or puddle for the eye to tie it to.
5. **No hands, no body.** First person shows a floating pole; third person a
   man standing beside one.
6. **The pole works in any depth.** It pushes off the bed in 6 m of water.

#### 9.2 Paddle, sweep, pole or oars: two oars, no pole

| Way | For | Against |
| --- | --- | --- |
| A paddle per side (the Loon's) | Built (the Loon); one rule | A 2.4 m raft is too wide to paddle from the middle: the paddler must kneel at an edge and change sides for every correction; weak on 1.3 t of green logs |
| A sweep oar at the stern | The log raftsmen's way; steers well | It steers more than it drives; one person sculls slowly; better kept for the big log bundles (design 5) |
| A pole | Built; pushes off the bed, so it beats the current upstream (the owner, 2026-11-18: "the raft guy pushing the raft up and down the river") | Only where the bed is in reach; it has to be carried along the raft and replanted every stroke, and drawn it clips the deck and the poler |
| **Two oars on a rowing frame** | The real way to row a raft (a river oar rig); W rows both, A or D rows one, so the turn is plain; seated and facing the bow, so the player sees where the raft goes | New: a frame, two oars, a seat |

**Two oars on a rowing frame, rowed facing the bow (the push stroke). No
pole.** Version 3 proposed the pole beside the oars for the shallows. The
owner, on the Rowing tab of version 3, 2026-11-19: "no pole just oar is
best, pole you have to actually move, and the pole is clipping, overall too
much awkwardness". So the raft carries oars only; the proposed pole, its
stages, its "Too deep" chip and its tunables are dropped from the design and
the mockup. Today's game pole (9.1) stays on the page only as the
comparison.

**Shallow water without a pole.** Recommendation taken (ask only with
screenshots):

- The oars work at any depth that floats the raft. The push is always the
  blade's drag through the water (9.4), never a push off the bed.
- Where the bed (or a bank) is shallower than the blade's stroke depth, the
  blade comes down on it and rests there, `bed_clear_m` (0.1 m) above the
  bed's top: a bump, no splash at the catch, and only the share of the blade
  still in the water pulls. The HUD shows "Oar on the bed". Nothing pushes
  off the bottom, so a raft aground is rowed off by the water still under
  the other blade, or not at all.
- Going upstream is rowing against the current: the Rowing tab's 1.28 t
  raft, rowed steadily with W, makes about 0.5 m/s through the water
  (measured on the page, 0.42 to 0.53 over a stroke), so it barely holds its
  own on that tab's 0.45 m/s reach and cannot climb fast water. Today's push off the bed in any depth
  (9.1, problem 6) is gone with the pole.

#### 9.3 The oar stroke: five stages

One oar swings about its pin: a sweep angle `theta` about the pin's upright
(+ toward the bow), a droop `phi` that puts the blade in or out of the
water, and a feather `psi` about the loom (square in the water, flat in the
air). The hand holds the handle end, so the hand moves on an arc about the
pin, and the arm reaches it from a shoulder that swings with the body about
the hip. The stages map onto the skill's four:

| Stage | Skill stage | What happens | Curve |
| --- | --- | --- | --- |
| **Reach** | windup, anticipation | Hands drawn in to the chest, blades forward at `catch_deg`, squared; the body upright, a little back (`lean_catch_deg`). It coils `coil_deg` further and holds. | slow ease into a held pose |
| **Catch** | hit | The hands lift, the blades drop in: `phi` from clear to buried. Fastest as the blade meets the water; the splash and the start of the surge land on its end. | ease in (t cubed) |
| **Drive** | hit, carried | The hands push forward and the body swings forward (`lean_release_deg`): `theta` from `catch_deg` to `release_deg`. The force is the blade's drag through the water, so it peaks mid-drive. The blade throws a wash. | ease in-out (half cosine), the body leads by `body_lead` |
| **Release** | recoil | The hands press down, the blades lift out and feather flat in a wrist flick; the sweep carries `follow_deg` past the release. Two swirls (the oar's puddles) are left where the blade came out. | quick kick, ease out |
| **Recovery** | recover | The hands draw back to the chest, the body comes upright, the blades travel forward flat and clear, dripping. | ease in-out |

Backing (S) runs the same stages with the sweep reversed: the catch at
`release_deg`, the drive toward `catch_deg`, the body leaning back on the
drive.

#### 9.4 One beat; the surge and the turn

- **One clock for both oars.** While any row key is held one stroke clock
  runs, period `P = reach_s + catch_s + drive_s + release_s + recovery_s`
  (1.70 s by default, 35 strokes a minute). An oar whose key goes down
  mid-stroke waits for the next reach, so the two oars never fall out of
  step. An oar whose key comes up finishes its stroke and rests at the
  reach pose; nothing snaps.
- **Keys.** W rows both oars; S backs both; A rows the right oar alone
  (the raft turns left); D the left (it turns right); Q and E row the left
  or right oar on its own, held with W or alone. A and D with W row both,
  the turn's side a little harder (`turn_share`).
- **The surge is the force, not a drawing.** The blade's own velocity (from
  the schedule) plus the raft's at the blade, less the water's, makes the
  drag `F = -1/2 rho Cd A s |v| v` (`s` the share of the blade under
  water), applied at the blade. So the raft surges from the catch to the
  release and glides through the recovery, and one oar alone pushes 2.75 m
  off the centre line and turns the raft with no separate turn force. The
  physics reads the stroke schedule (it is the rule); the arms, the body
  and the splash only draw it.
- **The bed.** Where the bed under a blade is higher than the blade's
  stroke depth, the blade rests on it `bed_clear_m` above it and pulls only
  with its share still in the water (9.2); it never pushes off the bed.

#### 9.5 The water's answer

- **Catch:** a splash at the blade: `splash_drops` drops thrown up and out
  at up to `splash_speed_mps`, falling under gravity, and a small ring.
- **Drive:** wash at the blade, a few drops and foam a step.
- **Release:** two swirls, the puddles an oar leaves (a pair of eddies
  turning opposite ways, one at each edge of the blade), `puddle_m` across,
  carried by the current, turning, and fading over `puddle_s`.
- **Recovery:** drips off the blade, `drips` a stroke.
- Drops and swirls are lit as the water is: the sun, the sky and every lamp
  near them (the lantern on the raft at night). Drawn only; nothing reads
  them.

#### 9.6 First person

The rower's eye rides the body: it moves forward with the lean on the drive
and back on the recovery (`head_follow`, a share of the shoulders' move),
added to the view, never to the mouse look. Each hand is the approved hand
of M36 Tools in Hand (the Hand tab, `hand-model` design 5, approved
2026-11-17), copied into the page from `tool-motion.html`, closed on the
oar's handle, its arm turned toward an elbow solved from the shoulder (two
bones, the elbow out and down). The hands and arms are lit as
the terrain is (`LitByField`, `LitLikeTerrain` in the port): at night they
are as dark as the raft except where the lantern reaches. Third person draws
the same pose on the blocky rower, his arms two blocks a side and a block
hand round each handle. **Each part is drawn in exactly one view**: the
first-person hands and arms only in first person, the rower's body, legs and
arms only in third (the owner, on version 3 in third person at night: "why
is fps arm in third person").

#### 9.7 Tunables

For `assets/config/vehicles.ron`, `raft:`, beside `paddle:` (which stays for
the Loon and the tube), validated at load (every time and length positive,
`release_deg < catch_deg`, the blade's droop able to reach the water).
Defaults; the Rowing tab's sliders print the same block:

```ron
oars: (
    pin: (0.0, 0.62, 1.0),      // m: along (+bow), over the waterline, out (mirrored for the left)
    length_m: 3.0,              // m, handle end to blade tip
    inboard_m: 0.95,            // m, handle end to the pin (the hand grips 0.12 m in from the end)
    blade_m2: 0.14,
    blade_cd: 1.3,
    catch_deg: 22.0,            // sweep at the catch, + toward the bow
    release_deg: -16.0,         // sweep at the release
    coil_deg: 3.0,              // the reach's extra coil before the catch
    follow_deg: 3.0,            // the release's carry past release_deg
    blade_depth_m: 0.12,        // blade centre under the water through the drive
    blade_clear_m: 0.22,        // blade centre over the water through the recovery
    feather_deg: 80.0,
    turn_share: 0.35,           // A or D with W: the outer oar's extra sweep share
    bed_clear_m: 0.1,           // a blade on a shallow bed rests this far above it (9.2)
),
stroke: (
    reach_s: 0.16, catch_s: 0.12, drive_s: 0.62, release_s: 0.14, recovery_s: 0.66,
    exaggerate: 1.0,            // scales coil, follow-through, lean and the head's follow together
),
rower: (
    seat: (-0.52, 0.62, 0.0),   // m, the hip; the feet braced on the frame's beam
    shoulder_m: 0.5, shoulder_half_m: 0.27, upper_arm_m: 0.3, forearm_m: 0.37,
    lean_catch_deg: -6.0, lean_release_deg: 22.0,
    body_lead: 0.25,            // share of the drive the body swings ahead of the arms
    head_follow: 0.6,
    hand_scale_m: 0.03,         // metres a hand-model unit (the fist closes on a 34 mm handle)
),
water: (
    splash_drops: 16, splash_speed_mps: 2.4, drips: 5,
    puddle_m: 0.24, puddle_s: 4.0, puddle_spin_rps: 0.6,
),
```

#### 9.8 The raft's own lantern (owner, 2026-11-19: "Rafts should also have a light that comes with it at night")

- **Today.** The game's raft already carries a lantern on its bow post
  (`vehicles/draw.rs`, `Builder::lantern`): an emissive flame
  (`Moving::Flame`) shown while `DuskLamps.lit`. It lights nothing: it is
  not a lamp in the light field, so at night the raft, its poler and the
  water round it are as dark as open water, with a glowing box at the bow.
  The raft's own meshes are lit as the terrain is (`LitByField`,
  `LitLikeTerrain`, `craft-lighting`), so they would answer to a lamp if
  there were one.
- **Proposed.** Every raft comes with its lantern fitted: nothing to craft
  or place, and it cannot be taken off. It is lit at dusk and out at dawn on
  the same switch as the town lamps (`DuskLamps`, `lamps-and-lanterns`
  decision 1), and while lit it is a lamp in the light field's lamp term:
  it lights the raft's deck, the water round the raft, the oars, the splash
  and the swirls, and the rower's first-person hands, with the town lamps'
  colour and falloff. The raft stays lit like the terrain.
- **How it joins the field.** The town lamps are baked into the cell light
  field (a lamp record in the column, relit twice a day). A raft moves every
  frame, so its lamp cannot be a baked cell record without a re-bake as it
  drifts. Recommendation taken (ask only with screenshots): a short list of
  moving lamps (the four nearest lit craft lamps to the camera), uploaded
  each frame and summed into the same lamp term the field-lit shaders
  already read (terrain, water, `LitLikeTerrain` meshes, the held hand), at
  the bake's colour and falloff. Limitation, stated: a moving lamp has no
  baked shadow or bounce; the deck and the rower do not shadow it.
- **Tunables**, in `assets/config/vehicles.ron`, `raft:`, beside `oars:`
  (`LoonSpec` gains `lamp: Option<CraftLampSpec>`, `None` for the Loon and
  the tube, so their saves and looks are unchanged):

```ron
lamp: (
    at: (-0.95, 1.45, -1.95),   // m, the bow post's lantern, the game's frame (+x right, -z the bow)
    range_m: 15.0,              // where the light falls to nothing (the town lamps' falloff; a pool about 8 m across reads on the water)
    colour: (1.0, 0.62, 0.30),  // linear, the town lantern's warm flame
    intensity: 2.4,             // the town lantern's, in the lamp term's units
),
```

  The mockup's Rowing tab prints the same block under `lamp:`.
- **Saves.** The lamp is part of the raft, and whether it burns is the
  clock's answer, unless the player puts it out in the boat's menu (10.3),
  which the craft's record keeps (10.5).
- **The inner tube has none** (the owner, 2026-11-19: "no light for tube
  guy"): at night a tube is lit only by the lamps near it and the moon.

#### 9.9 In the game (on a request, after the owner's look)

- `pbd_core::vehicle`: a `RaftState` (the stroke clock, each oar's stage
  and whether it rows, the gear) and one function `oar_pose(spec, clock,
  oar)` that gives the stage, the sweep, droop and feather and the impact
  events. `loon.rs`'s stroke is left as it is for the Loon and the tube;
  the raft's force reads the oar's blade from `oar_pose`. Tests pin the
  schedule: which stage at which time, the catch event on the catch's end,
  one oar joining only at the next reach, no pose jump when a key comes up,
  a one-oar stroke turning the raft toward the other side, a blade over a
  shallow bed resting on it and pushing nothing off it.
- `pbd-app` `vehicles/draw.rs`: the oars and frame as moving parts reading
  `oar_pose`; the first-person hands from `held_tools.ron`'s approved hand,
  closed on the handle; splash, drips and swirls as particles lit by the
  field (`LitByField`, `LitLikeTerrain`).
- `vehicles/draw.rs` and the field-lit shaders: the raft's lantern as a
  moving lamp (9.8), with a test that it is lit only while `DuskLamps.lit`
  and that a raft without one (the Loon) adds no lamp.
- Saves: the frame and oars are part of the raft; nothing new is stored
  except the gear in hand, which is derived on boarding.
- Not measured here: frame cost (no GPU in this session), and the feel of
  the beat in real time (judged on the owner's machine).

#### 9.10 Version 4: the stroke smoothed (owner, 2026-11-19: "overall too much awkwardness")

Measured on version 3 with a clearance check (a measurement instrument run
from the screenshot script, not part of the page: it samples the stroke at
160 points, ahead and back, at full and turn range, and reports the closest
approach of each oar, fist and arm to the deck, the frame, the crate, the
bench and the rower's torso, head and legs):

- The arms could not reach the handles: from the shoulder to the grip at
  the release was 0.92 m against an arm of 0.6 m, so the drawn forearm
  stretched and the first-person hand floated off its sleeve.
- The loom ran through the top of each post (the post rose 6 cm above the
  bottom of the loom at the pin).
- The rower had no front: a plain head and hat, so from most third-person
  angles he could be read as facing the stern.

Version 4: the seat 0.26 m nearer the pins (x -0.52), the sweep 38 degrees
(catch 22, release -16), a forearm length of its own (0.37 m), the
shoulders at the torso's sides (0.27 m out), the grip 0.12 m in from the
handle's end, the posts 2 cm lower with a swivel oarlock (two iron horns
either side of the loom, turning with the sweep), the catch's lift and the
release's press overlapping the sweep a little so the hands go round one
rounded loop. Now every clearance is positive: the closest are the oar to
the torso when backing (4 mm), the loom over a post (1 cm), the arm 1 cm
short of straight at the release. The rower faces the bow: a cap peak and a
nose forward, hair at the back, the feet braced on the frame's beam. Today's
pole hangs just outboard of the logs (z 1.3 m) and tilts outboard for
either key, so it no longer runs through the deck and the poler; what it
shows (a pole leaning on the held key, snapping) is unchanged.

### 10. The boat's menu (owner, 2026-11-19: "rope length can be controlled in the boat menu, yes boats have menus for inventory and various stuff")

The owner, on the Anchor tab of version 5, where the rope now stops at the
bed: "rope length can be controlled in the boat menu, yes boats have menus
for inventory and various stuff". And with it: "also only really need one
anchor per boat" (4). So a longer rope is not a pair of buttons on the
screen; it is a setting in the boat's own menu, beside its hold.

#### 10.1 What the game has today, measured

Searched `crates/pbd-app/src/vehicles*`, `crates/pbd-core/src/vehicle/`,
and `crates/pbd-app/src/desktop/` (`pack.rs`, `slots.rs`, `menu.rs`, the
station screens).

- **No boat menu.** No screen belongs to a craft. Aboard, the only UI is
  the craft's panel of key rows (`vehicles/hud.rs`, `panel`: the kind's
  name, "at anchor" or "tied up", the keys, T to drop or raise the anchor,
  F to step off, V for the camera, B to bail while there is water).
- **A hold, shown inside the pack.** A raft has a hold of ten slots
  (`pbd_core::vehicle::hold`, `HOLD = 10`, `small-boats` design 5). The
  pack (I or Tab) shows it as a HOLD panel under the pack's own grid while
  the player is aboard a raft or on foot within `HOLD_REACH_M` of one
  (`desktop/pack.rs`, `find_hold`, `paint_hold`, `click_with_hold`). A click
  picks up and places as in the pack; a shift-click sends a whole stack
  across (`hold::send`). It is stored as its own record (`HOLD_KIND`, keyed
  by the craft's id) in ONE line with the player's slots
  (`vehicles/hold.rs`, `WorldSave::store_with_hands`), so a moved stack is
  never in both or neither. Only the raft has a hold; the Loon and the
  inner tube have none.
- **One anchor, no length control.** T drops the anchor where the water is
  at most 35 m deep (`ANCHOR_DEPTH_M`) with a rode of 3 x depth + 2 m
  (`place::rode`), or casts it off; `Craft::mooring` is one
  `Mooring { at, length, anchored }`. Nothing sets the length afterwards.
- **What a craft's record keeps** (`vehicle/record.rs`, `VehicleRecord`):
  id, kind, frame, pose and velocities, the water aboard (`bilge_kg`), the
  mooring (point, length, anchored), `owner` (always none in a one-player
  world), a harbour berth, and `made`. There is no name and no damage: the
  kind's name ("Raft", "Inner tube", "Loon") and the water aboard are all
  the game knows of a boat's identity and condition.
- **The lantern** is the raft's, fitted and lit by the clock (`DuskLamps`);
  nothing turns it off (9.8).
- **Keys in use** (`controls.rs`, `BINDINGS`, and the readers): W A S D,
  Q E, F (interact, board), G, B (bail, brake, breeding), T (anchor), V
  (camera), X, M, R, H, P, L, J, I and Tab (pack), O, Space, Shift, Ctrl,
  Alt, Esc, the digits and F-keys. **C, K, N, U, Y and Z are free.**

#### 10.2 Opening it

- **C while aboard** opens the boat's menu; C or Esc closes it, like every
  menu screen (it holds the pointer, `MenuOpen`). Recommendation taken (ask
  only with screenshots): C, which is free, sits beside the boat's other
  keys (V camera, B bail, T anchor) under the left hand, and is rebindable
  in the bindings table as `vehicles:KeyC`, "the boat".
- **C while looking at a boat on foot**, within reach: the same aim that
  boards it with F (`interact`), so a moored boat's hold can be loaded
  from the bank. The prompt over the boat gains a line, "C  boat".
- The craft's panel (`hud.rs`) gains the row "C  the boat".
- The pack keeps its HOLD panel as it is (built, and played): the boat's
  menu shows the same hold with the same rule (`pack::click_with_hold`,
  `hold::send`, `vehicles::hold::store`). One rule, two views.

#### 10.3 What it holds

A panel in the stone-and-carved-wood kit (the dressed-stone frame, the oak
lintel for the title, the pack's 24 px slots, the kit's slider and check),
in this order:

1. **Name and condition**, on the lintel: the kind's name ("RAFT") and
   "Dry" or "Water aboard" from `bilge_kg` (with B to bail as today).
   Naming a boat, and damage, are not added (recommendation taken, ask only
   with screenshots): the game tracks neither, and neither is asked for.
2. **Hold** and **Pack**: the boat's ten slots above the player's slots,
   with the pack's own clicks (click to pick up and place, shift-click to
   send a stack across; on a phone, a tap sends).
3. **Anchor**: its state ("Stowed", "Setting", "Holding", "Weighing"),
   **Drop** or **Weigh** (T does the same without the menu), and **Rope**:
   "To the bed" by default; **+** and **-** step it a metre, a slider sets
   it from the bed up to the rode's maximum; **To the bed** puts it back.
   **Out** shows how much rode is out now. Set while the anchor is
   stowed, it is used on the next drop: the rode stops at the bed, then
   veers to the set length.
4. **Lantern**: a check, on or out. On, it is lit from dusk to dawn (9.8);
   out, it is dark all night and adds no light.

Plain labels only (CLAUDE.md, "Game text is plain" and "No notes in the
game"): no explanations of what a setting does, no costs, no "default".

#### 10.4 Per kind

| Kind | Hold | Anchor | Lantern | Its own |
| --- | --- | --- | --- | --- |
| Raft (game) | 10 slots (today's) | one, rope control | yes (9.8) | |
| Loon (game) | none (as today) | one, rope control | no | |
| Inner tube (game) | none | none | none | no menu: it carries nothing and has nothing to set; C does nothing aboard |
| Skiff | 10 slots | one, rope control | the anchor light | |
| Tug | 10 slots | one, rope control | yes | **Tow lines**: each line's length (the same stepper, up to its rope's length) and **Cast off** / **Make fast** (within reach) |
| Net boat | 10 slots; the catch lands in it | one, rope control | yes | **Net**: **Cast net** / **Haul in**, and the fish in the bag |

Recommendation taken (ask only with screenshots): one hold size for every
kind that has one, the game's ten slots, so the `hold` record keeps its one
shape; the tube has no menu at all.

**The rode's maximum** is a tunable per kind in `assets/config/vehicles.ron`
(`anchor: (rode_max_m: 40.0)`, metres): 40 m for the raft and the Loon, so
the 35 m depth limit still leaves scope. The mockup uses 30 m on its 2 m
river.

#### 10.5 What saves

On the durable path, the same record as the craft (`VehicleRecord`, written
by `save_vehicles` the moment `Fleet.dirty` is set, as dropping the anchor
is today), two optional fields written only when set, so every existing
record reads and writes as it was (the `berth` and `made` pattern):

- `rope_m: Option<f64>`: the anchor's rope setting, per craft (one anchor,
  so one value). Absent is "to the bed". The rode actually out stays in
  `mooring.length`.
- `lantern_out: bool` (skipped when false): the lantern put out.

The hold is already saved (`hold` record, one line with the slots). An old
save opens with every boat's rope "to the bed" and its lantern on, and an
anchored boat keeps the rode length it was saved with.

#### 10.6 In the mockup (version 6)

`docs/mockups/river-craft/boatmenu.js`: **Boat** (C) on the Anchor, Raft
and Rowing tabs opens the panel over the river; C, Esc or its X close it.
On the Anchor tab the one anchor drops and weighs on T or the menu's
**Drop** / **Weigh**; the menu's rope control replaces Pay out and Shorten,
and the boat visibly drops back further when it is lengthened and is pulled
up when it is shortened. The hold shows a few things (rope, bait, grilled
fish on the skiff; logs, planks and rope on the raft) beside the first ten
pack slots, and a tap sends a stack across. The Lantern check puts the
skiff's anchor light or the raft's lantern out. At phone width the panel
fills the river view and scrolls. The Tug, Log drive, Nets and Tube tabs
have no menu yet.

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

**Version 4 (owner, 2026-11-19: "no light for tube guy, fix arms/body").**
Version 3's rider stood stiffly in the ring with his arms straight out, and
a lantern on a stick was tied to the ring. Now:

- **No lantern.** At night the tube is lit only by the bank lamps and the
  moon (9.8).
- **Sitting back.** Bottom in the hole, back on the far side of the ring
  (leaning back 40 degrees), legs over the front rim with the knees bent and
  the feet in the water; the arms rest on the rim at the sides.
- **Paddling.** Each hand goes round an arc beside the ring: in at the
  front, down into the water, pulled back, out behind, back in low over the
  rim; the rider sits up to 20 degrees to reach. W or S: both hands
  together, ahead or back. A or D: the outer hand pulls ahead while the inner
  one backs, half a stroke apart. A hand never goes further than the arm
  reaches, and the elbow is solved out and down (up while the hand is in the
  water), so no frame is a T-pose. The same blocks, sizes and colours as the
  raft's rower, lit like the terrain.

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

## 11. The port, part one (owner, 2026-11-20: "approve sand and mountains, and rivers")

The owner approved M34 for the game on 2026-11-19 (version 6) and asked for
it to be built on 2026-11-20. The order taken is the owner's most recent
asks first: rowing with two oars (9), the one anchor (4), the boat's menu
(10), the raft's lantern (9.8); then the tube rider's pose; the tug, the log
drive and the nets after those. Each is built with its tests and captures
and pushed before the next.

### 11.1 Rowing (design 9) in the game

- `pbd_core::vehicle::oars`: `RowingSpec` (`oars`, `stroke`, `rower`, the
  tunables of 9.7), `oar_pose` (the schedule, one function read by the force
  and the drawing), `oar_geom`, the elbow solve, and `RowingState` (the one
  stroke clock, each oar's rest and pose, the catch and release counts the
  drawing reads). `LoonSpec` gains `rowing: Option<RowingSpec>`, `Some` only
  for the raft; a kind with it is rowed and never takes the Loon's stroke.
  The Loon's and the tube's stroke are untouched.
- **The frame.** 9.7 wrote the RON in the mockup's frame (+x the bow). The
  game's spec files are all in the craft's own frame (+x right, -z the bow),
  so `vehicles.ron` writes the same numbers there: the pin `(1.0, 0.62,
  0.0)`, the seat `(0.0, 0.62, 0.52)`. Recommendation taken (ask only with
  screenshots): one frame for every number in the file.
- **The water at the pin.** The mockup's raft floats with its waterline at
  y 0. The game's bobs and heels, so the blade's depth and clearance are
  taken from the water's height at each pin in the raft's frame each
  substep (`Context::above_sea`), not from y 0: the blade buries
  `blade_depth_m` under the water it is in.
- **The force** is the mockup's: the blade's drag through the water
  (`-1/2 rho Cd A s |v| v`, horizontal), `s` its share under the water
  times its share over the bed, the blade's own velocity from the schedule.
  Measured in the core test sea: W for 30 s makes 25.6 m and 1.03 m/s (the
  pole made 6+ m in the same test); one oar alone turns the raft 107
  degrees in 8 s; on a 0.16 m bed the blades rest on it and the pull falls
  from 583 N to 243 N and the raft makes 0.1 m in 10 s against 5.9 m in
  deep water. The mockup's 0.5 m/s was on its own simpler hull drag; the
  game's raft keeps its hull and resistance as they were, so it rows faster
  through the water. Not judged yet in real time (no GPU here).
- **The seat.** The rower sits on the frame's bench amidships, so the
  raft's `paddler` mass moves to `(0.0, 0.85, 0.52)` and its seat eye to
  `(0.0, 1.36, 0.42)`; the seat camera rides the body's lean
  (`Craft::eye`, `head_follow`, 9.6), never the mouse look.
- **The drawing** (`vehicles/draw.rs`, `rowing_frame`): the beam, two posts
  with swivel oarlocks, the bench, the two oars (loom, handle, grip, a
  feathering blade), the rower facing the bow (cap peak and nose ahead,
  hair at the back, feet braced on the beam) leaning from the hip, his arms
  two blocks a side from the shoulders to the grips through the solved
  elbow, a block hand round each handle. The first-person hands
  (`vehicles/oar_hand.rs`) are M36's approved hand built from the axe's
  grip as M34's are, closed on each handle and turned toward the elbow,
  hung on the raft rather than the camera so they are lit as the raft is
  (`LitByField`, `LitLikeTerrain`). Each part is in one view only: the
  rower's body and arms in the chase view, the hands in the seat view.
- **The tool in hand.** While the oars row, both hands are on the handles,
  so the held tool (`held.rs`) is put down in the seat view; with the oars
  at rest a tool in hand is shown as before (fishing from the raft,
  `small-boats` design 6) and the oar hands are hidden. Recommendation taken
  (ask only with screenshots).
- **Keys** (`controls.rs`, `RAFT`): W/S "row forward or back", A/D "turn",
  Q/E "row the left or right oar". The panel says "Oar on the bed" while a
  blade rests on a shallow bed (9.2).
- **Not built yet:** the splash, wash, drips and swirls (9.5). They are
  drawn only and nothing reads them; the catch and release counts they
  would key on are in `OarState`.
- **Saves:** nothing new. The stroke is derived; the frame and oars are
  part of the raft.

### 11.2 The anchor (design 4) in the game

- `pbd_core::vehicle::anchor`: `AnchorSpec` (`vehicles.ron`, `anchor:` on
  the Loon and the raft; the tube has none, 10.4) and `Anchor`, its work:
  dropping at `drop_mps`, the rode paying out only to the bed (`None` rope)
  or veered to the boat's rope setting, setting in `set_s`, sliding toward
  the boat under a pull over `slide_n` until set, holding once set, hauled
  in at `haul_mps` on Weigh and lifted aboard. One anchor per boat.
- **The bed.** The game does not yet tell the anchor what the bed is made
  of; every bed sets in 1.3 s (the mockup's rock) and slides over 900 N.
  Recommendation taken (ask only with screenshots): the bed's three rates
  wait until the bed's material is read at the anchor.
- **The rode is the mooring line.** While the anchor is on the bed,
  `Craft::mooring` is the line to it (the existing pull, `Craft::moor`), so
  a save keeps exactly what it kept before: the point, the rode, anchored.
  Falling and coming aboard are not lines. The anchor's work is not saved:
  a save taken while it falls keeps the boat unanchored until it lands, and
  the landing is written at once (`Fleet.dirty` on every change of phase).
- **Old saves.** An anchored record reads back as an anchor set and holding
  on the rode it was saved with (design 4: "a rode already saved keeps its
  length"). A harbour boat placed at anchor (`place.rs`, `rode`) is the
  same. `place::rode` (3 x depth + 2 m) is kept for those berths only; a
  new drop stops at the bed.
- **T** drops it where the water is at most 35 m deep (as before), or weighs
  it when it is out (and stops hauling if pressed again while weighing); a
  boat tied to a bollard is cast off as before. The panel reads "at anchor",
  or the anchor's state ("dropping", "setting", "weighing"), and "T  weigh
  the anchor" or "T  drop the anchor".
- **Drawn:** the anchor (M34's shank, stock, crown and flukes) on the bed
  or falling, and its rode as a straight line from the bow; stowed on the
  raft's deck by the bow. The sagging, current-dragged rope of design 3 is
  not built.

### 11.3 The boat's menu (design 10) in the game

- **C opens the pack with the boat's panel beside its HOLD**
  (`desktop/boat.rs`), aboard a boat with an anchor or on foot within its
  boarding reach. The HOLD is the pack's own panel and clicks, so the hold
  has one rule and one view; the boat's panel adds the name and condition,
  the anchor and the lantern. C or Esc closes it, as I or Esc close the
  pack. Recommendation taken (ask only with screenshots): the pack screen
  with a panel added, not a second screen with a second copy of the hold.
  On foot, C finds the nearest boat in reach rather than the aimed one.
- **The panel:** the kind's name and "Dry" or "Water aboard"; Anchor: its
  state and Drop / Weigh / Stop; Rope: `-`, the setting ("To the bed" or
  metres), `+`, and To the bed, with "Out" the rode out now; Lantern: a
  check, On or Out, on a boat with a lantern. Plain labels only.
- **Saved** (`VehicleRecord`): `rope_m: Option<f64>` and `lantern_out:
  bool`, each written only when set (the `berth` and `made` pattern), so
  every record before them reads and writes as it was, and opens with its
  rope "to the bed" and its lantern on. A change in the menu sets
  `Fleet.dirty`, so it is on the durable path the update it happens.
- The C row is in the bindings table (`vehicles`, "the boat") and on the
  craft's panel.

### 11.4 The raft's lantern (design 9.8) in the game

- `LoonSpec` gains `lamp: Option<CraftLampSpec>` (`at`, the bow post's
  lantern), `Some` for the raft only. Its light is not a second set of
  numbers: it is the placed post lantern's, from the one table
  (`pbd_core::light::carried_light(Material::LanternPost)`), its level,
  tint and dusk switch. Recommendation taken (ask only with screenshots):
  the `range_m`, `colour` and `intensity` tunables of 9.8 are not added,
  so the raft's lamp cannot drift from the town lantern's.
- **How it joins the field:** the carried-light list (`carried_light`,
  `glowing-mushrooms` part B) is the "short list of moving lamps uploaded
  each frame and summed into the same lamp term" 9.8 asked for: the
  terrain, the water and every field-lit mesh already add it per pixel
  with the bake's falloff. `vehicles::lamps::gather` lists every lit craft
  lantern; `carried_light::gather` puts the player's own light first and
  then the craft lanterns nearest the eye, up to the list's four. No baked
  shadow or bounce (stated limitation, as for a carried torch).
- The drawn flame and the light both go out with the lantern check.

### 11.5 The tube rider (the inner tube, version 4) in the game

The game's tube rider lay back and kicked (2026-11-16). M34's version 4,
approved with the page: no lantern, the rider sits back in the ring with
the legs over its front and the feet in the water, and paddles with the
hands. The tube's physics stays the Loon's stroke (a stroke on one side at
a time; W alternates); the drawing reads it: the hand on the stroke's side
goes in at the front, pulls back through the water through the power
phase and comes back low over the rim through the recovery (backing the
other way round), the other hand resting on the rim; the body sits up 20
degrees from its 40 to reach while a hand paddles; each arm is two bones
through an elbow solved out and down, up while the hand is in the water.
The feet no longer kick.

### 11.6 The rower's look

In the seat the hands are on the handles, half a metre under the eye and
in front of the chest, so a level look does not see them. Boarding a raft
in the seat starts the look 0.55 rad (31 degrees) down along the oars to
the bow (M34's Rowing tab starts at 0.72); the mouse turns it anywhere as
before. Recommendation taken (ask only with screenshots).

### 11.7 Open after this part

- The splash, wash, drips and swirls (9.5).
- The bed's material deciding how fast an anchor sets (4).
- The current derived from the cells (1, 2), lines between craft (3), the
  tug and barges, logs, booms and bundles (5), nets (6).
- The frame cost: not measured (no GPU in this session).
