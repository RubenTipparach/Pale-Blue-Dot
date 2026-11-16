## ADDED Requirements

### Requirement: A key saves the player's place for a repro
Pressing Shift+Alt+F in play, on foot, flying, swimming or aboard, and with
any panel open, SHALL write a screenshot of the frame and a text record to
`repro/` beside `saves/`, named by the date and time, log one `REPRO` line,
and show a short confirmation for about two seconds. The record SHALL hold
the world's name, seed and generator version; the latitude and longitude as
`--at` takes them; elevation, height above ground and eye height; yaw and
pitch as `--yaw` and `--pitch` take them; the mode and the water; the hour,
day and season; the weather shown here; the look settings; the build's
commit; and a command line that reproduces the view. The press SHALL NOT
also interact.

#### Scenario: The command line goes back to the same view
- **WHEN** a record is made from a place, a look and a clock
- **THEN** its command line, parsed by the game's own argument parser, gives back the same latitude and longitude, yaw, pitch, hour and day
- **AND** the walker started from that yaw faces the way the record was taken

#### Scenario: A repro never writes to the player's world
- **WHEN** the record's command line is run
- **THEN** it opens no saved world, because it names none
