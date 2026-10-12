# Water Specification

## ADDED Requirements

### Requirement: The water surface streams along a per-cell flow
The water cap SHALL advect its wave field along a per-cell flow vector on
horizontal faces and scroll it radially on vertical faces, and SHALL read that
vector from data rather than deriving one. A zero vector SHALL leave the wave
field exactly as the unadvected form.

#### Scenario: A world with no rivers
- **WHEN** every cell's flow vector is zero
- **THEN** the sea renders identically to a cap pass without the term
