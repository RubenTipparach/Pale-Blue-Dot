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
`saves/`: every directory there is listed as a world. Names are local
date and time, `YYYY-MM-DD-HHMMSS`, then `.png` and `.txt`; a second press in
the same second gets `-2`. The text is `key: value` lines, one per field, with
the command last, so a person reads it and a script greps it.

The PNG is written by the `Screenshot` observer, as F12's is; the text and
the note are written when the PNG is, so a failed shot says so instead of
claiming success, and the note is not in the shot.

## 5. The note

`farm::FarmNote`, the line under the crosshair, for 2 s: "LOCATION SAVED".
A failed write says "LOCATION NOT SAVED". Nothing else is drawn.

## Verification

- `repro` test: a record built from known numbers has every field, and its
  command, fed to the game's own `Launch::parse`, gives back the place, yaw,
  pitch, hour and day; and the heading the walker starts with from that yaw
  is the forward the record was taken from.
- In the game: the key pressed by a scripted input on a running `--walk`,
  then the written command run; the two screenshots side by side in
  `docs/screenshots/repro-capture/`.
