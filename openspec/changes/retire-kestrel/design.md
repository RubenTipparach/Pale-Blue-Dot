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
