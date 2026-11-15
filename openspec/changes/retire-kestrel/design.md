## Context

The Kestrel is one of three craft placed by `vehicles::place::spawn_fleet`
when a world has no fleet yet, and every placed craft is saved to the world's
vehicle file (`pbd_core::vehicle::record::VehicleFile`, layout version 1).
Every world played since the vehicles landed has a Kestrel record in it. The
record names its kind as text (`kind: "Kestrel"`) and carries the Kestrel's
nacelle and brake as optional fields.

The owner wants the plane gone, in favour of the aircraft being designed in
`vehicle-yards`. The save rule (CLAUDE.md) says nothing a player made is
dropped and opening a world never resets it.

## Goals / Non-Goals

**Goals:**
- No Kestrel in any world from this build on: none placed, none drawn, none
  flown.
- Every existing save still opens, and its vehicle file keeps the Kestrel's
  record exactly.
- The Kestrel's own code removed, not hidden behind a flag.

**Non-Goals:**
- A replacement aircraft. That is `vehicle-yards`.
- Moving a saved Kestrel into a new aircraft. That is left possible, for when
  there is one.

## Decisions

### 1. Old saves: hold the record aside, write it back unchanged

On load, a record whose `kind` is a retired kind (`"Kestrel"`) is not turned
into a `Craft`. It is kept, as read, in a `Retired` list on the fleet
resource. When the vehicle file is rewritten, those records are written after
the live ones, unchanged, and `next_id` is never lowered below their IDs, so
no new craft can take a retired one's ID.

- *Recommendation taken (ask only with screenshots).* Alternatives:
  - **Keep it flyable in old worlds only:** this keeps all its code, which is
    the opposite of what the owner asked.
  - **Delete the record on load:** this drops something from a save, which
    breaks the save rule, and it cannot be undone.
  - **Turn it into a boat or a crate:** this invents a rule for a thing the
    player did not make.
- Because the record keeps its text kind and its own optional fields, no
  layout version change is needed. A reader of version 1 already reads it.

### 2. The code comes out, the types stay narrow

- `Kind::Kestrel` and `CraftState::Kestrel` are removed. `Kind` parsing
  reports a retired kind as `Retired` rather than unknown, so the loader can
  tell "hold aside" from "refuse".
- `kestrel.rs`, the Kestrel's telemetry and HUD panel, its authored model under
  `assets/models/vehicles/`, its swatches, its draw arms and its view index
  go.
- `VehicleRecord`'s `nacelle` and `brake` fields stay. They are part of
  layout 1, and the retired records carry them.

### 3. Bindings and the Input tab

The Kestrel's bindings (throttle, nacelle tilt, brake, assist) leave the
binding table, `--help` and the Input tab. The Input tab's "Flying" sub-tab
goes with them while no craft flies, and comes back with the next aircraft.

A player's saved rebinding of a removed action is ignored on load and kept in
the settings file. This uses the same rule the rebinding design uses for an
unknown action, so the file is not rewritten without it.

### 4. The start does not move

`walking.rs` finds the start on a level plain over the sea, the place the
Kestrel's pad was looked for. That search is unchanged; only the names in it
lose "pad". A new world therefore starts where it did before this change. The
spawn digest tests are untouched and must still pass.

### 5. Captures and tests

- `--ship-in-cave` and its scene go. That scene was the Kestrel parked in a
  cave chamber, to show the moving things' light, and a boat in a cave would
  be odd. Whatever the scene's test pinned about lighting is retargeted to a
  lit thing that stays: a town lamp, or a moored boat's lantern. This is
  recommendation taken (ask only with screenshots).
- Tests that test only the Kestrel are removed with the requirements they
  pinned.
- Tests that use the Kestrel as "any craft" (boarding, leaving, records,
  frame placement) are switched to the Tern.

## Risks / Trade-offs

- **A held record could be lost by a later rewrite path that forgets the
  retired list.** Mitigation: one test loads a save with a Kestrel, moves a
  boat, saves, and checks the Kestrel's record is still there and
  byte-identical.
- **The UI agent is editing `controls.rs`, the HUD and the menu now.** This
  work starts after that lands, on top of it.
- **M19's mockup still shows the Kestrel.** It is the design record of a
  retired craft. Its row says so, and the page is not republished for this.

## Findings when the code was met (2026-11-15)

What the design assumed and the code turned out to say, and what was done.
Each is a recommendation taken (ask only with screenshots) where it needed a
choice.

- **The saved kind is `"kestrel"`, lower case.** Decision 1 writes
  `kind: "Kestrel"`; a record's `kind` is `Kind::key()`, and every save says
  `kind: "kestrel"`. The retired list (`Kind::RETIRED`) holds `"kestrel"`, and
  `Kind::parse` reports it as `KindKey::Retired`. Nothing else changes.
- **The Flying sub-tab is not only the Kestrel's.** It holds two groups:
  FLYING, the free flight's keys (R toggles walking and flying), and KESTREL.
  The free flight stays, and `interface-look` ("Key bindings are grouped by
  context and can be changed") requires a flying sub-tab and pins that E
  "still rolls in flight". So only the KESTREL group left the table, `--help`
  and the Input tab; the Flying sub-tab stays, holding the free flight's
  keys. Removing the tab would have moved the free flight's bindings
  somewhere the design does not say.
- **The rebinding rule did not keep an unknown rebind.** Decision 3 says a
  removed action's rebind is kept "by the same rule the rebinding design
  uses for an unknown action". That rule (`stone-and-wood-ui`, "Rebinding: the
  file") skipped it with a warning, and the next write dropped it, because
  the file is written from the binds that were read. Keeping it needed
  `KeyBinds` to carry what it skipped (`kept`) and write it back. That now
  holds for every rebind this build does not know, a later build's as well
  as a removed one's. A kept rebind moves no key and is never shown. Pinned by
  `settings::tests::a_rebind_of_a_retired_slot_is_ignored_and_kept`.
- **The authored-model loader served only the Kestrel.** `model::AUTHORED`
  listed the Kestrel alone (the Tern and the Loon are built from their numbers;
  their Blender sources in `assets/models/vehicles/` are not registered). The
  loader stays for the next authored craft, with an empty list. The Kestrel's
  export contract, and the checks only it used, are removed. The tests that
  run per registered craft now run over none, and they will run again when a
  craft is added. The swatch provenance test still checks every swatch left.
- **`--ship-in-cave` had no lighting test of its own.** Its one test held the
  Kestrel where the rig put it, and that test goes with the rig. What the
  scene showed, the moving things lit by the field, is pinned on the boats
  instead: `vehicles::tests::every_craft_is_lit_by_the_field`. The field's
  own tests never used the Kestrel.
- **An old world's boats can be stowed when it opens.** The fixture is an old
  build's own file, and its Tern lay 1.4 km from that world's start. The test
  planet starts its walker elsewhere, so both boats open stowed
  (`cities-in-the-world` 4.2b). The test counts the world's craft and the
  stowed ones, then brings the Tern near the walker before moving it.
- **`inputs.json` was already stale** (no cog or rowboat). Only its Kestrel
  entries (`specs.kestrel`, `wing_panels`) were removed. It was not
  regenerated, so the Tern and Loon inputs are byte for byte as they were.
- **`tools/art/ship_tiers_art.py` copied the Kestrel's model** into the
  ship-tiers mockup. It no longer does. The copy the mockup already has is
  kept, and the page is not republished.
