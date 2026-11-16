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

## 3. Recipes

| Code | Name | Station | Takes | Gives |
| --- | --- | --- | --- | --- |
| `inner-tube` | INNER TUBE | hand | fibre 8, rope 2 | `inner-tube` x1 |
| `raft` | RAFT | workbench | log 6, rope 4 | `raft` x1 |
| `rowboat` | ROWBOAT | workbench | planks 16, rope 2 | `rowboat` x1 |

- **The tube comes first and is cheap.** It is reed fibre bound into a ring.
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

## 8. Not measured here

This runs in a Claude Code cloud container with no GPU, so the frame cost is
not measured. The change adds at most a few craft, each the Loon's step, and a
row of UI.
