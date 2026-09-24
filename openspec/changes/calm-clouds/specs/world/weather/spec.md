# Weather: calm clouds

## ADDED Requirements

### Requirement: Cloud drifts at a pace the eye reads as weather

The simulation SHALL carry cloud with its steering wind scaled by a validated
`cloud_pace`, and SHALL leave the transport of vapour, heat, charge and wind
unscaled. The wind the renderer drifts cloud detail with SHALL be that same
carrying wind.

#### Scenario: The pace scales cloud and nothing else

- **WHEN** one step is taken at a pace of one half and at a pace of one
- **THEN** the cloud moved by the half-pace step is the half-length move
- **AND** vapour, heat and wind are the same in both

### Requirement: The clouds pass draws no pattern

The per-pixel offset of the cloud march SHALL be unstructured noise, and the
march's step count SHALL grow with the span it crosses, so that no regular
pattern appears across a cloud without temporal accumulation.

#### Scenario: A native capture of a sky of cloud

- **WHEN** the same view is captured at native resolution
- **THEN** no regular halftone is visible across the clouds
