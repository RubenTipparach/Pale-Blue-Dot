# Player settings

## ADDED Requirements

### Requirement: The player's settings are kept apart from the game's data and saves

The game SHALL keep the player's settings in one versioned file in the
user's configuration directory, outside `assets/` and outside `saves/`. A
missing, unreadable or older file SHALL give the defaults for whatever it
does not say, and the file SHALL be written only when the player changes a
setting. Reading or writing it SHALL NOT read, write or alter any saved
world.

#### Scenario: No file yet

- **WHEN** the game starts and there is no settings file
- **THEN** every setting has its default
- **AND** no file is written until the player changes one

#### Scenario: A file from an older build

- **WHEN** the settings file has no version and only some of the fields
- **THEN** the fields it has are kept and the rest take their defaults

#### Scenario: A broken file

- **WHEN** the settings file is not valid
- **THEN** every setting has its default
- **AND** the file is left as it was

#### Scenario: Round trip

- **WHEN** settings are written and read back
- **THEN** they are the settings that were written

### Requirement: Captures and benchmarks take their settings from their flags

A run with `--capture` or `--frame-log` SHALL NOT read the player's settings
file. It SHALL take the engine as it was before these settings (uncapped,
100 percent, auto step on), overridden only by `--frame-limit`,
`--render-scale` and `--no-auto-step`.

#### Scenario: A benchmark on a player's machine

- **WHEN** the player has chosen a 67 % render scale and a perf run starts
  with no `--render-scale`
- **THEN** the run renders at 100 %

### Requirement: The frame rate can be capped at 60 or 120

The player SHALL be able to cap the frame rate at 60 or 120 frames a second,
or leave it uncapped. A new player's default SHALL be 60 (the owner's choice,
2026-11-13). A cap SHALL hold the mean frame
period at the cap without drifting, and SHALL NOT run frames early to make
up for a late one.

#### Scenario: Pacing at a cap

- **WHEN** frames finish faster than the cap's period
- **THEN** each frame is held until the next deadline, and deadlines are one
  period apart

#### Scenario: A hitch

- **WHEN** one frame lands more than a period late
- **THEN** the next deadline is taken from that frame, not from the missed
  ones

### Requirement: The 3D view can render at a fraction of the window

The player SHALL be able to render the 3D view at 100, 85, 75, 67 or 50
percent of the window's size. A new player's default SHALL be 85 (the
owner's choice, 2026-11-13). Below 100 percent the
view SHALL be stretched to fill the window, and the HUD and menus SHALL still
draw at the window's full resolution. At 100 percent the view SHALL render to
the window directly, with no extra camera or pass.

#### Scenario: Full size draws as before

- **WHEN** the render scale is 100 percent
- **THEN** every 3D camera targets the window and no present camera exists

#### Scenario: A reduced scale

- **WHEN** the render scale is 67 percent
- **THEN** the 3D cameras render into an image 67 percent of the window's
  size, and a present camera draws it across the window

### Requirement: The rain's sun shadows can be turned off
The settings page SHALL offer RAIN SHADOWS, on or off, kept in the settings
file with the others, and off for a new player and for a file written before
it existed. Off, the lit rain SHALL take the sun's light unshadowed: no
shadow-map samples in the rain volume, so trees and eaves cast no shafts
through it. A capture or a benchmark SHALL draw them unless its flags say
otherwise (`--rain-shadows on|off`).

#### Scenario: A new player
- **WHEN** the game starts with no settings file
- **THEN** RAIN SHADOWS reads OFF and the rain takes no shadow-map samples

#### Scenario: The choice is kept
- **WHEN** the player turns RAIN SHADOWS on and starts the game again
- **THEN** it reads ON and the rain's shafts are drawn
