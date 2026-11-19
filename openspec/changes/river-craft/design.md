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
- **Bow and stern.** The first anchor down veers to five times the depth.
  One dropped while the other holds is snubbed short where it lands, so the
  two together hold the craft still on its heading.
- **Weighing is the only way to move again.** It hauls the rode in to
  straight up and down and lifts the anchor aboard.

Dropped as proposals: the sand anchor that never sets and drags, the rock
anchor that skips until a fluke catches, and a short rode lifting the shank
and breaking the anchor out. Pay out and Shorten change only how far the
craft can swing. The existing anchor's depth limit and default rode stand.

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
- **Saves.** Nothing new: the lamp is part of the raft, and whether it
  burns is the clock's answer.
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
