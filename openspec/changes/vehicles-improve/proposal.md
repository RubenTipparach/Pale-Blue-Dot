# Proposal

## Why

The three vehicles already have substantial physics, but control authority,
fluid sampling and instruments disagree in ways that make their behavior
hard to trust. The investigation also found a translated-planet placement
bug and visible geometry that disagrees with the forces it represents.

## What Changes

- Correct the shared foil's lift direction in reverse flow, make Kestrel
  rotor steering depend on delivered power, and keep assisted hover finite
  when local gravity reaches zero. Measure conversion before choosing any
  further flight change; keep physical stall and pilot-controlled nacelles.
- Sample the Loon's lateral plane and skeg independently at their own
  positions. Distinguish boat motion through water from motion over ground
  in telemetry, including Tern leeway.
- Show apparent-wind side, windward VMG and explicit hull-speed percentage
  for sailing, clear vehicle state/action hints, and water/ground speeds.
  Match the canoe's visible rudder blade and the Kestrel's wing panels to
  their physical state and configured geometry.
- Frame the Tern's full rig in its default chase view and keep chase cameras
  out of shoreline terrain, as exposed by the baseline captures.
- Place a new fleet correctly on translated planets, and test vehicle
  camera selection, raw same-frame mouse look, controls and menu suppression.
- Keep a reproducible measurement example and before/after release captures.

## Capabilities

### New Capabilities

None; this strengthens the existing vehicle capability.

### Modified Capabilities

- `player/vehicles`: physically consistent control/foil forces, water-relative
  boat instruments, data-derived moving visuals and frame-correct placement
  and immediate camera controls.

## Impact

`pbd-core/src/vehicle`, its scenario tests and a measurement-only example;
`pbd-app/src/vehicles`, water-state publication in `planet.rs` /
`planet_water.rs` / `sea.rs`, app integration tests and capture help text.
No engine dependency is
added to core, no save format changes are planned, and no orbital ship
mechanics or camera easing are introduced. Main specs change only alongside
the implementation and tests establishing each new requirement.

Deferred original work remains explicit: sea-state maps per vertex, boat
swamping validation, craft-to-craft collision, sleeping, and human playtesting.
The camera requirement from the original `vehicles` change can move only
when an integration test proves that water state uses the occupied camera.
