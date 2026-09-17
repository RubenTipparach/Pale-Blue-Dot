# Handoff implementation validation

This pass implements the handoff's independent gravity, body-frame, visibility,
foliage-submission and night-rim changes. The preview scale, LOD seams, RON body
assets, dedicated water pipeline and voxel engine remain separate work.

## Reproducible checks

Run from the repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --release --locked --lib
cargo test --release --locked -p pbd-app --lib actual_gpu_visibility -- --ignored --nocapture
cargo run --locked --manifest-path tools/validate_shaders/Cargo.toml
npx --yes @fission-ai/openspec@1.13.0 validate --all
```

The GPU test is explicitly enabled because ordinary unit-test runs must also
work without a graphics adapter. It dispatches the shipped visibility shader
against the production binding layout and reads the resulting draw records and
cell IDs only in the test harness. Runtime terrain authority has no readback.

Coverage includes finite and infinite reverse-Z clip volumes, screen-edge
hexagons and pentagons, cliff bottoms, independently visible tree crowns,
material/seed selection, the strict 2,300 m foliage cutoff, undersized buffers,
dispatch tails, and joint camera/body translation. The sky ECS regression checks
the shared frame after subtracting astronomical system coordinates in f64.

## Capture setup

Before-edit executable, assets, and surface/night/orbit images are retained in
`output/handoff-baseline/`. Runtime asset loading still points at the compiled
repository path: the copied executable alone is not a frozen baseline.

Use 1440 by 900, seed `0x5eed2026`, a fixed 1/60 s timestep, and 240 frames.
Compare the central scene between HUD bars, rows 94 through 751. Offset captures
use `--render-offset 16384 -8192 32768`; camera orientation is calculated before
translation. Floating-point rasterization can change a few boundary pixels.

Machine: Intel Core i7-9700F, NVIDIA GeForce RTX 3070, driver 32.0.15.9597,
32 GiB system RAM. Background capture wall timings are smoke-test output, not a
controlled performance comparison. No speedup is claimed by this change.

GPU storage remains 128 bytes per terrain column, uploaded once (80 MiB for
655,362 columns). Each view now has two 2.5 MiB cell-ID lists and 32 bytes of
indirect arguments, with a 112-byte view/time uniform update. These are actual
allocation sizes from the implementation, not measured transfer counters.

The camera's infinite reverse-Z projection has no finite far clip plane encoded
in its matrix. Visibility follows that clip volume; this change does not add a
separate distance cap for terrain.

## Gameplay scale caveat

At 1 g, gravity is now 25 m/s^2 and a 12 m/s jump reaches about 2.88 m. The
preview still has six-metre elevation steps, so an entire terrace is taller than
the jump. One-metre cells require the separately planned LOD and rescale work.

The actual 60 Hz Avian walker measures one metre at 0.467 s with the old
9 m/s^2 versus 0.283 s with 25 m/s^2. Six metres measures 1.150 s versus
0.700 s. These are discrete simulation results; the analytical six-metre
prediction at 25 m/s^2 is 0.693 s.

## Automated results, 2026-09-17

The release library suite passes all 60 ordinary unit tests (38 app, 22 core),
including walking/jumping, hover, both circumnavigations, and shared gravity.
Formatting, Clippy with warnings denied, all seven standalone WGSL modules,
the validator's three tests, and all 14 OpenSpec items pass. The desktop release
executable builds successfully with the repository's normal ThinLTO settings.
The separately enabled GPU regression also passes on the RTX 3070 (Vulkan),
including the infinite reverse-Z fixture.

## Capture results, 2026-09-17

All five release captures exited successfully with no renderer warnings or
errors. The actual surface and Bevy-composed sky pipelines ran on Vulkan.
The surface scene region is pixel-identical before and after the rendering
changes. Night and orbit differences include the intended restored rim floor;
night-rim owner acceptance remains open in the shader-parity change.

| Joint camera/body translation | Identical scene pixels | Mean RGB error, 0-255 |
| --- | ---: | ---: |
| Surface | 99.9493% | 0.001090 |
| Night | 99.9555% | 0.000526 |

Both pass the comparison tolerance set before capturing: mean RGB error at
most 0.5, 99th-percentile maximum-channel pixel error at most 4, and at most
0.25% of pixels differing by more than 16. The 99th-percentile error was zero
for both. The small remaining differences are consistent with bounded f32
rasterization precision; the test does not claim arbitrary offsets are
bit-identical.

Local results: [surface](../output/handoff-after/surface.png),
[night](../output/handoff-after/night.png),
[translated surface](../output/handoff-after/surface-offset.png), and
[numeric comparison](../output/handoff-after/comparison.json).
