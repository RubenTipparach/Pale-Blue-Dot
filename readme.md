# Pale Blue Dot

A hexagonal-voxel exploration and building game across kilometer-scale planets.
Rust + Bevy 0.18.1 + Avian 0.6.1, with a GPU terrain architecture based on WGSL
compute and resident face buffers.

This repository contains the **design and initial engine foundation**. It does
not yet render a playable solar system. Planets, moons and stations use on-rails
motion; spacecraft use gravity falloff, bounded Newtonian flight and optional
inertial/rotational dampeners.

## Start here

- [Comprehensive game design](docs/game-design.md): worlds, all 14 biomes, flight,
  construction, survival, progression, persistence, performance and milestones.
- [Engine architecture](docs/engine-architecture.md): spherical topology,
  precision, ECS boundaries, Avian scheduling, streaming and GPU geometry.
- [Biome art gallery](docs/biome-atlas.html): offline interactive browser for
  14 generated tilesheets, each containing 16 material studies.
- [Shader ports](docs/shader-port.md): actual WGSL assets, source mappings,
  buffer layouts and remaining renderer integration.
- [Source migration](docs/source-migration.md): pinned upstream commits,
  adapted CLAUDE.md rules, reuse decisions and implementation status.
- [AGENTS.md](AGENTS.md): contributor instructions for this repository.
- [Validation record](docs/validation.md): commands, results and tested limits.

## Run the foundation

Install a Rust toolchain meeting the workspace's minimum Rust version and the
native C/C++ linker tools for your platform. The checked-in lockfiles pin the
resolved dependencies. The app is headless and requires no window or GPU.

```sh
cargo run --locked -p pbd-app -- 600
```

This runs a finite Bevy/Avian simulation and prints flight telemetry, on-rails
body count and a deterministic terrain checksum. It demonstrates the scheduling
and control foundation; it does not load the tilesets or execute the WGSL passes.

```sh
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
python tools/build_art_catalog.py
```

The shader validator has its own manifest; its commands and validation scope
are documented in [shader-port.md](docs/shader-port.md). The art catalog builder
uses Python's standard library and reads PNGs unchanged. Open the generated
`docs/biome-atlas.html` directly in a browser; no web server is required.

## What is implemented

`pbd-core` supplies deterministic local axial terrain sampling, chunk addressing,
f64 frame conversions, hierarchical circular rails and bounded ship control.
`pbd-app` supplies a fixed-step Bevy/Avian adapter and finite smoke executable.
Five standalone WGSL modules cover hex/pentagon face extraction and drawing,
voxel light propagation, water and atmosphere. Their source, binding and memory
contracts are validated separately from the headless app.

The art pack contains 14 generated source atlases with 224 named material slots,
the [exact prompts](assets/tilesets/prompts.json), and an asset manifest. These
used the built-in image_gen tool. Production pixel-grid normalization, tiling
approval and runtime texture-array import remain art-integration tasks.

The next implementation gates are the hierarchical spherical topology, streamed
chunk/collider store, actual Bevy render-world pipelines and GPU runtime tests.
The [milestone table](docs/game-design.md#17-milestones-and-acceptance-gates)
then sequences editable worlds, ships/stations, production and cooperative play.
Performance budgets are design targets; no GPU speedup or FPS benchmark is
claimed by the current headless foundation.

## References

The redesign draws from [Tenebris](https://github.com/RubenTipparach/tenebris)
and [swarm-demo](https://github.com/RubenTipparach/swarm-demo). Reference clones
under `.reference/` are ignored research inputs. Builds and committed assets do
not depend on those clones or on sibling repositories.
