# Tasks

## 1. Investigation and planning commit

- [x] 1.1 Record source audit, reproducible core and static geometry measurements in the design, with a release capture protocol.
- [x] 1.2 Complete proposal/design/spec deltas/tasks, validate OpenSpec and run fmt, both release package suites and all-target Clippy; commit only planning artifacts.

## 2. Core forces and handling

- [x] 2.1 Correct signed foil lift; test reverse flow, force continuity and dissipative drag, preserving existing sailing/stall scenarios.
- [x] 2.2 Bound rotor steering by power and guard zero-gravity assist; test unpowered and powered inputs, zero gravity, hover and conversion.
- [x] 2.3 Sample each boat hydrofoil at its own wet point/depth; test independently immersed Loon lateral plane and skeg.
- [x] 2.4 Add explicit through-water telemetry and water-relative Tern leeway; test cross-current motion and retain labeled ground readings.
- [x] 2.5 Commit the measurement instrument, compare before/after numbers, migrate only satisfied force/rotor requirements with their tests, and run all required checks before the core commit.

## 3. Frame and camera integration

- [ ] 3.1 Fix translated fleet placement; test equivalent body-local berths, boarding and camera translation with nonzero planet centre.
- [ ] 3.2 Test direct same-update vehicle mouse look without a fixed tick, camera switching, control mappings and menu suppression; fix any exposed integration defect.
- [ ] 3.3 Prove water-state selection follows the occupied camera and migrate that original delta with the test; migrate new frame/input requirements and run all required checks before the integration commit.

## 4. Instruments and moving geometry

- [ ] 4.0 Frame the Tern rig and constrain chase camera booms against terrain; test mast projection and clear/blocked boom placement without smoothing, and inspect the formerly obstructed Loon view.

- [ ] 4.1 Expose authoritative wing panel geometry, draw matching panels, size the paddle blade from configuration and draw its active rudder state; test axes/area and blade placement including zero speed.
- [ ] 4.2 Improve the vehicle panel with a backing, clear state/action hints, apparent wind side/angle, VMG, hull percentage and water/ground motion; test meaningful panel content, retain shared bindings and verify existing vehicle capture flags are documented by --help.
- [ ] 4.3 Retain the unchanged baseline executable and inspect its three craft captures before visual edits; capture and inspect all three improved craft, including seat and translated-frame views; record evidence and limits, migrate only proven visual/instrument requirements and run all required checks before the visual commit.

## 5. Final evidence

- [ ] 5.1 Record final measurement comparisons, test counts, capture paths and unverifiable human/underway/storm behavior; confirm clean current branch and no push.
