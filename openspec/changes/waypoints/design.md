## Context

- **The map** (`desktop/map_screen.rs`) is a menu screen. Left-drag pans it,
  the wheel zooms, Home finds the player. Since `starter-missions` G3 (revised
  2026-11-16) a right click adds a waypoint, up to four, the fifth dropping
  the oldest, and a click on a pin (either button, a left one only if it did
  not drag) removes it. The pins are red teardrops with no label.
- **The save** holds them as one `map-mark` record (`pbd_core::map`), id 0,
  authored by the player, through the durable path: schema 2 is the list,
  oldest first; schema 1 was one optional mark and still reads.
- **The compass** (`compass.rs`, `desktop/compass_bar.rs`) turns body-local
  points into `Place { bearing, distance_m }` with `Place::seen`, and lays out
  N, E, S, W, the four between, ticks, and a brass stud for each of the
  twelve nearest towns in reach, naming the one nearest the middle in the
  line under the bar. It knows nothing of waypoints.

## Goals / Non-Goals

**Goals:**
- See every waypoint on the compass while walking, flying or sailing, and
  read which one is ahead.
- Tell waypoints apart on the map and the compass by a number that does not
  change under the player.
- Every save that has waypoints keeps them, numbered.

**Non-Goals:**
- **Naming, renaming or recolouring a waypoint.** They are numbered. A name
  field can come later in the same record.
- **A route, or a line drawn to a waypoint in the world.**
- **Waypoints on other bodies.** Only one body has a map today.
- **Sharing waypoints between players.** The record carries its author.

## Decisions

Every decision here is a recommendation taken (ask only with screenshots),
except where it quotes the owner; the capture shots go to the owner with the
build.

1. **Placing and removing is as built** (`starter-missions` G3, owner
   2026-11-16): a right click adds; a click on a pin's head, within 12 px,
   removes it; a left click that dragged more than 4 px was a pan.
2. **At most four (owner, 2026-11-16); a fifth drops the oldest.** This
   replaces this change's first draft (eight, the ninth refused).
3. **Numbered W1 to W4 by the lowest free number.** A new waypoint takes the
   smallest number from 1 not held by another. Placed one after another they
   go W1, W2, W3, W4 (the owner's "w1,w2...."); removing W2 leaves W1, W3 and
   W4 as they were, and the next one placed is W2. When a fifth drops the
   oldest, it takes the oldest's number. Alternative: number by place in the
   list, rejected because every waypoint after a removed one would change its
   number while the player was walking to it.
4. **The number is saved with the waypoint**: `Mark { at, set_from_m,
   number }`, the `map-mark` record at schema 3. `number` is a serde default
   field, so a schema 1 or 2 body reads, and `read_marks` numbers any
   waypoint without a number (or with a repeated one) by the lowest free
   number in list order, oldest first: an old save's waypoints come out W1,
   W2, ... in the order they were set. The record kind, its id and its author
   are unchanged; the journal keeps writing one record line per change.
5. **The map labels each pin** "W1" ... in the kit's pixel font (`KitText`),
   `LABEL` with an `INK` outline, just right of the pin's head, so it reads
   on sea, sand and snow alike. The pin stays the red teardrop.
6. **The compass draws every waypoint, whatever its distance**, as a small
   red pin, 5 x 6 native pixels, on the bar's foot row where the town studs
   are, drawn over them, so a letter is never hidden. It is the map pin's
   shape and red (`ui_kit::WAYPOINT`) with an `INK` rim, so the player knows
   it as the same thing. It fades toward the bar's ends as everything on the
   bar does, and not with distance (it is the player's own).
7. **The line under the bar names the waypoint faced.** The waypoint nearest
   the middle within `NAMED_WITHIN` (7 degrees) is named there as its label
   and its distance along the ground, "W2 1.2 km", in `TEXT` with the `INK`
   shadow the town line already uses; it is named ahead of a town. With no
   waypoint in that window the town rule is unchanged. The label is the
   waypoint's number and a distance and nothing more.
8. **The instrument**: `--map-mark-east M` (repeatable; the first due east,
   each next turned 72 degrees toward north) now also runs in play once the
   walker is out, not only with the map up, so a capture with `--yaw 180`
   (facing east) faces the first waypoint on the compass.

## Risks / Trade-offs

- **An older build reading a schema 3 body.** It parses `Mark` without
  `number`, and serde passes over a field a struct does not declare, so the
  older build reads the same waypoints and keeps them; if it then changes
  one it writes the list without numbers, and this build numbers it again
  in order on the next open (decision 4). A unit test reads a schema 3 body
  as the schema 2 struct to pin this.
- **The foot row with a waypoint beside a town**: both are on the same row;
  the pin is drawn over the stud and is a different shape and colour.

## Verification

- **Unit tests:**
  - `pbd_core::map`: lowest free number on add, after a removal, after a
    fifth drops the oldest; a schema 2 body with no numbers reads numbered
    W1.. in order; a schema 3 round trip.
  - `compass`: a waypoint at a known bearing lands on the bar there at any
    distance, and is named over a town on the same bearing; its label text.
  - `map_screen`: the saved record round trip keeps the numbers.
- **Captures** in `docs/screenshots/compass-waypoints/`, placed with
  `--map-mark-east`: the compass facing W1 with its label, and the map with
  labelled pins. Rendered in a cloud container on lavapipe with no GPU.
