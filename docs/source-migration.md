# Source migration and implementation ledger

This is a selective redesign of Tenebris's Rust game using swarm-demo's Bevy
and GPU architecture. It is not a wholesale copy of either project, and a WGSL
port is not equivalent to a working Bevy render pass. The user's current request
supersedes historical OpenGL-only and rocket-orbit constraints.

## Pinned references

| Repository | Inspected commit | Relevant root |
| --- | --- | --- |
| [RubenTipparach/tenebris](https://github.com/RubenTipparach/tenebris) | `ef9865166349c7a3c4ae09acdd20640daf85b4fd` | `tenebris-rs/` |
| [RubenTipparach/swarm-demo](https://github.com/RubenTipparach/swarm-demo) | `625596b437fec454412b3c7083a3a04e510617f1` | `crates/swarm_core/`, `crates/swarm_app/` |

The optional shallow `.reference/` checkouts are research inputs only. Runtime
assets, shaders, and builds must be self-contained in Pale Blue Dot. Links below
use exact commits so future upstream edits cannot change the design evidence.
Tenebris's Rust workspace declares MIT and swarm-demo declares MIT OR Apache-2.0
in their Cargo metadata. Preserve attribution and any notices for copied
material; this ledger does not relicense upstream art or third-party content.
No upstream music, models, or entire texture libraries are imported by this pass.

## CLAUDE.md to AGENTS.md adaptation

The local [AGENTS.md](../AGENTS.md) is intentionally shorter than either source
file. Their accumulated bug history is useful evidence, not a second set of
live instructions or a reason to copy obsolete paths and approval workflows.

| Source rule | Adaptation here | Reason |
| --- | --- | --- |
| Tenebris: one shared core path; search before recreating a feature | Preserve in engine-independent core/adapters | Avoid divergent SP/MP rules and duplicate state |
| Tenebris: quaternion-backed free flight | Preserve quaternion authority | Prevent roll loss and pole singularities |
| Tenebris: stations are local reference frames | Preserve; specify analytic anchors and frame transforms | Jumps should return to the deck |
| Tenebris: world edits persist immediately | Preserve immediate transaction participation; define durability acknowledgement | Same-frame queued I/O is not proof of a crash-safe commit |
| Tenebris: ship must remain after exit | Preserve persistent ship identity and occupancy | Dismount does not delete world state |
| Tenebris: water camera is planet-local | Preserve across all related shader inputs | Prevent offset-world fog/Fresnel failures |
| Tenebris: planet-specific content and airless-world gates | Preserve explicit rosters/config gates; use this game's catalog | Prevent accidental inherited flora, fauna, weather, and material fallbacks |
| Tenebris: readable shader sources and real interface validation | Use WGSL files, Naga validation, then actual wgpu pipeline tests | Parse success alone does not prove bindings, attachment state, or visuals |
| Tenebris: frozen C/JS archives | Treat all reference checkouts as read-only research | New code and copied assets live in this workspace |
| Tenebris: YAML knobs and zero-sentinel fallback | Keep data-driven tuning; replace implicit zero with typed optional overrides | Zero is meaningful for gravity, emission, density, damping, and atmosphere |
| swarm-demo: pure core / Bevy harness boundary | Preserve with `pbd-core` and `pbd-app` | Simulation contracts stay testable without a renderer |
| swarm-demo: fields/buffers rather than a million entities | Apply to terrain cells/faces and cosmetic particles | ECS operates at chunks, bodies, and actors |
| swarm-demo: one simulation clock, narrow queries, stable authority | Preserve | Reproducible fixed-step behavior and useful ECS parallelism |
| swarm-demo: measure before declaring a speedup | Preserve named scenes, hardware, percentiles, and baselines | Avoid mistaking a design assumption for measured performance |
| swarm-demo: all state is f32 | Override with f64 global/orbit and f32 local physics/rendering | Kilometer planets and moving celestial frames require explicit precision |

Source instructions:
[Tenebris CLAUDE.md](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/CLAUDE.md),
[swarm-demo CLAUDE.md](https://github.com/RubenTipparach/swarm-demo/blob/625596b437fec454412b3c7083a3a04e510617f1/CLAUDE.md).

Excluded instructions include desktop GL 4.1/Sokol-only rendering, old C paths
for tuning data, C/JS byte-for-byte parity, prescribed keyboard bindings, old
casino/economy/RTS features, repository-specific push/PR automation, mandatory
mockup/user-confirmation gates, punctuation bans, and historical unresolved
bugs copied as if already present here. Their underlying visual/collision
regressions become focused validation cases where relevant.

## Source-to-target map

| Upstream source | Reuse decision | Destination/status |
| --- | --- | --- |
| Tenebris [`goldberg.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/goldberg.rs) | Retain icosahedron dual, 12 pentagons, ordered edge neighbors; replace eager whole-globe storage and sequential array IDs | Production spherical topology is specified in architecture; current CPU hex module is a local axial patch only |
| Tenebris [`world.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/world.rs) | Retain radial voxel columns, material authority, edit invalidation; replace fixed dense 128-layer globe | Sparse chunk store, radial slabs, journaling, and streaming remain proposed |
| Tenebris [`planet_gen.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/planet_gen.rs) | Use source climates and planet-specific generation as design evidence | GDD/biome atlas catalog; current seeded sampler is a small new foundation, not the full generator port |
| Tenebris [`planet.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/planet.rs) | Preserve body identity, local gravity query, atmosphere/tileset concepts | New body/config contracts; no wholesale global Mutex table import |
| Tenebris [`orbit.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/orbit.rs) | Preserve analytic clock-derived planet/moon rails and spin concepts | New core circular-rail sampling for planet/moon/station anchors |
| Tenebris [`station.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/station.rs) | Preserve dock-relative motion and deck-frame behavior; decouple rail speed from ship orbital physics | Full rotating station/deck implementation is future work |
| Tenebris [`rocket.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/rocket.rs) and flight client code | Retain only useful assembly/persistence concepts when needed; reject rocket orbit mechanics | New bounded Newtonian flight policy and Avian adapter |
| Tenebris [`world_light.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/world_light.rs) | Adapt daylight seeding, occluders, local propagation, invalidation | `voxel_light.wgsl` prototype uses sky/RGB ping-pong propagation; CPU gameplay-light authority still to integrate |
| Tenebris [`hex_mesher.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-client/src/hex_mesher.rs) | Retain cap/side semantics, edge-aligned neighbor sampling, packed material and light meaning | New GPU face records and vertex pulling; CPU expanded meshes are not the target hot path |
| Tenebris [`hex.fs.glsl`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-client/src/shaders/hex.fs.glsl) | Adapt pixel albedo, directional/baked light, local frame inputs | `hex_terrain.wgsl`; full wetness/crack/weather parity is not claimed |
| Tenebris [`water.fs.glsl`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-client/src/shaders/water.fs.glsl) and [`water.vs.glsl`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-client/src/shaders/water.vs.glsl) | Port useful optical/wave equations and explicit parameters into WGSL; adapt clip-depth/texture conventions | `water.wgsl`; opaque scene inputs, material binding, render ordering, underwater capture remain integration work |
| Tenebris [`atmosphere.fs.glsl`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-client/src/shaders/atmosphere.fs.glsl) | Translate ray/shell intersection and single-scatter atmosphere model | `atmosphere.wgsl`; Bevy composition and visual parity remain unverified |
| Tenebris [`assets/config`](https://github.com/RubenTipparach/tenebris/tree/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/assets/config) | Retain subsystem/per-body tuning, units, and artist ownership | New typed manifests; not the historical zero-sentinel parser |
| swarm-demo [`swarm.rs`](https://github.com/RubenTipparach/swarm-demo/blob/625596b437fec454412b3c7083a3a04e510617f1/crates/swarm_app/src/swarm.rs) | Reuse render-world ownership, extracted small inputs, persistent buffers, compute-before-draw lifecycle as architecture | Proposed Bevy terrain plugin; no RTS swarm implementation imported |
| swarm-demo [`swarm.wgsl`](https://github.com/RubenTipparach/swarm-demo/blob/625596b437fec454412b3c7083a3a04e510617f1/crates/swarm_app/assets/shaders/swarm.wgsl) | Learn from compute field/particle dispatch organization | New terrain compute shaders; particle motion is not terrain geometry code |
| swarm-demo [`mesh.rs`](https://github.com/RubenTipparach/swarm-demo/blob/625596b437fec454412b3c7083a3a04e510617f1/crates/swarm_core/src/mesh.rs) | Retain dirty-brick halos and exposed-face principle | Do not copy cubic greedy merging directly to curved hex wedges |

The original lighting header calls its process a six-neighbor BFS, but the
Goldberg domain has 5/6 horizontal neighbors plus vertical adjacency. The new
GPU interface encodes two radial slots and six possible side slots explicitly.
This distinction matters at pentagons and must not be flattened during a port.

## What is present in this delivery

| Artifact | Concrete scope | Does not yet establish |
| --- | --- | --- |
| [AGENTS.md](../AGENTS.md) | Adapted contributor invariants and source boundaries | Upstream CLAUDE files becoming live instructions |
| [Game design](game-design.md) and [engine architecture](engine-architecture.md) | Product vision, catalog, systems, budgets, interfaces, milestones | A complete playable multi-planet game |
| `crates/pbd-core/src/hex.rs` | Local pointy axial coordinates, prism and chunk-address foundations | Global Goldberg seams, pentagons, production streamed topology |
| `crates/pbd-core/src/terrain.rs` | Deterministic CPU terrain sampling foundation | Tenebris's full climate, caves, flora, resource, and hydrology generator |
| `crates/pbd-core/src/frame.rs` | f64 origin/velocity conversion and rebase math | Rotating frame/deck physics or production render-origin synchronization |
| `crates/pbd-core/src/orbit.rs` | Time-sampled circular hierarchical rails for allowed anchor types | A complete celestial-body scene and station gameplay |
| `crates/pbd-core/src/flight.rs` | Gravity/control and bounded velocity/acceleration contracts | Validated player flight feel or collision-complete spaceship gameplay |
| `crates/pbd-app/` | Bevy/Avian headless foundation using local physics and summed prescribed gravity wells | A windowed renderer, GPU terrain pipeline, selected-gravity-owner hysteresis, or rotating body-following frames |
| `assets/shaders/*.wgsl` and [shader-port.md](shader-port.md) | Readable shader ports/prototypes with interface documentation | Render-graph integration, working scene-color/depth bindings, or measured visual parity |
| `assets/tilesets/` | Per-biome pixel-art source sheets and manifest | Automatically production-ready seamless materials or a wired Bevy texture-array importer |

Run/validation commands and actual results belong in the root README and the
shader port report. Do not convert a planned acceptance gate into a passing
result by documenting it. The core's local axial topology and GPU shader's
explicit spherical corner/neighbor input are deliberately different stages;
a real topology provider is still needed to connect them.

## Port sequence and exclusions

1. Keep the pure core/Avian tests passing while completing canonical spherical
   IDs and one CPU-resident patch with actual colliders and a durable edit journal.
2. Implement Bevy extraction, GPU page allocation, bind-group layouts, dispatch
   ordering, indirect draws, revision publication, and submission retirement.
3. Feed the existing WGSL contracts from real CPU topology/occupancy and compare
   a tiny rendered patch against an independent exposed-face reference.
4. Connect the biome material manifest and GPU lighting; implement invalidation
   and cross-chunk halos before increasing visible world size.
5. Integrate opaque scene color/depth, water, atmosphere, and airless-body gates;
   test surface/orbit/underwater views with an offset planet.
6. Add spherical streaming/LOD, extend durable entity/fluid edits, station-local
   walking, orbital anchor visuals, and travel among the catalog bodies.
7. Profile native hardware and a real browser separately, then expand content
   and multiplayer from the same authoritative rules.

Ship Kepler propagation, rocket ascent/transfer planners, orbital energy
requirements, and maneuver nodes are excluded. Planet/moon/station rails are
retained. Old saves are not silently compatible with new chunk/cell IDs: an
explicit importer must map source body/tile/depth IDs through a versioned
topology and material conversion. No importer is included in this scaffold.

## Official compatibility references

Verified against pinned primary sources on 2026-09-15:

- [Avian 0.6.1 compatibility table](https://github.com/avianphysics/avian/blob/v0.6.1/README.md#version-table): Bevy 0.18 is supported by Avian 0.5-0.6.
- [Avian 0.6.1 documentation](https://docs.rs/avian3d/0.6.1/avian3d/): ECS physics, f32/f64 features, default fixed scheduling, and interpolation.
- [Bevy 0.18.1 render manifest](https://github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_render/Cargo.toml): wgpu/Naga 27 and native/WebGPU backend features.
- [Bevy 0.18.1 compute example](https://github.com/bevyengine/bevy/blob/v0.18.1/examples/shader/compute_shader_game_of_life.rs): render-world compute pipeline lifecycle.
- [wgpu 27.0.1 feature definitions](https://github.com/gfx-rs/wgpu/blob/v27.0.1/wgpu-types/src/features.rs): capability negotiation, optional indirect first instance, and native-only indirect-count support.

The architecture's page allocator, LOD stitching, control envelopes, streaming
budgets, and migration plan are design proposals inferred from those capabilities
and the inspected source. They are not features supplied automatically by Bevy,
Avian, or swarm-demo.
