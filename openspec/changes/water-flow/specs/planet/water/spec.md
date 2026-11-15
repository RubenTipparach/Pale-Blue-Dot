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

### Requirement: Flowing water pans its texture along its flow
The water top of a flowing cell SHALL pan its surface pattern along the cell's
flow vector, faster as the flow is faster, and a falling face SHALL pan it
downward; a still cell, a source the water does not leave, and the generated
sea SHALL not pan. The flow vector SHALL be baked per cell when the water is
meshed, as Tenebris C bakes it (`hex_mesher.c`, `bake_water_flow_uv`), so a
single phase serves and the pattern never stretches. The owner, 2026-11-15:
"Add UV scrolling and water texture or shaders to show the flow map. This is
how tenebris c does it." Proposed; the mockups (M25, M28) show it.

#### Scenario: A spring runs off a ledge
- **WHEN** a spring flows across a plain and over a ledge
- **THEN** the pattern on each flowing top moves away from the spring, the
  pattern on each falling face moves down, and a pool of sources at rest
  stands still

### Requirement: A flow-arrow overlay
The game SHALL offer an overlay, off by default, that draws an arrow on every
water cell whose flow vector is not zero, pointing along that vector and
longer the faster it flows, drawn just above the water, depth-tested so
nearer water and ground hide it, and toggled from the debug keys and the
player settings. It SHALL read the same flow vector the surface pans along.
The owner, 2026-11-15: "Add button to toggle arrows on off, the game should
have this too." Proposed; Tenebris C's F5 overlay
(`renderer_draw_water_flow_debug`) is the precedent, and the mockups' "Arrows:
on / off" toggle shows it.

#### Scenario: Checking a river
- **WHEN** the player turns the overlay on beside a river that falls over a
  ledge into a pool
- **THEN** every flowing top shows an arrow downstream, the pool's still
  water shows none, and turning it off removes every arrow

