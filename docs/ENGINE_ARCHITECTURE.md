# Tenebris Engine Architecture

## 1. Purpose and scope

This document redesigns Tenebris around Bevy's ECS and Avian physics. It defines boundaries rather than prematurely fixing every implementation detail. The design must support:

- seamless travel among several-kilometer procedural bodies;
- hexagonal-prism voxel terrain, editing, construction, and persistence;
- local, responsive characters, vehicles, ships, and projectiles;
- deterministic on-rails planets, moons, and stations;
- assisted Newtonian ship flight, not player-ship orbital mechanics;
- bounded memory, CPU, GPU, disk, and network work.

### Architectural invariants

1. **Astronomical truth is not physics truth.** An `f64` ephemeris locates bodies; a rebased `f32` scene drives rendering and local physics.
2. **The voxel database is not the ECS.** Chunks and edits are data assets; only active representatives become entities.
3. **Generation is reproducible.** Coordinates and versioned seeds determine base content; player edits are a sparse overlay.
4. **Streaming is budgeted.** Visibility never implies immediate generation, meshing, collider creation, or upload.
5. **Simulation authority is explicit.** Rails own celestial transforms, Avian owns local contacts, and gameplay systems own rules such as damage and flight assists.

## 2. Workspace topology

Use a Cargo workspace with dependencies flowing downward:

| Crate | Responsibility | Must not know about |
|---|---|---|
| `tenebris_app` | executable, platform setup, plugin composition | generation internals |
| `tenebris_core` | IDs, units, time, seeds, coordinate types, errors | Bevy rendering |
| `tenebris_ephemeris` | rail trajectories, frames, SOI selection, time sampling | Avian |
| `tenebris_worldgen` | deterministic fields, biomes, features, chunk materialization | ECS entities |
| `tenebris_voxel` | chunk store, edit overlays, LODs, meshing interfaces | gameplay rules |
| `tenebris_sim` | gameplay components, flight controllers, health, interaction | platform UI |
| `tenebris_physics` | Avian adapters, physics bubbles, collider cooking | world-generation policy |
| `tenebris_render` | terrain materials, atmosphere, impostors, debug views | save authority |
| `tenebris_net` | replication, interest management, prediction contracts | rendering |
| `tenebris_persistence` | manifests, schema migration, edit/event storage | presentation |
| `tenebris_tools` | headless generators, inspectors, benchmarks, validation | shipping-only assumptions |

Begin with fewer physical crates if compilation overhead impedes iteration, but keep these module boundaries and dependency rules.

## 3. Bevy application model

### States

`AppMode` progresses through `Boot`, `MainMenu`, `LoadingWorld`, `InGame`, and `ShuttingDown`. `SessionMode` distinguishes single-player, client, listen server, and dedicated server. Loading is a tracked set of prerequisites, never a fixed timer.

### Plugin groups

Recommended composition:

- `FoundationPlugins`: diagnostics, settings, asset sources, task pools, clocks.
- `UniversePlugins`: catalog, ephemeris, reference frames, floating origin.
- `WorldPlugins`: generation, voxel storage, streaming, meshing, persistence.
- `SimulationPlugins`: Avian, characters, ships, interaction, combat.
- `PresentationPlugins`: cameras, terrain rendering, atmosphere, UI, audio.
- `NetworkPlugins`: transport, authority, snapshots, prediction, interest.

### Ordered system sets

Fixed simulation follows this order:

1. `SampleEphemeris`
2. `SelectReferenceFrames`
3. `RebaseIfNeeded`
4. `CollectControlIntent`
5. `ComputeForces`
6. `ApplyFlightAssists`
7. `PhysicsStep`
8. `ResolveGameplayContacts`
9. `CommitAuthoritativeState`

Streaming uses a separate pipeline: `Observe` → `Prioritize` → `Request` → `GenerateOrLoad` → `Mesh` → `CookCollider` → `Upload` → `Activate` → `Retire`. Each transition carries a generation/request epoch so late async results cannot resurrect stale chunks.

## 4. Coordinates, time, and reference frames

### Typed spaces

Define distinct types rather than aliases:

- `BarycentricPosition(DVec3)`: universe-scale inertial location.
- `BodyFixedPosition(DVec3)`: location relative to a rotating body's center.
- `HexCell { q, r, layer }`: canonical integer terrain address.
- `ChunkKey { body_id, face_or_region, cq, cr, cz, lod }`: stable storage/streaming address.
- `LocalPosition(Vec3)`: rebased render/physics coordinate.
- `SimInstant(i64)`: integer ticks since the save epoch.

All conversion APIs require a sampled `FrameTransform` and time. Unit tests cover round trips, boundaries, poles/seams, negative coordinates, and large magnitudes.

### Floating origin and physics bubbles

The focus entity stays close to the local origin. A rebase changes the mapping between astronomical and local space; it does not move astronomical truth. Apply a rebase at a fixed-step boundary, shift all local entities in one operation, invalidate interpolation history, and notify particles/audio/cameras.

One authoritative physics bubble surrounds the active player group. Far objects are represented analytically or at low frequency. Multiplayer servers may maintain multiple isolated bubbles keyed by interest cluster; objects crossing clusters undergo an explicit handoff. There is no planet-sized Avian world.

## 5. Celestial on-rails simulation

`CelestialBody` holds stable ID, parent, physical parameters, spin model, rail trajectory, epoch, and visual/streaming metadata. A trajectory may be Keplerian, circular, spline-authored, or fixed in a parent frame, but always supports deterministic `sample(time) -> pose + linear/angular velocity`.

Planets, moons, and stations are kinematic astronomical anchors. Their full bodies are never Avian rigid bodies. Near a surface or station, sampled rail velocity becomes the local frame velocity so a parked actor remains stable. Save files store rail definitions and epoch, not integrated per-frame transforms.

Rails must provide continuity validation, hierarchy-cycle detection, reproducible seeking, and debug visualization. Time warp, if later added, advances rails and coarse simulation only; it must be unavailable while local dynamic contacts require fixed stepping.

## 6. Ship flight and gravity

### Deliberate departure from the prototype

Player ships do not use transfer nodes, patched conics, or an orbit propagator. Flight is direct thrust under gravity, made playable by assistance. A ship can coast and conserve momentum when assists are disabled, but ordinary controls target comprehensible motion.

### Gravity model

Each relevant body supplies a gravity field. Use inverse-square falloff outside a configurable near-surface transition and a designer-safe interior/near-center function. Blend overlapping influences without discontinuities; cull insignificant contributors by a deterministic threshold. Surface gameplay may use a cached local `up` and gravity vector, updated at fixed cadence.

The gravity equation is conceptually `a = -mu * r / |r|^3`, with softening or an interior model where necessary. Gameplay caps may limit harmful numerical extremes, but debug tooling must expose raw and applied values.

### Control pipeline

`PilotIntent` contains desired translation, rotation, boost/brake, and assist toggles. `ThrusterLayout` converts intent to achievable force/torque. `FlightEnvelope` declares maximum commanded acceleration, angular acceleration, assisted speed, and optional axis-specific limits. `FlightAssistState` contains controller integrals/history.

Per fixed tick:

1. Calculate environmental acceleration and current velocity relative to the selected local frame.
2. Allocate pilot-requested force/torque within thruster, power, heat, and envelope limits.
3. If inertial damping is enabled and input is inside its dead zone, command bounded counter-thrust toward the selected target velocity.
4. If rotational damping is enabled, command bounded counter-torque toward the selected target angular velocity/attitude.
5. When approaching the assisted speed limit, reduce outward commanded acceleration using a smooth response curve; never truncate velocity instantaneously.
6. Submit forces/torques to Avian and record saturation for HUD feedback.

Limits are controller constraints, not universal laws. Gravity, collisions, another vessel, or disabled assists may push a craft beyond its assisted limit. The controller brakes it using physically available authority. This preserves understandable Newtonian behavior and avoids discontinuous velocity clamps.

### Numerical and gameplay acceptance tests

- With gravity and assists off, velocity remains constant within integration tolerance.
- Damping never commands more than available acceleration or torque.
- Frame changes preserve barycentric position and velocity.
- Speed limiting is monotonic, has no one-tick discontinuity, and behaves consistently across fixed-step rates.
- A landed craft inherits the body's rail and rotational frame without jitter.
- Control remains stable at low frame rates because fixed simulation is independent of render rate.

## 7. Hex-voxel terrain

### Representation

The logical cell is a vertical hexagonal prism addressed by axial coordinates `(q, r)` and integer layer `z`; derive cube coordinate `s = -q-r` for distance and neighbors. Terrain is stored in chunks of columns and layers. Exact dimensions are selected through profiling; candidate dimensions must align with meshing, compression, and edit locality.

A cell stores a compact material ID plus optional flags. Metadata with low density—damage, ownership, inventory, growth—is stored in sparse side tables. Never create an entity per voxel.

### Planet mapping

A single infinite axial plane cannot tile a sphere without defects. Use a geodesic region graph (for example, subdivided icosahedral patches) whose interior exposes hex adjacency and whose unavoidable exceptional cells/seams have explicit neighbor transforms. All systems query topology through `PlanetGrid`; no gameplay code assumes `q + direction` works across a patch boundary.

For small prototype bodies, a locally flat bounded hex map is acceptable behind the same interface. The production representation must support kilometer-scale circumference, vertical caves, and a stable address for every editable cell.

### Generation stages

Generation is staged and seeded independently:

1. system and body catalog;
2. low-frequency tectonic/elevation/climate fields;
3. regional biome and watershed graph;
4. terrain density and stratigraphy;
5. caves and large structures;
6. resources;
7. settlements, ruins, and routes;
8. local flora, props, and encounters.

Global/regional fields are cached at coarse resolution. Materializing a chunk samples them and applies deterministic local detail. Cross-chunk features are planned in region space first, preventing clipped rivers, caves, roads, or structures.

### Meshes and colliders

Generate indexed surface meshes from exposed prism faces, merge compatible faces where practical, and create skirts/transitions between LODs. Render distant terrain from coarse meshes, height shells, or impostors. Collider meshes exist only near dynamic actors and are cooked asynchronously from a simplified, watertight surface. Collision activation waits for a safe fixed-step boundary.

Edits mark affected chunks and neighbors dirty. Debounce rebuilds, prioritize collision near actors over visuals, and retain the previous collider until its replacement is valid.

## 8. Streaming and budgets

Interest is the union of camera visibility, actor safety radius, movement prediction, mission pins, network relevance, and editor requests. A priority score includes distance, time-to-contact, on-screen error, gameplay criticality, and starvation age.

Track separate configurable budgets for resident voxel bytes, terrain mesh bytes, GPU uploads per frame, generation milliseconds, collider cooks, active rigid bodies, and save writes. Degrade in this order: decorative entities, texture/material detail, render LOD, simulation rate, then terrain radius. Never drop collision inside the safety envelope.

Chunk lifecycle:

`Absent → Requested → DataReady → MeshReady → ColliderReady → Active → Cooling → Evicted`

Pin reasons are reference-counted. Diagnostics show state, age, memory, task owner, pin reasons, and last transition for every resident chunk.

## 9. Persistence and versioning

A world manifest stores schema version, generator version, seed, catalog/rail definitions, simulation epoch, enabled content packs, and compatibility flags. Base terrain is regenerated; sparse modifications are stored as chunk-local edit journals plus periodic compact snapshots. Persist stable IDs, canonical coordinates, and intent—not Bevy entity IDs, local floating-origin coordinates, or raw component memory.

Generation algorithm changes require one of: exact old-version support, an offline migration, or an explicit incompatible-world boundary. Journals use checksums and atomic replacement. Recovery can discard an incomplete tail without losing the last valid snapshot.

## 10. Networking contract

The server owns rails, world edits, inventories, damage, and authoritative dynamic state. Clients predict their control intent and reconcile ships/characters. Rails and base generation are reconstructed from seed/version/time; the server transmits definitions, corrections, edit overlays, and relevant dynamic entities rather than all terrain.

Interest management keys off body, region/chunk, physics bubble, and mission relevance. A client must validate generator and content versions before joining. World-edit commands include expected revision to reject stale writes deterministically.

## 11. Rendering and presentation

- Near field: detailed terrain meshes, decals, vegetation, shadows, and atmosphere depth cues.
- Mid field: merged/coarse terrain and reduced entity simulation.
- Far field: body shells/impostors sampled from the same macro fields.
- Space: scaled astronomical representation separate from the local physics camera.

Blend representations with hysteresis and temporal fades. Depth strategy, atmosphere, clouds, and celestial scale must be validated early; standard single-camera depth cannot cover centimeters to interplanetary distance reliably.

## 12. Observability and tooling

Ship a debug UI for coordinate frames, rail paths, gravity vectors, assist targets/saturation, physics bubbles, chunk state/LOD, task queues, seeds, and generation timings. Headless tools must reproduce a chunk from a printed generation key and export topology/mesh validation results.

Required automated suites include:

- golden hashes for generation stages;
- property tests for hex neighbors and coordinate round trips;
- seam and exceptional-cell topology tests;
- rail seek/continuity tests;
- flight-controller fixed-step tests;
- mesh manifold/material-boundary tests;
- save migration and crash-tail recovery tests;
- streaming cancellation and budget stress tests.

## 13. Delivery sequence

1. **Foundations:** workspace, plugins, typed units/coordinates, fixed clock, diagnostics, CI.
2. **Walking slice:** one flat hex region, streaming, mesh/collider cooking, character movement, edits and reload.
3. **Planet slice:** spherical topology, floating origin, macro generation, LOD shell, surface-to-altitude transition.
4. **Flight slice:** one rail planet and moon, gravity, assisted ship, landing, frame transitions, instrumentation.
5. **Persistence slice:** versioned manifest, edit journal, deterministic reload and migration harness.
6. **System slice:** multiple bodies and station rails, space rendering, multi-body streaming, save/load.
7. **Network slice:** dedicated authority, client prediction, seed/version handshake, edit replication.
8. **Content production:** settlements, ecology, progression, missions, balance, accessibility, optimization.

Each slice ends in a playable executable, a captured performance baseline, and updated risks. Do not build the full content pipeline before coordinate transitions, terrain seams, and ship controls feel correct.

## 14. Initial performance targets

These are profiling targets, not promises, and should be platform-tiered after the first slice:

- fixed simulation: 60 Hz, no accumulated spiral of death;
- foreground CPU: under 16.6 ms at 60 fps on the reference PC, with streaming jobs outside the critical path;
- main-thread streaming integration: configurable default of 2 ms/frame;
- no synchronous disk access, generation, meshing, or collider cooking during gameplay;
- bounded resident memory with visible high-water marks;
- deterministic headless generation of any chunk without loading its planet.

## 15. Decisions still requiring prototypes

- Avian dimensional/version choice and exact Bevy compatibility pin;
- spherical hex topology and exceptional-cell authoring UX;
- terrain meshing/LOD algorithm and transition treatment;
- multi-bubble Avian strategy on dedicated servers;
- gravity blending and local-frame handoff thresholds;
- persistence compaction format;
- target player count and authoritative tick/network budgets.

Resolve these through small measured prototypes and architecture decision records, not assumptions embedded across gameplay code.
