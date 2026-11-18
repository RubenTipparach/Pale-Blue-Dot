## Context

After `retire-tern` a world makes rowboats and cogs in three places:
- `vehicles::harbour::boats_for` makes every harbour template boat of kind
  `rowboat` into a `Kind::Rowboat` (fourteen in Marenstrand's template,
  `assets/settlements/v1/coast.json`), moored at its berth;
- `vehicles::harbour::cog_for` makes the template's `cog` into a `Kind::Cog`
  at its berth, ridden on the mooring swing (`swing_moored_cogs`) and drawn
  from the town's ship piece (`pieces/cog.rs`);
- the workbench's `rowboat` recipe (16 planks, 2 rope) makes a `rowboat`
  good, launched from the hotbar as a made craft (`made: true`).

The harbour template's other boats are three `canoe`s (Loons) and six `sail`s,
whose Terns are retired. So after this change a harbour moors its three
canoes.

The Kestrel and the Tern built the machinery for a retired kind:
`Kind::RETIRED`, `KindKey::Retired`, `Fleet::retired`, the writer appending
held records unchanged, a held record's berth counted as made, and the
walker put ashore when it loads over the sea beside a held record.

## Goals / Non-Goals

**Goals:**
- No rowboat or cog in any world from this build on: none moored, made,
  drawn, sailed or walked on.
- Every existing save opens; its vehicle file keeps every rowboat and cog
  record exactly.
- Nothing a player made is dropped: a made rowboat, a rowboat good and a
  queued rowboat job all come back as their materials, exactly once.
- A player saved aboard (or on a cog's deck) loads on dry land.
- Their code removed, not hidden behind a flag.

**Non-Goals:**
- The new boats (M23's rowing launch, M34). A separate change, on a request.
- The towns' dressing boats: a beached rowboat on trestles or one lying on
  the shore is a prop, like a crate, drawn by `pieces/dressing.rs`; it is not
  a craft and stays (recommendation taken, ask only with screenshots).
- The drowned-boat finding (below): reported, not changed.

## Decisions

### 1. Old saves: the Tern's rule

`"rowboat"` and `"cog"` join `Kind::RETIRED`. A record of either is held in
`Fleet::retired` as read, appended unchanged by the writer, and `next_id`
never falls below its ID. Its `yard` field stays in layout 1 for the held cog
records. No version change. A held harbour boat's berth (`(site, n)`, the
cog's `(site, u32::MAX)`) stays in the "made" set, so its harbour does not
make another at it.

- *Recommendation taken (ask only with screenshots).* The alternatives are
  the ones `retire-kestrel` and `retire-tern` turned down: keep them sailable
  in old worlds (keeps all their code), delete the records (drops something
  from a save), or turn them into another boat (invents a rule).

### 2. A made rowboat is refunded, once

A rowboat the player made is the only one of these the player built. Its
record says so (`made: true`, `small-boats` design 4); a harbour's has a
`berth` and no `made`, so the two are told apart in every save.

On the fleet's first spawn after a load, each held retired record with
`made: true` that has no `refund` record yet is refunded:
- its materials, the rowboat's recipe as it shipped (16 planks, 2 rope:
  `station::RETIRED_RECIPES`), and every stack in its hold (`hold` record of
  its ID; only a raft could be given one, but a hold is moved if found);
- given into the slots by the pack's own `Slots::give`, all or nothing: if
  everything fits, the slots and a `refund` record for the craft's ID
  (schema 1, body: what was given) are written in ONE line
  (`WorldSave::store_with_hands`), so a crash leaves either both or neither;
- if it does not all fit, nothing changes and it is tried again every
  second while the world is open (and at every later load) until it fits.
  The materials are owed, not dropped.

The held record itself stays byte for byte; the `refund` record is what says
it was refunded, so the refund happens once whatever happens to the record.

- *Why not a dropped bundle when the pack is full.* A drop lasts 300 s of
  world time (`drops::LIFE_S`) and then is gone, and the craft may lie
  kilometres from where the player loads: a refund left there could be lost.
  Waiting in the save cannot lose it. Recommendation taken (ask only with
  screenshots).

### 3. A rowboat good, and a queued rowboat job

- **A rowboat good** (made and never launched, or taken back up) is an item
  in the pack. The good's row stays in `goods.rs`, so a saved stack still
  reads (an unknown code reads as an empty slot, which would lose it). It
  cannot be launched (`Kind::of_good_code` knows no rowboat). Whenever the
  player's slots hold one, it is swapped for its recipe's materials, all or
  nothing, and the slots written (`record_pack`); the good itself is the
  token, so it is swapped once. One in a raft's hold is swapped when it is
  taken into the pack.
- **A queued job** for the rowboat at a workbench names a recipe the book no
  longer has. Today such a job is kept and never collected, which would stop
  the bench's queue behind it for good. A job whose recipe is in
  `RETIRED_RECIPES` takes no time and collects as its inputs, by the
  collect's own all-or-nothing rule: the player gets the planks and rope back
  from the bench.

### 4. A player saved aboard is put ashore

The Tern's rule (`place::at_a_retired_craft`) moves a walker loaded over the
sea within 6 m of a held record to the nearest dry land. A cog is 15 m long
and its deck is walked on, so a player who saved on its deck may stand up to
7.5 m from its reference point: for a held cog the reach is 9 m. Everything
else about the rule is unchanged.

### 5. Harbours moor canoes only

`harbour::kind_of` makes a craft only for a `canoe` (a Loon). `rowboat` and
`sail` make none; `cog_for`, the swing, `Berthed`, making fast at the berth
and the COG berth number's uses go. The town no longer cuts the cog's ship
or its gangplank: with no ship, a gangplank up to nothing. The template's
`cog` entry and its `gangplank` are still read, and `sea::over_water` still
lays the cells under them exactly as before (the cog's plan points move into
`sea.rs` unchanged), so a harbour's ground is cut cell for cell as it was.
Removing the two pieces renumbers only the dressing pieces after them, which
carry no doors, so no saved door state moves.

### 6. The code comes out

- Core: `Kind::Rowboat`, `Kind::Cog`, `CraftState::Cog`, `Telemetry::Cog`,
  `Hulls::{cog, rowboat}`, `VehicleSpecs::{cog, rowboat}`,
  `LoonSpec::rowboat`, `vehicle/cog.rs`, `CogSpec`, `SquareSailSpec`,
  `Input::sheet`; `settlement::pieces::cog` and `Built::{cog, cog_meshes,
  cog_bow}`; the rowboat recipe.
- App: the draw arms (the rowboat's boards and the cog's ship, rig, tiller,
  lantern and carried light), the cog's HUD panel, the view index, the
  ROWBOAT and COG key groups, `--aboard rowboat|cog`, `--sail`, `--sail-for`,
  `--cog-swing`; `decks.rs` and the walker's riding of a moving deck
  (`Decks`, `CraftDecks`, `Piece::Craft`, `ride_decks`, `GroundState::drift`
  and `on_deck`), which only the cog's deck used; the perf suite's `sail`
  scenario.
- The foil model and the frame types (`pbd_core::frame`) stay: the Loon,
  tube and raft use the first, the app's flight the second.
- `vehicles.ron` and `inputs.json` are regenerated from the specs, as
  `retire-tern` did.

### 7. Saved rebinds

A rebind of a ROWBOAT or COG key (`rowboat:KeyW`, `cog:KeyA`) in a settings
file is ignored on load and kept in the file, by the rule `retire-kestrel`
decision 3 made for unknown slots. A test pins it for both.

### 8. Tests

- Rowboat- and cog-only tests go with their requirements (the rowboat
  floats and rows; the cog sails, rides its swing, makes fast, is walked
  on, its lantern; the moving-deck walker tests).
- Tests that used the rowboat or the cog as "a harbour boat" switch to the
  canoe.
- New: an old build's vehicle file with harbour rowboats, a cog and a made
  rowboat opens with its Loons, every rowboat and cog record written back
  byte for byte; the made rowboat refunds 16 planks and 2 rope once, and not
  again after a reload; one whose pack is full waits and refunds when room
  is made; a hold moves with it; a rowboat good in the pack becomes its
  materials; a queued rowboat job collects as its materials; a fresh world's
  harbour moors canoes only; a walker loaded on a held cog's deck is put on
  land.

## The drowned boats (owner, on `retire-tern`: "idk why but that boat is capsized lol")

`retire-tern` found that the tipped boats are swamped ones: water that comes
over the rim of an open boat at its berth (the tide, before `small-boats`
1.4; rain into the open hull) never leaves it unattended, because the Loon
and the rowboat drain nothing (`drain_kgps: 0.0`) and only a paddler's
bailing empties them. With the rowboat gone, the open boat left is the Loon:
a new world's own Loon, and each harbour's three canoes. Whether they swamp
at their berths is measured on this change's runs and reported in the
tasks (3.4); the behaviour is not changed here.

## Risks / Trade-offs

- **The held records grow with the world** (fourteen rowboats and a cog per
  harbour met): written back on every vehicle-file rewrite, as the stowed
  ones are. Negligible.
- **A refund waits while the pack is full.** The player sees the materials
  arrive only once there is room. Nothing is lost; the log says what is owed.
- **The walker loses moving decks.** Nothing else moves under a walker
  today. M23's ships will need it again, designed for their own hulls.
- **The mockups that show the rowboat or the cog are not redrawn.** The
  tracker's rows say they are retired; the pages are not republished.
