# Performance: a rig that measures every build the same way

## ADDED Requirements

### Requirement: Frame time is split into the clouds and the rest

With `--frame-log`, each frame's row SHALL carry the GPU time of the cloud
passes and of the instrumented frame, alongside the wall time.

#### Scenario: A flight into cloud

- **WHEN** the scenic route is flown with `--frame-log`
- **THEN** the log's GPU cloud column is non-zero while cloud is on screen

### Requirement: Scenarios end themselves and are checked

A performance scenario SHALL quit on its own when done, and a run whose
stressful part did not happen SHALL be reported invalid, not measured.

#### Scenario: The scenic route finds no cloud

- **WHEN** a `clouds` run's log has no line saying the route found cloud
- **THEN** the suite marks the run invalid and leaves it out of the report
