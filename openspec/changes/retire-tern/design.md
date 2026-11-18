## Context

The Tern is the sailing dinghy: a 4.6 m off-white lofted hull with a salmon
deck strip, a mast, a boom and a white triangular sail, a lifting keel and a
tiller (`pbd-core/src/vehicle/tern.rs`, `vehicles/draw.rs`). A world makes
Terns in two places:
- `vehicles::place::spawn_fleet` puts one at anchor in 3 m of water near the
  player when a world has no fleet yet;
- `vehicles::harbour::boats_for` makes every harbour template boat of kind
  `sail` into a Tern (`harbour::kind_of`), moored at its berth. A world from
  the current build holds 31 of them (six harbours).

Every craft is saved in the world's vehicle file (`VehicleFile`, layout 1), the
Tern with `kind: "tern"` and its `sheet`. `retire-kestrel` (2026-11-14) built
the machinery for a retired kind: `Kind::RETIRED`, `KindKey::Retired`,
`Fleet::retired`, and the writer that appends held records unchanged.

The owner asked for "these boats" to be deleted, in favour of the boats being
designed in M23 and M34.

## Which boat it is

Identified from captures of every craft the world spawns, on `origin/main` at
`2de3c03a`, fast build, the M4 Max Mac's GPU
(`docs/screenshots/retire-tern/`):

| Craft | Hull as drawn | Matches "plain white hull"? |
| --- | --- | --- |
| Tern | off-white loft, salmon deck strip, white sail | yes: on its side only the white topsides and bottom show |
| Loon | dark driftwood planks | no |
| Rowboat | brown boards, thwarts, oars | no |
| Cog | timber ship, square sail | no, and much larger |
| Inner tube, raft | felt ring; logs | no, and not placed by the world |

The Tern is the only white hull, so it is the one retired. The others stay.

## Goals / Non-Goals

**Goals:**
- No Tern in any world from this build on: none placed, none moored, none
  drawn or sailed.
- Every existing save still opens, and its vehicle file keeps every Tern
  record exactly.
- A player saved aboard a Tern loads on dry land, not in open water.
- The Tern's own code removed, not hidden behind a flag.

**Non-Goals:**
- The new boats (M23, M34). They are a separate change, on a request.
- Moving a saved Tern into a new boat. Left possible, for when there is one.
- Fixing the capsize (below): the Tern is going; the cause is reported for
  the boats that stay.

## Decisions

### 1. Old saves: the Kestrel's rule

`"tern"` joins `Kind::RETIRED`. On load a Tern record is held aside, as read,
in `Fleet::retired`; the writer appends it unchanged; `next_id` never falls
below its ID. Its `sheet` field stays in layout 1 for those records, as
`nacelle` and `brake` did for the Kestrel. No version change.

- *Recommendation taken (ask only with screenshots).* The alternatives are
  the ones `retire-kestrel` turned down: keep it sailable in old worlds
  (keeps all its code), delete the record (drops something from a save),
  turn it into another boat (invents a rule for a thing the player did not
  make).
- The Tern has no hold, so there is no cargo to move anywhere. The player
  never made a Tern (no recipe makes one), so nothing the player made is in
  it.

### 2. A player saved aboard a Tern is put ashore

Occupancy is not saved: the pose is the walker's, which rides the seat while
aboard, so a player who quit aboard loads at the seat. With the Tern held
aside that is open water, possibly far from land.

On the fleet's first spawn after a load, if the walker stands over the sea
within 6 m of a held retired record, it is moved to the nearest dry land
(`PlanetContact::find_land_near`, the start search's own rule), keeping its
heading. The log says so. Any other load is untouched: a player who was
merely swimming somewhere else is not moved.

### 3. Harbours: a `sail` berth is left empty

`harbour::kind_of` returns no craft for `sail`; `boats_for` skips those
berths, counted with the skipped ones. A held Tern's berth stays in the
"made" set (the retired records' berths count), so an old world's harbour
does not grow a new boat at it. The towns' templates and charts are not
touched: the berths are part of the town record and stay where they are.

### 4. The code comes out

- `Kind::Tern`, `CraftState::Tern`, `Telemetry::Tern`, `Hulls::tern`, the
  `tern` field of `VehicleSpecs` and its `TernSpec`, `SailSpec`, `CrewSpec`
  and `KeelLift` go, with `tern.rs` and the Tern's section of
  `vehicles.ron`.
- In the app: the draw arm and its moving parts (boom, tiller, keel, crew),
  the HUD panel, the chase framing of the masthead, the view index, the key
  group TERN, `--aboard tern`.
- The foil model stays (the cog sails on it); so does the hull model.
- The Tern's authored but unregistered Blender model
  (`assets/models/vehicles/tern/`) and its builder in `author.py` go;
  `vehicle_art` stops exporting its hull.

### 5. Saved rebinds

A rebind of a TERN key (`tern:KeyW` and the like) in a settings file is
ignored on load and kept in the file, by the rule `retire-kestrel` decision 3
already made for unknown slots. Nothing new is needed; a test pins it for
the Tern.

### 6. Tests

- Tern-only tests go with their requirements (sailing, keel lift, the Tern's
  panel and chase framing).
- Tests that used the Tern as "any craft" switch to the Loon or the rowboat.
- New: an old build's vehicle file holding a Tern opens with its other craft,
  the Tern record written back byte for byte; a new world places no Tern; a
  harbour moors no Tern; a walker loaded at a held Tern over water is put on
  land.

## Why it was capsized (owner: "idk why but that boat is capsized lol")

Not a frame bug. The saved orientations were measured against the planet's
radial up in every dev save on this Mac (a script reading `vehicles.ron`):
moored craft lie within 1 degree of upright, which a wrong frame after
`world-frames` would not give. The tipped boats are the **swamped** ones:
- in the worlds made before `small-boats` 1.4 (the tide fix), the new
  world's Tern holds its full bilge (`bilge_kg: 900.0`, its capacity) and its
  Loon holds 700 kg (its capacity) heeled 50 to 81 degrees;
- `small-boats` 1.4 found the cause for the Loon: the boats were made at the
  mean sea level and the tide came over their sheer at the berth. New craft
  are now made on the tide, and a fresh world's Loon stays dry;
- but water already aboard never leaves an unattended boat. The Tern drains
  0.6 kg/s; the Loon, the rowboat and the raft drain nothing
  (`drain_kgps: 0.0`), and only a paddler's bailing (B) empties them. A boat
  swamped in an old world stays swamped, heeled by its slosh, for good.

So the boats that stay can show the same thing: a Loon or rowboat in a world
made before the tide fix, or one that ships water at its berth, lies swamped
and tipped until a player boards it and bails. A fresh world on the current
build was run for 100 s near Marenstrand: its harbour rowboat nearest the
player took on 57 kg, and none heeled past 1 degree. Fixing it (a drain for
an unattended open boat, or righting a swamped one) changes how the kept
boats behave, so it is not done here; it is reported to the owner.

## Risks / Trade-offs

- **The held records grow with the world** (31 Terns in a typical world):
  they are written back on every vehicle-file rewrite, as the stowed records
  already are. Negligible.
- **The Kestrel-era fixture holds a Tern.** It now exercises two retired
  kinds at once, which is what it should do.
- **M19's mockup still shows the Tern.** Its tracker row says "Tern retired",
  and the page is not republished for this.
