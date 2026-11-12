# Performance: the lighting model is priced and its reflections are bounded

## ADDED Requirements

### Requirement: Each part of the reflections can be priced apart

The game SHALL let a measurement run switch off each part of the lighting
model's reflections on its own (the terrain trace, the sea trace and the
reflection capture) through `PBD_REFLECT`, and `PBD_REFLECT=0` SHALL keep
switching all of them off.

#### Scenario: Only the sea trace is switched off

- **WHEN** the game runs with `PBD_REFLECT=sea`
- **THEN** the sea draws its reflection from the sky alone, and wet ground
  still traces its reflections

### Requirement: The frame log carries GPU time per pass

With `--frame-log`, each frame's row SHALL carry the GPU time of each
top-level render pass the game times, by name, beside the existing
`gpu_clouds_ms` and `gpu_total_ms`.

#### Scenario: The bloom pass is read from the frame log

- **WHEN** a run with bloom on writes a frame log on a GPU that supports
  timestamp queries
- **THEN** every row after the first few frames has a bloom column with a
  finite, non-negative value

### Requirement: A reflection ray that cannot hit anything stops

The wet-ground reflection walk SHALL stop, and return the miss it returns
when it runs out of reach today, once its ray is rising and at or above the
highest solid layer of every column it could still enter within its reach.
The screen-space trace that follows a miss SHALL still run: the towns'
buildings and the trees are not in the columns, and the screen trace is
what reflects them.

#### Scenario: A puddle under open sky

- **WHEN** a wet fragment's reflected ray points up into open air above the
  highest solid layer within its reach
- **THEN** the walk ends in the column it is in, and the frame is the frame
  the full walk gives, pixel for pixel (`--capture` shots of Holbrook's
  lane, 2026-11-12)

### Requirement: The lighting model's cost on foot in the rain is held

On the `rain-stroll` scenario on the reference desktop (RTX 3070, 1440 x
900), the lighting model SHALL cost no more than 1.0 ms at p50 over the old
model (`PBD_LIGHTING=today`) in the same exe, measured interleaved with two
repeats.

#### Scenario: The suite compares the two models

- **WHEN** `perf_suite.py` runs `rain-stroll` with the new model and with
  `PBD_LIGHTING=today`, two repeats each
- **THEN** the new model's p50 is within 1.0 ms of the old model's
  (2026-11-12: 2.7 ms over)
