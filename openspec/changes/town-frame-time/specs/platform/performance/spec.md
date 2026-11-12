# Performance: towns and weather are measured

## ADDED Requirements

### Requirement: The suite walks through a town under changing weather

The performance suite SHALL have a `town` scenario that walks from the middle
of a walled town out through it by day with rain forced on, and SHALL mark a
run invalid when the walk did not finish.

#### Scenario: A town walk is checked

- **WHEN** a `town` run's log has no `WALK_DONE` line
- **THEN** the suite marks the run invalid and leaves it out of the report

### Requirement: A slow frame can be traced to towns and weather

With `--frame-log`, each frame's row SHALL carry the town vertices baked, the
town materials rewritten, and the weather pair's generation and blend, and any
town bake or weather upload taking over 2 ms on the main thread SHALL be
logged with its cost.

#### Scenario: A bake lands in a frame

- **WHEN** a fine set lands near a town during a `town` run
- **THEN** that frame's `baked_verts` is non-zero, and if the bake took over
  2 ms a `SPENT` line names it
