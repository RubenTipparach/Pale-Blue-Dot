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

### Requirement: The repro record carries the blocks round the player
The record SHALL list every cell within three hex rings of the player's cell
and, in each, the layers from three below the feet's layer to three above the
head's, with each layer's material as the game has it, whether it is an edit,
its water, and the stored records on the cell, read from resident data only;
as a table in the text and as JSON beside it.

#### Scenario: The dump is what the tier holds
- **WHEN** a dump is taken over a fine set with a dig in it
- **THEN** its cells are the neighbour table's three rings, each layer's material is the column's, and the dug layer reads as air and as an edit

### Requirement: The repro record goes on the clipboard
The whole record text SHALL be put on the clipboard as written to its file,
the command line first and the screenshot's path last; a failed copy SHALL be
a warning and SHALL NOT stop the files being written.

#### Scenario: The clipboard gets the file's text
- **WHEN** a record is written
- **THEN** the clipboard is handed exactly the text the file holds
