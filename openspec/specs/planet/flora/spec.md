# Flora Specification

## Purpose
What grows on the ground and how it is decided: plants are decoration drawn
from the cell record at draw time. The rest of flora v2 is designed in
`openspec/changes/flora-v2` and moves here as it is built.

## Requirements

### Requirement: Nothing grows where water stands

A cell whose top has water standing on it SHALL be drawn as a bed: a grass
of any kind or snow as dirt, sand as the seabed's sand. No tuft, flower,
fern, mushroom, bush or tree SHALL be drawn on it or be workable there. The
rule SHALL be decided at draw time from the column (and the generator's
river beyond the column tier), SHALL NOT change any cell's material or any
save, and SHALL need no new generator version.

#### Scenario: A river's bed

- **WHEN** the player looks down at a river's bed in any world, old or new
- **THEN** the bed is dirt, with no grass, flowers, mushrooms or trees on it
