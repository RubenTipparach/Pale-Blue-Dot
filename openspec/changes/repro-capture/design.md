# Design: the repro key

## 1. The key

Shift+Alt+F. Nothing reads Alt today, so the chord clashes with nothing on
Shift (sprint, cruise) or Alt. F alone is interact; the repro reader runs in
`PreUpdate` after `controls::remap_keys` and before every F reader and the
menus, and on the chord it clears F's `just_pressed`, so the press that saves
a repro never also opens a door or boards a boat.

It reads the PHYSICAL keyboard (the `KeyboardInput` messages), not
`ButtonInput<KeyCode>`, which the rebind remap rewrites while a slot is
moved: with interact rebound to K, the F in `ButtonInput` is K's, and the
chord would follow the rebind. It runs whether or not a menu is open (the
owner wants panels in the shot), and not while the saves page's name field
has the keyboard, where Shift+Alt+F is typing.

The controls table gets a row in SCREEN: `SHIFT + ALT + F`, "save the
location for a bug report". A chord row is listed but not rebindable (a new
`chord` flag on `Binding`, like `range`): the slot model binds one key to one
action, and a three-key chord in it would make Shift and Alt slots of their
own. Recommendation taken (ask only with screenshots): fixed, like Escape.

## 2. The numbers, and the flags they go back through

All from the active `Camera3d`, less `PlanetRenderFrame.center` in f64:

| Record | Read from | Flag |
| --- | --- | --- |
| lat, lon | `place::Place` (`geo::lat_lon`, the body frame) | `--at <lat> <lon>` |
| map reading | `geo::compass_lat_lon` | none (printed for the reader) |
| elevation, above ground | `Place` | none |
| eye height | eye radius less the walker's feet, or less `terrain_radius` flying | `--fly --height` (flying) |
| yaw | `place::yaw_of(up, forward)` | `--yaw` |
| pitch | `asin(forward . up)`, degrees, up positive | `--pitch` (clamped to +/-89) |
| hour, day | `Sun.clock` | `--time`, `--day` |
| render scale, reflections, rain shadow | `PlayerSettings` | `--render-scale`, `--reflections`, `--rain-shadows` |
| generator | `WorldSave.identity.generator` | `--generator` when not this build's newest |

`--at` takes the FRAME latitude (+Y north); the HUD's "map 28.250S 0.750E"
is the compass's, whose north is the frame's -Y (`geo.rs`). So the record
writes the frame pair to the flag and prints the map pair beside it, as
`position-readout` does. A HUD reading of "28.250S" is `--at 28.25 ...`, not
`-28.25`.

**Yaw.** The walker starts facing `R(up, -yaw) * normalize(Y x up)`; the
flight starts facing `-east * cos(yaw) + north * sin(yaw)` with the compass's
north and east. Those are the same direction (Y x up is frame west, which is
compass west; R(up, -90 deg) of it is compass north). One function,
`place::heading_at_yaw(up, yaw)`, replaces both, and `place::yaw_of(up,
forward)` inverts it: `yaw = -atan2(h . (up x w), h . w)`. In degrees that is
clockwise from compass west seen from above (90 faces compass north), which
is what the flag has always meant; the record only computes it.

**Pitch.** The walker's pitch is `asin(forward . up)`, up positive, and
`--pitch` is that in degrees; `--fly --pitch` is the same sense
(`view_pitch_down = -pitch`). The flag refuses past +/-89, so a record looking
straight up or down writes 89.

**Mode.**

- On foot (walking or swimming): `--walk --at`. `--at` is an exact start
  (`WalkingConfig::exact_start`): the walker stands at that direction, on the
  ground or floating at the sea's sheet where it is water, which is where a
  swimmer is. `--swim` is not used: it holds W, and the walker would swim
  off the spot.
- Flying: `--fly --at --height <eye over terrain_radius>`.
- Aboard: the craft is not reproduced (it is the world's, and `--aboard`
  makes a craft at the spawn); the command stands the walker where the eye
  is, and the record names the craft.
- Upstairs in a town, on a deck or in a cave: `--at` stands the walker on
  the floor the start search finds there, which may not be the one the
  player was on. The record's height above ground says so; `--up` is left
  for the agent to add by hand.

**Time.** `--time <hour>` to four decimals and `--day <day>`. A capture pins
the clock there.

**Weather.** `Weather.rain`, `.wetness`, `.cover`, `.snowing`, and the wind
from `Air.now.sample(here).wind` (speed, and the compass bearing it blows
toward). The command adds `--rain <rain>` when `rain > 0.05`: a storm forced
to that strength here, which is the nearest flag. Recorded as approximate.

**Commit.** `env!("PBD_GIT_COMMIT")` from a `build.rs` running `git
rev-parse --short=12 HEAD`, rerun when HEAD or its ref moves; "unknown" when
git is not there (a source tarball).

## 3. The command line

```text
target/fast/pbd-app --walk --at -28.25 0.75 --yaw 132.4 --pitch -8.0 \
  --time 14.5000 --day 3 --render-scale 100 --reflections high \
  --rain-shadows off --capture repro/2026-11-16-142233-again.png
```

No `--world`: a capture with no `--world` is memory-only (`open_world`), so
the repro never touches the owner's saves. With `--capture` the settings
file is not read (`settings::for_launch`'s isolated path), which is why the
look settings are passed as flags. `target/fast/pbd-app` is the `fast`
profile's binary (`.exe` on Windows).

## 4. The files

`repro/` beside `saves/` (both relative to the working directory). Not inside
`saves/`: every directory there is listed as a world. Names are the UTC
date and time, `YYYY-MM-DD-HHMMSS`, then `.png`, `.txt` and `.blocks.json`; a
second press in the same second gets `-2`. The text is, in order: the command
line, the `key: value` fields, the block table (section 6), and the
screenshot's path last.

**Clipboard (owner, 2026-11-16: "It should auto save to my clipboard").**
The whole text, exactly as written to the `.txt`, goes on the clipboard, so
the owner pastes it straight into a chat; the image cannot go on a text
clipboard, so its path is the last line. It goes through a `Clipboard` trait
(`repro::write_record`), the game's `SystemClipboard` running the platform's
own command (`pbcopy` on macOS, `clip` on Windows, `wl-copy` or `xclip` on
Linux) on a thread, so no crate is added (`arboard` would bring its own
objc2 and windows bindings for one debug key). A failed copy is a warning;
the files are written regardless. A test hands a fake the record and checks
it got the file's text byte for byte.

The PNG is written by the `Screenshot` observer, as F12's is; the text and
the note are written when the PNG is, so a failed shot says so instead of
claiming success, and the note is not in the shot.

## 5. The note

`farm::FarmNote`, the line under the crosshair, for 2 s: "LOCATION COPIED".
A failed write says "LOCATION NOT SAVED". Nothing else is drawn.

## 6. The blocks round the player

The owner, 2026-11-16: "Shift alt F should also read local block data as
well within a range of 3 of the player", then "So 3 up 3 down 3 forward,
left etc".

- **The box.** Every cell within 3 HEX RINGS of the player's cell, counted
  over the finest level's own neighbour table (`FineSet::finest_neighbors`,
  five sides at a pentagon): 37 cells away from a pentagon. In each, the
  layers from 3 below the feet's layer to 3 above the head's (the feet's
  layer is the one the soles stand in, the head's the eye's): 8 layers on
  foot.
- **Facing.** The side of the player's cell the look faces most nearly
  (0..5 in the neighbour table's order) and its compass bearing. Each cell is
  labelled with its ring and its offset from the player's cell in cell
  spacings, forward or back along the look and left or right across it.
- **Per cell:** its key; the record's surface height and the column's
  surface layer; the biome and the cleared-for-a-town bit, from the
  record's material word (`terrain::surface_code`, already resident: not
  re-generated); the top material; the water standing over the surface.
- **Per layer:** the column's material (edits applied: the tier's columns
  are what the game walks on and draws), whether the save holds an edit at
  that layer (`Edits::for_cell`), and its water: source, flowing N or
  falling N, and the top of its run.
- **Records on the cell:** every stored record whose id names it: a tree
  (logs taken, and "felled" at the gathering's axe count), a picked wild
  plant, a plot, fruit, a station (`id >> 16`), a fence or gate (either of
  its edge's two cells), an animal living there. Lamps are materials, so
  they are in the layers.
- **Resident only.** Nothing is generated: the records and columns of the
  current fine set and the save's edits and records. A cell with no
  resident column says so and lists no layers. Measured cost: **0.11 ms**
  to gather the whole record with its 37-cell dump on the M4 Max Mac (one
  frame, on the press only; the `REPRO gathered in` log line).
- **Out:** a table in the text record, compact (one line of layers per
  cell, top down, `*` marking an edit), and the same as JSON in
  `<stem>.blocks.json` for an agent to load.
- **Test.** The dump is compared with the tier read directly: the same cells
  as a walk of the neighbour table to 3 rings, each layer's material the
  column's, and a dig made through `Edits` flagged as an edit and read as
  air.

## Verification

- `repro` test: a record built from known numbers has every field, and its
  command, fed to the game's own `Launch::parse`, gives back the place, yaw,
  pitch, hour and day; and the heading the walker starts with from that yaw
  is the forward the record was taken from.
- In the game: the key pressed by a scripted input on a running `--walk`,
  then the written command run; the two screenshots side by side in
  `docs/screenshots/repro-capture/`.

## Result (2026-11-16, the M4 Max Mac, `--profile fast`)

- `PBD_PRESS_REPRO=200` pressed Shift+Alt+F through the keyboard's own
  messages on a `--walk --at 20.3 40.2 --yaw 60 --pitch -10 --time 15 --day 2`
  run. It wrote the PNG, the text and the JSON (37 cells, 8 layers each),
  logged `REPRO`, put the text on the clipboard (`pbpaste` matched the file
  byte for byte) and gathered in 0.11 ms. F did not interact.
- The written command, run as written, gave the same view
  (`docs/screenshots/repro-capture/pressed.png` against `reproduced.png`:
  mean absolute difference 3.8 of 255, the grass and leaves' sway).
- A plain launch (`--walk --world reprotest`) reached READY with no
  `LATE_PIPELINE`. The two `--capture` runs log `LATE_PIPELINE` lines for
  Bevy's standard mesh pipelines after READY; this change adds no render
  code, so they come from the capture path's own prewarm (50 pipelines
  against a play launch's 111), not from this change. Not compared against
  a build of the branch before it.
