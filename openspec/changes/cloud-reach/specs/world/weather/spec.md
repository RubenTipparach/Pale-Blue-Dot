# Weather: the cloud layer's reach

## ADDED Requirements

### Requirement: A view ray sees the cloud to the end of its span

The cloud march SHALL sample a view ray's span through the cloud layer out to
its end, or until the cloud is opaque, whatever its step budget. When the
budget is spent short of the span's end, its last steps SHALL stretch to
cover the rest. The steps nearer the eye SHALL NOT depend on the span's
length.

#### Scenario: Looking toward the horizon from inside the cloud layer

- **WHEN** the eye is inside the cloud layer and looks toward the horizon over broken cloud
- **THEN** no hard arc is drawn along the cloud base's horizon, and the far cloud deck continues above it as it does with a 256-step budget
