## Context

- **The map** (`desktop/map_screen.rs`) is a menu screen. Left-drag pans it,
  the wheel zooms, Home finds the player, and `markers` lays out the player,
  the ship and every craft from body-local positions through `geo::project`.
- **The compass** (`compass.rs`, `desktop/compass_bar.rs`) turns body-local
  points into `Place { bearing, distance_m }` with `Place::seen`, and lays out
  marks: N, E, S, W, the four between, ticks, and `Place(i)` diamonds for the
  twelve nearest towns in reach, naming the one nearest the middle.
- **The save** is a journal of lines. Records (`pbd_core::records`) are
  "something generated once that must then stay put or be able to change",
  each with a kind, a stable id, a schema version and a RON body kept as
  text, and every line names its author. A kind this build does not know is
  kept as it came.

## Goals / Non-Goals

**Goals:**
- Place and remove waypoints from the map in two clicks.
- See every waypoint on the compass while walking, flying or sailing.
- Waypoints outlive the session and every later change, as everything the
  player does must.

**Non-Goals:**
- **Naming, renaming or recolouring a waypoint.** They are numbered. A name
  field can come later in the same record.
- **A route, or a line drawn to a waypoint in the world.** The compass gives
  the bearing; a beam or a ground path is a separate request.
- **Waypoints on other bodies.** A waypoint belongs to the body it was placed
  on. Only one body has a map today.
- **Sharing waypoints between players.** The record carries its author, so
  multiplayer can decide later.

## Decisions

Every decision here is a recommendation taken (ask only with screenshots);
the owner sees the capture shots before code lands in a PR.

1. **Right-click places and removes.** Left-drag is the pan, and a left-click
   that ends a drag must not drop a pin. A right-click within 12 px of a pin
   removes that pin; anywhere else on the map it places one. No dialog, and
   no confirmation: putting a pin back is one click.
2. **At most eight, numbered 1 to 8, each with its own colour.** A new
   waypoint takes the lowest free number. At eight, a right-click on empty map
   shows "Eight waypoints: right-click one to remove it" in the readout for
   three seconds and places nothing. Eight keeps the bar legible, and eight
   distinct colours can be told apart against the bar and the map.
   Alternative: one waypoint only, which is simpler but means removing one to
   mark the next place.
3. **A waypoint is a direction on the body, not a height.** It is stored as
   the unit direction from the body's centre (f64 x, y, z), so the map
   projects it, the compass gets its bearing from `Place::seen` and its
   distance along the ground, and a change to the terrain never moves it.
   The compass distance is along the ground, as a town's already is.
4. **Every waypoint shows on the compass, whatever its distance.** Towns are
   limited to the nearest twelve in reach so the bar does not fill with
   them. A waypoint is the player's own, so all of them always show. They
   are drawn as pins on the bar's foot row, as the town diamonds are, so a
   letter is never hidden, and over the towns where both fall on the same
   spot.
5. **The label goes to the waypoint nearest the middle**, ahead of a town:
   "Waypoint 3 · 1.2 km". When no waypoint is within the bar's label window,
   the town rule is unchanged.
6. **Saved as records of a new kind, `waypoint`.**
   - Each waypoint has a stable id from the journal's id counter, never
     reused, and its number 1 to 8 is a field of the record.
   - A body holds `(direction: (f64, f64, f64), number: u8)`, schema
     version 1.
   - Placing one writes the record; removing it writes the record's removal
     (the journal's existing way to retire a record).
   - Both go through the same durable path as an edit, on the frame of the
     click, authored by the player.
   - Old saves have no `waypoint` records and open with none: no migration.
   - An older build keeps the records as text, so a waypoint survives a round
     trip through it.
7. **The arrival rule is none.** A waypoint stays until it is removed. A
   waypoint that is reached has distance 0 m on the bar and the map; removing
   it is the player's choice. Alternative: auto-remove within 10 m, rejected
   because it surprises a player who meant to come back to the spot.
8. **The instrument**: `--waypoint LAT LON`, repeatable, in `--at`'s frame
   (body frame, +Y north), places waypoints at launch and is otherwise a
   normal placement, saved with the world. It exists so a capture can show
   pins.

## Risks / Trade-offs

- **The bar's foot row gets busy** when a waypoint sits by a town. Pins are
  drawn over diamonds and are a different shape (a teardrop, not a diamond),
  so both read. Verify in a capture beside a town.
- **Right-click on a Mac trackpad** is a two-finger click. Ctrl+click is the
  same on macOS and is accepted too.
- **A removal line written by this build, read by an older one.** The older
  build keeps unknown kinds as text and must keep the removal too, or the
  waypoint would come back in it. Verify against how the journal retires
  records of a kind the build does not know, before the save code lands.

## Verification

- **Unit tests:**
  - placing and removing on the `Waypoints` resource, including lowest free
    number and the eight-cap refusal;
  - the compass marks for a waypoint at a known bearing and distance;
  - a journal round trip: place two, remove one, reload, and one remains with
    its number;
  - an old save with no waypoint records opens with none.
- **Captures** in `docs/screenshots/waypoints/`, placed with `--waypoint`:
  - the map with three pins;
  - the compass facing a pin with its label;
  - the compass with a pin and a town on the same bearing.
- **A real-time check** of right-click placing and removing on the map.
