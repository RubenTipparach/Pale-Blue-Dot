# Design: small boats

Every decision below is a recommendation taken (ask only with screenshots):
the captures go to the owner with the change, and any look they reject is
changed after.

## 1. The finding

- Inner tubes are not in the game.
- No craft is made: the Tern and the Loon are placed at a new world's start,
  and the rowboat, canoe, sail boats and cog belong to harbours.
- `recipes.ron` has no boat.
- Fishing runs only on foot.

## 2. Three boats, all on the Loon's stroke

The Loon's model (`pbd_core::vehicle::loon`) is the whole of a paddled boat:
- a hull of cells;
- a lateral foil and a skeg;
- a stroke that is drag on a blade;
- a bilge;
- three seabed contacts;
- a seat.

The rowboat already runs on it with a `LoonSpec` of its own. The tube and the
raft do the same, so neither needs new physics:

| Craft | Kind key | Hull, m (L x B x D, sheer) | Paddler | Stroke |
| --- | --- | --- | --- | --- |
| Inner tube | `tube` | 1.2 x 1.2 x 0.35, 0.22 | sits in the ring | hands: short, small blade |
| Raft | `raft` | 3.0 x 2.0 x 0.45, 0.30 | sits mid-raft | a paddle, longer reach |
| Rowboat | `rowboat` (existing) | 4.2 x 1.45 x 0.6, 0.35 | as today | as today |

**The tube** is round: bow and stern powers of 2 make an ellipse of a hull as
wide as it is long. Its `deck` is set, so the ring takes no water in. It turns
easily and is slow. It is a float, not a vessel.

**The raft** is boxy: high section and end powers make a flat-bottomed slab.
Its deck is set, so it does not swamp. It is steadier and faster than the
tube and slower than the rowboat.

Specs are `LoonSpec::tube()` and `LoonSpec::raft()`, written out to
`vehicles.ron` as `tube:` and `raft:`. `VehicleSpecs` is `#[serde(default)]`,
so a file without them still reads.

## 2a. The models are M34's (owner, 2026-11-16: "you should use the same models from the mockup")

The tube and the raft are drawn as M34 River Craft draws them
(`docs/mockups/river-craft/scenes.js` on `claude/env-v2`, `tubeModel` and
`raftModel`), at its sizes:

- **The inner tube.**
  - A felted-wool ring sealed with pine pitch: 0.52 m to the middle of the
    tube, 0.22 m thick.
  - A darker pitched seam over a quarter of it.
  - A lantern on a short pole tied to the ring.
  - Its paddler sits low in the ring, leaning back, arms out over it.
- **The raft.**
  - Five six-sided logs, 4.4 m long and 0.25 m round, bark round and rings
    on the ends.
  - Three planks across them, and a crate.
  - A lantern on a post at the bow's corner.
  - A standing poler with a 4.6 m pole, held upright at rest. A stroke
    works the pole where the paddle stroke puts it.
- **The people** are M34's blocky villager: legs, body, head and hat.

Colours are M34's tiles under its tints, taken down about 30% so they read
as the mockup's under the game's brighter noon. The hulls in section 2 take
M34's sizes:

| Craft | Hull, m | With paddler | Waterline |
| --- | --- | --- | --- |
| Tube | 1.48 x 1.48 x 0.44 | 115 kg | the ring about half under |
| Raft | 4.4 x 2.4 x 0.5 | 2.08 t | the logs a little over half under, as the mockup draws them |

The raft is heavy because a raft of green logs is.

`docs/screenshots/small-boats/{raft,tube}-beside-m34.jpg` show each beside
the mockup's.

## 2a-1. The tube's rider, and the boat panel (owner, 2026-11-16)

**The rider.** The owner, on the first port: "shouldnt the dude be laid
back, also his arms are in a weird spot, maybe need 2 joints for the arms".
The rider now lies back in the ring:
- seat in the water through the hole, back against the ring's far side;
- knees over its near side, shins hanging over it;
- each arm in two parts: the upper arm out to the ring, the forearm resting
  along it to the hand.

The parts are blocks laid joint to joint (`Builder::limb`). The seat's eye
moves low and aft to the head (0.5 m up, 0.58 m aft). The rider's mass is
over the hips (0.06 m aft), and the tube leans back about 3.5 degrees with
them in it.

**The panel.** The owner, on the boat panel: "please just show simple raft
controls, make it user friendly. I dont know if this text is just debug or
part of the game". Every boat's panel was instruments, water slip, ground
drift, strokes a minute and sailing advice, over one long line of keys. It
is now:
- the boat's name, with "at anchor" when it is;
- its keys one to a line, in plain words (`controls.rs`: "pole forward or
  back", "turn", "steer with the pole"; the Tern's "steer", "pull the sail in
  or let it out", "lean out left or right");
- the anchor key, named for what it will do now;
- step off and camera;
- "Water aboard: bail with B" only when there is water to bail; the bail key
  shows only then too.

Every label is read off the bindings, so a rebind shows. The instruments
moved behind F3 with the frame graph (`controls::DebugOverlay`), for whoever
tunes a hull.

## 2a-2. The tube kicks; stone key glyphs (owner, 2026-11-16)

**The tube kicks.** The owner: "dont need the stick thing in the tube, the
guy can just kick his feet around to turn and accelerate".
- M34's lantern pole is off the tube.
- The tube's stroke is the rider's feet: a foot either side of the middle
  (0.3 m out), under the water ahead of the ring (0.95 m forward, 0.2 m
  down), each kick a 0.45 m sweep aft.
- W/S kick forward or back, and A/D turn by kicking on one side. The tube
  has no Q/E row.
- The shins swing from the knee with each kick.
- Tests: `a_tube_turns_on_its_kicks` (131 degrees in 10 s), and it still
  makes 0.7 m/s ahead.

**Stone key glyphs.** The owner: "can you also make some stone button glyphs?
to go with the controls prompt". A key is drawn as the UI kit's own stone
button (`button-idle`, the approved stone-and-wood art), 9-sliced to the
key's name with the name carved in as a stone plaque's label is
(`ui_kit::key_cap`).
- **The interact prompt** draws its key this way (it was the oak tooltip).
- **The boat panel** has no background of its own (owner: "probably leave
  off the background here, and add drop shadow or outline for text"):
  - the boat's name in the title colour;
  - then a row a control, each key its own stone glyph, followed by what it
    does;
  - every word outlined in ink, so it reads over water and sand alike.
  - no row for moving on W, A, S and D (owner: "we probably dont need WASD,
    its obvious enough"); a row the player has rebound to other keys shows.
- The F3 instruments stay plain text, a developer's.

## 2b. Rivers and lakes float a boat (found building it)

Craft floated only on the sea: `Surroundings.sea_radius` was the drawn sea
everywhere. A boat put on a river over the sea's level sat on the riverbed,
which is not "building on the river stuff".

- **`vehicles::fresh_water`** reads the column tier. Where a craft stands in
  water whose top is more than 0.25 m over the sea's own cells' top, the
  craft floats on that top:
  - the sea's cells fill every layer under the sea layer, so their top is
    that layer's bottom;
  - the sea is drawn `depth_offset_m` (0.5 m) under that, and craft at sea
    keep floating on the drawn sheet as before;
  - the river mesher draws a river at its own column top, so a boat on it
    floats where it is drawn.
- **On fresh water:**
  - there is no tide;
  - the sea's waves are shoaled to 0.15 m of "depth", so they read as
    ripples;
  - the current is the surface's slope, read 1.5 m either side, times the
    water mesher's own pan (1.6 m/s at a slope of 0.12), capped at 2 m/s.

  So a boat drifts downstream as the drawn water streams.
- **Anchoring** (T) measures depth from that surface too.
- **The first cut was wrong.** It compared the column's top with the drawn
  sea's radius, which is 0.5 m lower. Every sea cell read as fresh, and boats
  at sea floated half a metre too deep. A trace, `PBD_BOAT_TRACE`, a
  measurement instrument that logs the aboard craft's water each tick, showed
  it. The rule now compares with the sea cells' top.

## 2c. Boats sank at high tide (a bug on `main`, found building it)

The captures showed the Loon at its new-world berth swamped, with 700 kg
aboard and heeled 60 degrees. The raft showed its deck 0.5 m under the
water. The trace put the surface about 0.85 m over the mean.

- **The cause.** `LocalSea::reach()`, "the highest the surface can stand
  above the mean", summed the waves only. The tide (`ocean-environments` 5.1)
  was added later as an offset on the same surface. The hull's float skips
  every cell more than `reach()` over the mean as "dry whatever the sea
  does". At high water, every cell between the waves' reach and the tide
  gave no lift, and each boat sat low enough for the sea to come over its
  gunwale.
- **The fix.** `reach()` adds the tide's highest, `tide_high_m(range)`.
- **The test.** `every_boat_floats_on_the_tide` sets the moon overhead with a
  1.2 m range (0.8 m over the mean). Without the fix the Loon swamps
  (700 kg, freeboard -0.58 m). With it, the Loon, the rowboat, the tube and
  the raft each ride the tide, gunwale clear and bilge dry.
- **New craft go on the water as it stands.** A boat made on the mean under
  a high tide still had water over its gunwale before it could rise.
  `place::on_the_tide` lifts a craft being made to the tide's height there.
  It covers the new world's two boats, a harbour's boats (not its cog, which
  lies on its swing), a launch and a capture's launch. A craft restored from
  a save is left where it was.

This changes what a player sees: boats no longer sink at high water. It is a
bug fix, not a look change.

## 3. Recipes

| Code | Name | Station | Takes | Gives |
| --- | --- | --- | --- | --- |
| `inner-tube` | INNER TUBE | workbench | wool 4, rope 2 | `inner-tube` x1 |
| `raft` | RAFT | workbench | log 6, rope 4 | `raft` x1 |
| `rowboat` | ROWBOAT | workbench | planks 16, rope 2 | `rowboat` x1 |

- **The tube is M34's**: felted wool made into a ring and sealed with pitch
  (the world has no pitch yet, so the rope's tar stands for it), made at the
  workbench. Its wool comes from the herd. A first boat that needs no
  herding is the raft. It was planned by hand, but the pack's Craft list
  shows six rows and the hands already make six things (cooking adds the
  campfire). A seventh row would have pushed the campfire off the list.
  The world has no rubber, and the owner calls it an inner tube, so it is
  named one.
- **The raft is logs**, so an axe and a few trees.
- **The rowboat is planks.**
- **Each boat is a good, one to a slot**, appended to the goods table with a
  16 px icon. Icons are UI art, so they keep their 1 px outline.

## 4. Launching and taking up

- **Launching.** With a boat good selected on the hotbar, the place button
  (right) aimed at water within 8 m puts that craft on the water's surface
  there, its bow along the look. Aimed at land, nothing happens and the HUD
  says so.
  - The craft is a new record with the next ID, marked `made: true`.
  - The fleet's snapshot is written first, then the slots without the good.
  - A crash between the two can leave both a boat and the good; it never
    leaves neither. "Nothing a player made is dropped."
- **Taking up.** On foot, the dig button held for 1 s on a craft you made
  (`made`), within reach, takes it back into the slots. It must have nobody
  aboard and an empty hold.
  - Harbour boats and the world's first two are not taken up: they are not
    yours to carry off.
  - The slots are written first, then the fleet without it. The same rule
    holds: a crash keeps the boat.
  - With the rod in hand the dig button casts, so the rod does not take up.

## 5. The raft's hold

- **The hold** is ten slots, stored as a `hold` record whose id is the raft's
  craft id (schema 1). Its body is the ten stacks in the pack line's own
  stack text.
- **Opening it.** The pack opens aboard the four paddled boats, as well as on
  foot. With a raft aboard, or the nearest raft within 3.5 m on foot, the
  pack shows a HOLD row of ten above it.
- **Moving goods.** Clicks move stacks between the hold and the slots exactly
  as between pack and hotbar: pick up, put down, merge, swap, and right-click
  for half. Shift-click sends a stack between the hold and the pack.
- **Saving.** Each move is one line, `WorldSave::store_with_hands`: the hold
  record and the slots together. A move cannot be half-saved.
- **Taking a raft up** is refused while its hold holds anything.

## 6. Fishing afloat

The line's rules are the core's and are unchanged. What changes is who the
angler is:

- **On foot**: as today, the walker's feet and the walking camera.
- **Aboard the tube, the raft, the Loon or the rowboat**: the craft's seat.
  - The rod's tip is placed from the seat's eye with the vehicle camera's
    look, in both the seat view and the chase view.
  - The feet are the seat's exit point.
  - The pointer is the vehicle view's capture.
- **The tool picker (G) opens aboard those four**, so the rod can be taken
  up there. Paddling keys are untouched, so a paddler can paddle with the
  line out.
- **The rod is drawn in the seat view**, the held models moved under the
  vehicle camera while aboard. In the chase view it is not drawn.
- **Boarding or leaving with a line out** winds it in, as putting the rod
  away does.
- **Where the catch goes.** A catch lands in the slots and the journal as on
  foot. The hotbar is drawn aboard those four, so the catch is seen.
- **The Tern and the Cog do not fish from the helm.** The cog's deck is on
  foot and fishes already.

## 7. Saves

- **Records.**
  - `VehicleRecord.made` is `#[serde(default, skip_serializing_if = ...)]`,
    so every existing record writes as before.
  - Kinds `tube` and `raft` are new strings.
  - The `hold` record kind is new.
- **Goods.** `inner-tube`, `raft` and `rowboat` are appended to the goods
  table.
- **What an older build does.** It reads a newer save's `tube` record as an
  unknown kind, the same way any newer kind is met. It keeps `hold` records as
  text, as it keeps every record it does not know.

## 8. Captures and the late pipelines

The `--aboard` captures log 28 `LATE_PIPELINE` lines after `READY` (cloud,
rain, water, tonemapping and the mesh pipelines). They come from the
capture's switch to the vehicle camera, which compiles the view's pipelines
again. `--aboard loon`, a craft this change does not touch, logs the same 28
on the same build. That belongs to the loading screen's prewarm, not to this
change; it is noted here so it is not mistaken for one.

## 9. Not measured here

This runs in a Claude Code cloud container with no GPU, so the frame cost is
not measured. The change adds at most a few craft, each the Loon's step, and a
row of UI.

## 2d. A river's own current, a raft that floats, a pole (owner, 2026-11-18)

The owner: "float an inner tube down a river ... see if its fast enough,
otherwise we can tweak it", then "please ensure the raft is boyant first",
then "get me a 1 minute video of the raft guy pushing the raft up and down
the river".

- **A river's own current.** 2b's current was the surface's slope alone, and
  a river's reaches are level between their steps: a tube sat still, or made
  0.8 m/s and stopped in a level pool. A craft in a river's channel now also
  takes 2 m/s down the drainage map's course (`RIVER_FLOW_MPS`), and a pull
  of up to 0.8 m/s toward the river's line, so meanders carry it round their
  bends rather than into a bank. The slope's push stays on top; the cap is
  3.5 m/s. Measured: a tube floated 44-55 m in 30 s with no input, never
  stopping.
- **Floating on the drawn water** (`PlanetContact::drawn_water`). A flowing
  top is drawn through its corners, up to a layer's share over the column's
  level; craft floated on the level, so a raft could sit under the water seen
  over it.
- **The raft is 1,200 kg, not 2,000.** That's nine 4.4 m logs about 0.3 m
  thick, green. At 2,000 kg its deck rode 7 cm clear, awash in the ripples;
  now 16 cm, steady (`PBD_BOAT_TRACE`). The core vehicle tests pass.
- **The raft's stroke is a pole** (`PaddleSpec::pole`). It pushes on the bed,
  measured against the ground, where a paddle pushes on the water: a paddle
  can't beat a current, and the raft poled flat out only slowed its drift
  from 2 to 1.1 m/s. Its push area is 0.7 m². Now it poles up the river
  against 2 m/s at 0.2-0.65 m/s.
- **Instruments.**
  - `--aboard tube|raft` with `--at` launches on the river there, at the
    water's drawn surface.
  - `TUBE` lines log a scripted craft's drift each second.
  - `--pole-run` poles a raft up the river for 55 s from its launch, then
    lets the current carry it back down. It's launched facing upstream,
    because a 4.4 m raft turned round in a narrow channel swings broadside
    and wedges, and poled backward it spins.
- **Videos:** `inner-tube-float-2026-11-18-450p60.mp4` and
  `raft-pole-up-and-down-2026-11-18-450p60.mp4`, Drive validation folder.
- **The pole moves as M34's does** (owner: "why his stick flyaway?", "why is
  the game implmenetation different on how the pole moves", "the mockup worked
  well"). The game drew the raft's pole at a paddle's blade, out to whichever
  side the stroke was on, 1.35 m from the rider's hands. It now hangs from his
  hand on the controls, exactly as `river-craft.html`'s raft: 0.12 forward at
  rest, 0.5 with W or S, tilted 0.35 to the side with A, D, Q or E
  (`vehicles::draw::pole`). Video: `raft-pole-up-and-down-v2-2026-11-18-450p60.mp4`.
