# Desktop prototype validation

This records the original flight-only prototype. The subsequent
[walking and camera validation](walking-validation.md) covers default walking,
immediate mouse look, explicit nearest sampling, and the updated test results.

The subsequent [controlled performance series](performance.md) records three
foreground rounds per scene, memory measurements and the final horizon shader.
Use that report for the flight-only performance baseline; the original captures and
timings below remain as the visual acceptance record.

Recorded on 2026-09-16. The final subdivision-8 desktop scene passed its release
build, tests, native rendering captures, and default/polar physical flight
checks. This accepts the current surface-column flight prototype within the
scope below; it does not complete the production voxel-game milestones.

Environment: Windows 10 Home, Intel Core i7-9700F, 32 GB RAM, NVIDIA GeForce
RTX 3070, NVIDIA driver 595.97, Vulkan, and a 1440 × 900 window. Toolchain:
Rust 1.96.0, Bevy 0.18.1, Avian 0.6.1, and wgpu 27. Terrain seed:
`0x5eed2026`. These results do not establish browser or other-adapter support.

## Completed checks

| Check | Recorded result |
| --- | --- |
| Final release workspace build, all targets | Passed |
| Application check with `--no-default-features` | Passed |
| `cargo test --release --workspace --locked` | 35 passed: 17 core tests and 18 application tests |
| Strict workspace Clippy, all targets, `-D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo run --manifest-path tools/validate_shaders/Cargo.toml --locked` | Seven standalone WGSL modules passed Naga semantic, binding, entry-point, and layout checks |
| `cargo test --manifest-path tools/validate_shaders/Cargo.toml --locked` | Three validator regression tests passed |
| Actual S8 Avian default and polar circumnavigations | Both completed in headless and rendered runs; measured results below |
| Final S8 orbit, coast, surface, night, pole, and both tour captures | Saved and inspected; no shader, pipeline, or GPU validation errors reported in their logs |
| `run.bat --headless 600` launched from the repository's parent directory | Passed, exit code 0 |
| Actual-window input and screenshot smoke | Window-message cursor capture, W + Shift, and F12 produced a flight screenshot at 211 m/s; reset/release returned to 0 m/s at 252 m sea-level altitude |
| `python tools/build_art_catalog.py` | Fourteen PNGs and 224 named material slots checked; manifest/gallery generated |
| Offline gallery JavaScript and browser rendering | Script parsed; gallery rendered and inspected in headless Microsoft Edge |

The interactive control evidence is preserved in
[pbd-manual-flight.png](screenshots/pbd-manual-flight.png). The subsequent
[ready-state screenshot](screenshots/pbd-ready.png) shows the reset ship at
0 m/s, 252 m altitude, approximately +28.6° latitude and zero longitude;
the cursor was released and the game remained open. The completed checks
establish the listed contracts and scene behavior, not a blanket acceptance
of every world or hardware configuration.

Core coverage includes gravity falloff, vector acceleration/speed bounds,
dampeners, bounded mode-change braking, f32 speed-boundary steering, negative
chunk addressing, deterministic terrain, f64 frame conversion, and analytic
celestial rails. Application coverage includes one Avian integrator, collisions,
quaternion limits, physics-clock pause/time-scale/timestep behavior, spherical
topology contracts, input delivery before physics, camera interpolation/reset,
hovering, and actual normal/polar tours.

The five volumetric-engine shader ports and two live planet pipeline modules
are standalone WGSL. `sky_atmosphere.wgsl` uses Bevy imports and material
substitutions, so the standalone validator explicitly defers it. The final
native captures supplied real Bevy composition/backend validation of its
sixteen view/four light sample implementation. See [shader-port.md](shader-port.md).

## Physical flight evidence

Both runs used the same `ShipController` and Avian integration as manual flight.
The tour requested a great circle at approximately 1,000 m sea-level altitude.
Completion accumulated changes in actual physical position; no assigned orbital
poses or elapsed-time substitute drove the ship.

| Metric | Default great circle, S8 | Polar great circle, S8 |
| --- | ---: | ---: |
| Completed angle | 360.093° | 360.093° |
| Simulated duration | 56.300 s | 56.300 s |
| Minimum terrain/ocean clearance | 657.284 m | 567.243 m |
| Maximum speed | 600.000 m/s | 600.000 m/s |
| Maximum commanded acceleration | 80.000 m/s² | 80.000 m/s² |
| Maximum angular speed | 0.120 rad/s | 0.120 rad/s |
| Maximum commanded angular acceleration | 0.017 rad/s² | 0.017 rad/s² |
| Emergency terrain corrections | 0 | 0 |

The two headless circumnavigations and the two rendered tours independently report
the same completed angle, duration, clearance, speed, and zero emergency
corrections. The default path reaches approximately ±28.6° latitude; the polar
run physically crosses both poles.

A failed run exposed a real boundary bug: f32 velocities a few micrometres per
second above 600 caused the original f64 limiter to discard all steering.
The repaired limiter preserves steering within the reachable speed/acceleration
constraints. Regression tests cover that boundary and genuine lower-speed mode
changes. The repaired actual Avian runs produced the successful results above.

## Final scene and capture acceptance

The current S8 source generates **655,362 surface columns, including twelve
pentagons**, around a planet with a 4,000 m sea-level radius. Cell spacing is
approximately 19 m, with 6 m height terraces. Positive terrain elevations have
been reduced; the sampled peak is approximately 426 m, not a proven global
upper bound. This remains a coarse radial surface preview, not editable
metre-scale volumetric voxels.

The immutable 128-byte column records occupy approximately 80 MiB of GPU
storage, with approximately 2.5 MiB of visible column IDs per view. CPU-generated
topology is uploaded once. GPU compute compacts visible IDs and publishes an
indirect draw; vertex pulling creates caps, exposed skirts, and cosmetic trees.
There is no expanded terrain vertex/index upload per frame. The steady-state
planet path updates a 112-byte view/time uniform per view; this is not a claim
that the whole Bevy application uploads only 112 bytes.

| Final S8 scene / image | Wall-frame p50 (ms) | p95 (ms) | p99 (ms) | Samples |
| --- | ---: | ---: | ---: | ---: |
| [Orbit](screenshots/pbd-orbit.png) | 2.14 | 3.73 | 5.16 | 539 |
| [Coast](screenshots/pbd-coast.png) | 2.09 | 3.48 | 4.37 | 539 |
| [Surface](screenshots/pbd-surface.png) | 2.78 | 4.93 | 7.19 | 539 |
| [Night](screenshots/pbd-night.png) | 1.88 | 2.76 | 3.27 | 539 |
| [Pole](screenshots/pbd-pole.png) | 1.84 | 2.72 | 3.03 | 539 |
| [Default physical tour](screenshots/pbd-tour.png) | 2.35 | 3.86 | 5.22 | 3,539 |
| [Polar physical tour](screenshots/pbd-tour-polar.png) | 1.81 | 16.77 | 17.85 | 3,539 |

Static captures requested frame 600; both tour captures requested frame 3,600.
Capture mode uses fixed simulation steps and disables the interactive frame
pacer. The application measures elapsed wall time between frames, excludes
its first 60 samples, and excludes the capture-request frame and all later
samples. PNG capture/readback/save overhead is therefore excluded from these
percentiles. **GPU timestamp queries were not collected.** These are wall-frame
measurements of CPU work and synchronization, not isolated GPU execution times.

The polar tour's 16.77 ms p95 differs substantially from its 1.81 ms median.
Desktop focus/presentation scheduling may contribute, but these logs do not
identify the cause. This single-adapter capture set does not prove a production
60 FPS guarantee or the relative speedup from GPU geometry. Logged S8 CPU world
generation took 0.90–1.22 seconds across these runs.

## Art and remaining scope

All fourteen generated originals are **1254 × 1254**, rather than the requested
1024 × 1024, and remain flagged for production pixel-grid normalization.
Dimensions and hashes are in
[generated-manifest.json](../assets/tilesets/generated-manifest.json).
Visual inspection found distinct palettes and a four-by-four material layout;
strict 16 × 16 logical pixels and opposite-edge seamlessness are not certified.
The live renderer currently uses `fields.png` with biome palettes; it does not
bind all fourteen sheets as an independent runtime material array.

The live ocean, atmosphere, local skylight approximation, and surface renderer
are integrated prototypes. Editable volumetric terrain, chunk streaming,
propagated sky/RGB lighting, durable saves, full terrain/tree/cave colliders,
station walking, multiple playable planets, multiplayer, and complete origin
rebasing remain unimplemented. The height-shell safety guard is not a landing
or voxel-collision system. Standalone optical water and voxel meshing/light
kernels are not automatically active because their validation passed.

The [engine architecture](../openspec/changes/voxel-engine-foundation/design.md),
[flight controls](flight-controls.md), and
[game design acceptance gates](game-design.md#17-milestones-and-acceptance-gates)
describe the implementation boundaries and remaining work.
