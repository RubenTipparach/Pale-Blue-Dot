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
| `inner-tube` | INNER TUBE | workbench | fibre 8, rope 2 | `inner-tube` x1 |
| `raft` | RAFT | workbench | log 6, rope 4 | `raft` x1 |
| `rowboat` | ROWBOAT | workbench | planks 16, rope 2 | `rowboat` x1 |

- **The tube comes first and is cheap.** It is reed fibre bound into a ring,
  made at the workbench. It was planned by hand, but the pack's Craft list
  shows six rows and the hands already make six things (cooking adds the
  campfire). A seventh row would have pushed the campfire off the list, and
  the workbench is six planks.
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
