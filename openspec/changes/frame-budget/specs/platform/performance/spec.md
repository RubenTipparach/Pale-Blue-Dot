# Performance: a frame budget measured on a focused window

## ADDED Requirements

### Requirement: Every frame within 16.7 ms

Owner, 2026-10-28: "16 ms, hard requirement". On the reference machine, in a
focused window, every frame of play after the warm-up SHALL take no more than
16.7 ms: no landing, rebuild, world open or weather change may exceed it.
Work that cannot fit one frame SHALL be spread over frames or done off the
main and render threads, never paid as one long frame.

#### Scenario: The scripted flights hold the budget

- **WHEN** `tools/perf_suite.py` flies `clouds`, `storm` and `walk` on the
  reference machine, focused
- **THEN** the maximum frame after the warm-up is no more than 16.7 ms

#### Scenario: Opening a world

- **WHEN** a saved world is opened and its towns' ground installed
- **THEN** no frame between the menu and play exceeds 16.7 ms

### Requirement: A measured run is a focused run

With `--frame-log`, the frame log SHALL record each change of the window's
focus and occlusion. The performance suite SHALL mark a run invalid if its
window lost focus or was hidden, and on macOS SHALL refuse to start while the
screen is locked.

#### Scenario: The window loses focus mid-flight

- **WHEN** another application takes focus during a `storm` run
- **THEN** the frame log carries a `WINDOW_FOCUS false` line
- **AND** the suite reports the run invalid and leaves it out of the report

#### Scenario: A locked screen

- **WHEN** the suite is started on a Mac whose screen is locked
- **THEN** it refuses to run and says why

### Requirement: A fine-set landing does not re-bake town light it cannot change

When a fine set lands, a standing town's baked light SHALL be recomputed only
if the tier's light changed where the town reads it. A landing whose column
tier does not reach a town in the old set or the new one SHALL leave that
town's bake as it was.

#### Scenario: A landing high above the towns

- **WHEN** a set with no columns lands 400 m up while three towns stand
- **THEN** no town's bake is recomputed in that frame

#### Scenario: A dig in a town's margin

- **WHEN** a set lands with an edit inside a standing town's margin
- **THEN** that town's light is baked again from the new set
- **AND** it eases to the new light rather than stepping
