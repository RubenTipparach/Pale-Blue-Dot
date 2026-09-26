# Weather: calm clouds

The cloud pace is built and its requirement is in
`openspec/specs/world/weather/spec.md`. What remains here is the smoothing.

## ADDED Requirements

### Requirement: The clouds pass draws no pattern

The per-pixel offset of the cloud march SHALL be unstructured noise, and the
march's step count SHALL grow with the span it crosses, so that no regular
pattern appears across a cloud without temporal accumulation.

#### Scenario: A native capture of a sky of cloud

- **WHEN** the same view is captured at native resolution
- **THEN** no regular halftone is visible across the clouds
