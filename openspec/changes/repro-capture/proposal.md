# Proposal: a key that saves where the player is, for a repro

## Why

The owner, 2026-11-16: "add a button for capturing location data so I can
help repro stuff (shift alt F or something)".

A bug seen in play is reported today as a screenshot and a description, and
the agent that picks it up has to guess where it was: which world, which
spot, which way the camera faced, what hour, what weather. `position-readout`
put the place on the HUD's debug line and in the log, but the owner has to
read it off and retype it, and the rest (the look, the hour, the day, the
weather, the settings) is in four other places or nowhere.

## What changes

- **Shift+Alt+F** in play, on foot, flying, swimming or aboard, and with any
  panel open, writes a repro record:
  - a screenshot of the frame as shown, through the same screenshot path the
    captures use;
  - a text file: world name, seed and generator version; the latitude and
    longitude `--at` takes (and the map's reading beside it); elevation,
    height above ground and eye height; yaw and pitch as `--yaw` and
    `--pitch` take them; walking, flying, swimming or aboard, and whether in
    water and how deep; the clock's hour, day and season; the weather here
    (rain or snow shown, wetness, cover, wind); the look settings (render
    scale, reflections, rain shadow); the build's git commit; and a
    ready-to-paste command line that puts an agent there with `--capture`.
- Both go to `repro/<date>-<time>.{png,txt}`, beside `saves/`. The log gets
  one line, `REPRO <path> <command line>`.
- The blocks round the player (owner, 2026-11-16: "Shift alt F should also
  read local block data as well within a range of 3 of the player", "So 3 up
  3 down 3 forward, left etc"): every cell within 3 hex rings, the layers 3
  below the feet to 3 above the head, with material, edit or generated,
  water and the records on each cell; a table in the text and a JSON file
  beside it. Read from resident data only.
- The whole record goes on the clipboard (owner: "It should auto save to my
  clipboard"): the command first, then the fields, the block table and the
  screenshot's path, by the platform's own clipboard command (`pbcopy`,
  `clip`, `wl-copy` or `xclip`), with no new crate.
- On screen: "LOCATION COPIED" for 2 s on the line under the crosshair the
  farm's notes use. Nothing else.
- The key is a row of the controls table (SCREEN), drawn on the settings
  page and in `--help`.

## Not in this change

- The command reproduces the GENERATED world: it runs without `--world`, in a
  memory-only world of the same seed and generator. Running it with the
  owner's `--world` would load that world's saved pose over `--at` and then
  write the capture's pose into the save (`open_world`), which no repro may
  do to a player's world. The record lists how many edits the world holds,
  so a view that depends on them says so.
- Weather is simulated, not seeded: no flag recreates the storm the owner
  stood in. The record gives `--rain <rain>` (a storm forced to that
  strength) when it was raining or snowing, and says it is an approximation.
- A flight's roll is not carried: `--fly` starts level.

## Impact

- New: `pbd_app::repro` (the record and its command line, pure, tested),
  `desktop/repro.rs` (the key, the screenshot, the files, the note).
- `--generator <v>`: a launch flag for a memory-only run, so a world of an
  older generator is reproduced on its own terrain. Refused with `--world`.
- `pbd_app::place`: the yaw `--yaw` means, as a function both the walker's
  start and the flight's start call (one formula where there were two
  agreeing copies).
- `walking`: the walker's water state made readable to the record.
- `controls`: a chord row (SHIFT + ALT + F) that is listed but not rebindable.
- `build.rs`: the git commit compiled in as `PBD_GIT_COMMIT`.
