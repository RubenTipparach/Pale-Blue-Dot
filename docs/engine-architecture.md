# Engine architecture: spherical hex worlds in Bevy and Avian

Status: target architecture plus a desktop walking and flight prototype. The
subdivision-8 hex/pentagon planet, GPU visibility compaction/indirect drawing,
Avian flight, camera/HUD, and atmospheric shell have compiled and
rendered on native Vulkan hardware. The recorded flight baseline passed 35
release tests; its headless and rendered default/polar tours each completed a
physical 360-degree lap in 56.3 seconds. Those captures and wall-frame
measurements are preserved in the [validation record](validation.md).
No GPU timestamp measurements were collected,
and the prototype does not establish production-game performance.

The prototype is a surface-column explorer. It does not yet implement editable
meter-scale volumetric voxels, caves, streamed radial slabs, or durable edits.
The ledger below describes the implemented boundary; subsequent design
sections describe the production target unless explicitly marked as current.
See [flight controls](flight-controls.md) for preview controls and verification
commands, [source migration](source-migration.md) for pinned upstream provenance,
and [shader port](shader-port.md) for the standalone volumetric shader contracts.

## 1. Decisions and compatibility baseline

Build a new Bevy host around reusable Tenebris world concepts and swarm-demo's
separation between CPU rules and GPU presentation. Keep the pixel-art material
language, Goldberg hex/pentagon terrain, propagated voxel light, atmospheric
shells, and depth-aware water. Replace Sokol/OpenGL rendering and rocket orbital
simulation. Avian owns local contacts and rigid-body integration. The desktop
prototype compacts visible columns on the GPU and reconstructs their geometry
in the vertex shader. The target volumetric renderer will additionally generate
exposed faces from edited occupancy without uploading expanded vertices.

Use **Bevy 0.18.1 with Avian 0.6.1** as the initial compatible, pinned baseline.
This deliberately matches swarm-demo's Bevy release; it is not a claim that
0.18.1 is the newest Bevy version. Avian's pinned manifest depends on Bevy 0.18,
and Bevy 0.18.1 uses wgpu/Naga 27. Use Bevy's rendering re-exports and the lockfile
instead of introducing a second incompatible wgpu version.
Sources: [Avian manifest](https://github.com/avianphysics/avian/blob/v0.6.1/crates/avian3d/Cargo.toml),
[Bevy render manifest](https://github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_render/Cargo.toml).

Native Windows/Linux/macOS is the primary development target. A WebGPU browser
path is an architectural requirement and later integration milestone. WebGL is
not a compute backend. No native-only capability is required for the baseline.

| Boundary | Owner | Rule |
| --- | --- | --- |
| Stable IDs, procedural inputs, cell edits, flight policy | `pbd-core` | Independent of Bevy and GPU |
| Input, ECS scheduling, task orchestration, Avian adapters | `pbd-app` | Convert engine events into explicit core commands |
| GPU allocations, bind groups, pipelines, extracted changes | `pbd-app::planet` currently; volumetric chunk renderer later | Cannot mutate authoritative gameplay |
| Durable journal, snapshots, migrations | Persistence service, to be implemented | Serializes stable core data, never ECS entity IDs |
| Biomes, materials, ship limits, atmosphere/water settings | Validated asset manifests | Data versions and units are explicit |

Start with two crates; extract renderer, storage, or server crates when concrete
boundaries justify them. Do not create a crate per system. Pure deterministic
world rules can be shared by a future dedicated server; Avian collision outcomes
are server-authoritative, not assumed bit-identical across client hardware.

### Current implementation ledger

The ledger separates the implemented and exercised prototype from the
production target. Native evidence covers Windows 10, an RTX 3070 with NVIDIA
595.97/Vulkan, 1440 × 900, and seed `0x5eed2026`; broader platform, gameplay, and
performance acceptance remains part of the target work.

| Area | Implemented in the source tree | Production target still outstanding |
| --- | --- | --- |
| Desktop host | Window, camera/HUD, input, screenshots, graphical tours, and headless verification run on the recorded native device; launcher and actual-window control/screenshot smoke passed | Other devices/platforms, final user interface, and settings |
| Spherical topology | `planet_topology.rs` builds a complete level-8 icosahedron dual with 655,362 columns, including twelve pentagons and shared corner/neighbor data | Hierarchical persistent cell IDs, patch streaming, and fine/coarse LOD transitions |
| Terrain authority | `planet_terrain.rs` evaluates a deterministic spherical height/climate field on the CPU; 6 m elevation steps and compressed positive heights yield a sampled peak near 426 m | Editable 3D occupancy, radial materials, caves, overhangs, geology, and edit transactions |
| GPU geometry | `planet.rs`, `planet_visibility.wgsl`, and `planet_surface.wgsl` wire persistent 128-byte column records, horizon visibility compaction, GPU-written indirect arguments, and vertex pulling into Bevy | Volumetric exposed-face extraction, dirty-page updates, allocator retirement, streamed halos, and production culling/LOD |
| Preview materials | The active planet shader uses integer texel loads from the committed `fields.png` source atlas; explicit nearest/point image settings preserve pixels. Biome palettes, local texture detail, and decorative trees remain active | Independent production texture layers from all biome sheets, reviewed seams/mips, and full material/geometry variants |
| Preview lighting | CPU generation derives a local sky-occlusion factor from adjacent column heights; the surface shader adds a fixed sun/terminator and atmospheric treatment | Propagated sky/RGB light through volumetric occupancy, removal/invalidation, cross-chunk light exchange, and gameplay light authority |
| Preview ocean | The planet surface shader shades sea-level ocean columns with depth color, wave/specular/Fresnel cues, and atmospheric fade | Resolved-scene refraction, underwater transitions, the standalone `water.wgsl` pass, and authoritative local fluids |
| Preview atmosphere | Native captures exercise the Bevy material shell with sixteen view/four light samples, an 800 m air shell, and clouds at 600 m above sea level | Multi-body/airless configuration, offset/rebased bodies, production composition, and quality tiers |
| Flight | Input runs before physics; `ShipController` submits bounded accelerations and Avian integrates position/quaternion attitude once. Player view now reads raw mouse orientation immediately, independently of bounded physical attitude; no camera interpolation is applied | Fuel, damage, ship construction, station docking, full terrain colliders, and gameplay-ready landing |
| Walking | Default first-person walker uses Avian integration, exact CPU intersections with rendered hex caps, swept footprint/step support, jumping, and planet-relative up; F switches the active controller | Swimming, cave/overhang/tree colliders, interaction-based boarding, and edited terrain collision |
| Flight protection | A sampled segment check enforces 1.6 m manual clearance against exact rendered cap heights; automated tours retain the conservative radial height/ocean field and 45 m clearance | Detailed collision for voxel edits, caves, overhangs, trees, and streamed collider replacement |
| Celestial motion | Core supports analytic rails; the desktop scene includes a decorative moon on a rail and a static origin planet for flight | Planet spin/orbital reference-frame transitions, walkable stations, and multi-planet travel |
| Validation | The recorded flight baseline passed 35 release tests, strict Clippy/formatting, seven standalone WGSL modules and three validator regressions; five static scene captures, two rendered full tours, and actual-window flight/screenshot evidence are preserved | Production acceptance cases below, GPU timestamp profiling, repeated comparative benchmarks, and other hardware/backend tiers |

At radius 4 km, 655,362 columns average about 307 square metres each, roughly
19 m between neighboring centers for an equivalent regular hex grid. Their
128-byte GPU topology records occupy about 80 MiB. They are
coarse surface columns, not the one-meter microvoxels specified below. A single
height per column cannot represent an interior cave or independently removable
blocks. The prototype eagerly constructs this modest whole-globe topology once;
that does not justify allocating an entire fine voxel planet.

The walking controller shares those CPU column records with the renderer;
explicit neighbor IDs add about 15 MiB and the contact seed index adds 24 KiB.
Its capsule footprint samples exact rendered cap triangles before resolving
ground contact. Dry-land walking defaults to 8 m/s, Shift sprint to 14 m/s,
a 0.6 m automatic step, and a 12 m/s jump (about 8 m at 9 m/s^2 gravity).
The eye sits 1.6 m above the feet, with a small contact skin. Walking preserves
planet-relative up and clamps pitch to +/-89 degrees. Water blocks entry until
swimming exists; trees, caves, and overhangs have no preview contact geometry.
F changes creative walking/flight mode while retaining the ship entity;
returning to walking places the player on dry ground below or nearby.

Both manual cameras apply raw mouse motion at 0.002 radians per pixel, without
easing or pose interpolation. Interactive presentation uses VSync and requests
a maximum frame latency of one from the backend; this is a queue hint, not a
measured end-to-end latency guarantee. There is no additional main-thread frame
pacer. Capture runs remain uncapped with VSync disabled.

The live GPU path uses a visible-ID buffer sized for every prototype column;
each compute invocation can append at most one ID. It performs horizon rejection
and indirect column drawing, not the standalone `hex_faces.wgsl` volumetric
mesher. Near views draw 162 vertices per visible column, including optional
decorative trees. Above 3,200 m sea-level camera altitude, indirect draws use
only the 54 terrain vertices: even summit trees are then beyond the shader's
2,300 m decoration range. Its topology remains on the GPU after the initial
upload; steady-state planet updates write a 112-byte view/time uniform per view,
and a roughly 2.5 MiB per-view visible-ID buffer remains GPU-resident. Measured
wall-frame results are in [validation.md](validation.md); no controlled CPU-mesh
comparison or isolated GPU timing establishes a relative throughput improvement.

Preview flight uses a 120 m/s normal target, 600 m/s cruise, **80 m/s^2 controlled
acceleration**, and approximately 1.6 m of manual terrain clearance. Automated
tours retain 45 m clearance. The 20 m/s^2 values later in this document remain
the original game-design tuning
proposal. At the preview tour's 5 km orbital radius, a 600 m/s circular path
needs 72 m/s^2 of centripetal acceleration. This is controller-driven movement
around the planet, not assigning an on-rails orbit to the ship. The tour's
completion checks accumulate angle from Avian's actual position. The final
default and polar paths each reached 360.093 degrees in 56.300 seconds, with
zero emergency terrain projections. Minimum clearance was 657.284 m and
567.243 m respectively. The recorded maximums were 600 m/s, 80 m/s^2,
0.120 rad/s, and 0.017 rad/s^2. These route results do not replace landing,
collision, or other gravity-transition tests.

The current preview uses a single origin-centered `f32` physics/render bubble.
The core includes `f64` position/frame math, but the desktop scene does not yet
exercise global rebasing or rotating station frames. Its moon and its own height
field are illustrative inputs, not a complete implementation of the catalog.

## 2. Scale, coordinates, and topology

### World scale

The catalog targets Tenebris at radius 4 km, Sequoia 6 km, Crag 3 km, Frostbite
2.5 km, moons 0.5-1.5 km, and asteroids 40-250 m. Landable planets therefore
span 5-12 km in diameter in the initial catalog; generation accepts 2-8 km
planet radii. The full surface is traversable. Distance between bodies is
separate authored data, tuned for assisted flight rather than astronomical scale.

Near-player radial layers are 1 m high. Target lateral cell area is roughly
1-4 square metres at the reference surface, depending on body subdivision.
Curvature makes cell dimensions vary; these are wedge-shaped radial prisms,
not congruent Euclidean hexagonal blocks. Build tools preview the actual cells.

For the upstream midpoint-subdivided icosahedron dual, `cells = 10 * 4^L + 2`.
At radius 4,000 m and level 12 that is 167,772,162 surface cells, averaging
`4*pi*R^2/cells = 1.198 m^2`. A dense 512-layer shell at even 3 bytes per voxel
would consume about **258 GB per planet**, before topology and meshes. The
engine must never allocate the whole detailed globe. Unvisited terrain is a
seed plus generation rules; resident chunks and persistent edits are sparse.

### Spherical identity

Preserve the mathematical construction in Tenebris
[`goldberg.rs`](https://github.com/RubenTipparach/tenebris/blob/ef9865166349c7a3c4ae09acdd20640daf85b4fd/tenebris-rs/crates/tenebris-core/src/goldberg.rs):
subdivide an icosahedral triangle mesh and take its dual. At a uniform level
there are exactly twelve degree-five pentagons, with degree-six cells elsewhere.
Pentagons are real terrain cells supported by traversal, meshing, lighting,
collision, and construction. A global all-regular-hex sphere is impossible.

The production index is a **new versioned hierarchical index**, not the old
array offset. Define canonical primal vertices by their subdivision ancestry
and exact integer barycentric coordinates on their owning icosahedron face.
Deduplicate face-edge and original-vertex aliases with precomputed oriented
face adjacency and a canonical minimum owner. A dual cell is the corresponding
primal vertex; its corners are projected adjacent triangle centers. Normalize
shared corner directions using one canonical calculation so both sides of a
chunk edge agree. Store ordered edge neighbors and reciprocal edge indices.

`CellId = (body_id, topology_version, canonical_vertex_id, radial_layer)`.
`ChunkKey = (body_id, topology_version, surface_patch, radial_slab)` groups
nearby cell IDs. The concrete packed bit layout is a topology milestone, not
an invented compatibility promise. Near-patch arrays may use axial coordinates
for local algorithms, but seam crossings must resolve through the canonical
neighbor table. A pentagon has five valid side slots; slot six is invalid.

Use a uniform fine topology for authoritative cells. Rendering LOD aggregates
regions of that topology; it does not rewrite saved cells. Coarse dual cells
are not exact unions of fine dual polygons, so do not assume naive hex parent
division produces watertight LOD. Build coarse triangle patches over the same
icosahedron hierarchy, with explicit shared-edge samples and transition strips.
Keep the fine side's boundary samples at the transition, morph only interior
vertices, and use a bounded skirt as a secondary gap guard. Cap neighbor LOD
differences to one level. Test all face seams and all twelve defects.

### Precision and reference frames

Store system position, orbit epoch, body attitude, and frame transforms in
`f64`. Use meters, seconds, radians, and quaternions. The GPU and the default
Avian world use local `f32` coordinates near the active player group. Compute
`(global_f64 - origin_f64)` before converting to `f32`; casting both absolute
positions first loses the precision the origin is meant to preserve.

For a frame with global origin `O`, orientation `Q`, origin velocity `V`, and
world-space angular velocity `omega`:

```text
x_world = O + Q * x_local
v_world = V + omega cross (Q * x_local) + Q * v_local
x_local = inverse(Q) * (x_world - O)
v_local = inverse(Q) * (v_world - V - omega cross (x_world - O))
```

Use these transforms when changing frames, including station approach and
departure. Rebase only at fixed-tick boundaries and shift current/previous
poses, collider locations, camera poses, any actor interpolation history,
joints, trails, cached bounds, and extracted render origins together. Do not
infer velocity from a rebased `Transform`. A render extraction carries one
origin epoch; mixed epochs cannot
share a draw. Quaternion basis vectors are derived views, never competing
orientation authorities.

One local physics world suffices for the initial single-player bubble.
Multiplayer groups far apart require independent simulation bubbles, each with
its own origin and broad phase, or a separately benchmarked f64 physics server.
Putting multiple planet-local bodies near zero in one unfiltered Avian world
would create false collisions. Bubble migration is a transaction; contacts and
joints cannot silently span unrelated frames.

## 3. Streaming, generation, and persistence

Use planet-local 3D directional noise and climate fields rather than longitude
textures with a seam. Derive independent random streams from
`(world_seed, body_id, generator_version, feature_tag)`; changing tree counts
must not move ore veins. Persist generator versions with worlds. A new generator
needs an explicit migration or continued old-generator support, not silent
regeneration beneath player edits.

Generation proceeds from low-frequency elevation/continental fields to
temperature, moisture, drainage, biomes, strata/caves, ores, flora, and points
of interest. Coarse climate/drainage data is compact and body-wide; expensive
cell materialization happens only for requested patches. Build every feature
from canonical cell/world coordinates so task order cannot alter boundaries.
Store bedrock as an implicit lower bound, air/solid runs as compressed columns,
and edited radial slabs densely only when necessary. Caves, overhangs, and
buildings require full voxel occupancy; a heightmap alone is insufficient.
The 512 m planet excavation target is bounded by a positive per-body inner
radius. Small moons and asteroids need shallower shells or a separately designed
fully destructible representation; radial prisms must never cross the center
or invert their inner/outer radii.

Initial tuning candidates are about 512 surface columns per patch and 32 radial
layers per slab. At two bytes of material plus two bytes of sky/RGB light,
16,384 resident cells need 64 KiB before auxiliary data. These are budget inputs,
not a final chunk format. Maintain occupancy bitsets, palettes/RLE, revision
counters, material summaries, and neighbor halos. Liquid amounts and gameplay
metadata are optional channels rather than always-paid bytes.

```text
Unloaded -> Requested -> Generating -> CPUReady -> UploadQueued
         -> GPUReady -> Visible -> Retiring -> Unloaded
```

Collider readiness and durable-edit state are separate flags. Generation,
light, mesh, and collider jobs carry `(chunk_key, revision, origin_epoch)`.
Discard stale results. Merge repeated edits into one dirty range and update
neighbor halos before scheduling dependent jobs. Eviction cancels unneeded
work; a late job may not resurrect an evicted chunk.

Streaming priority is collision safety, immediate view, predicted travel,
then distant detail. A high-speed ship requires an elongated preload corridor.
At 600 m/s, a 0.5 s cold-chunk deadline already needs 300 m of forward coverage
before braking distance; `v^2/(2*a)` at 20 m/s^2 is 9 km. Cruise therefore needs
coarse body/intersection proxies far ahead, early surface speed transitions,
and a safety brake if detailed collision cannot be made ready. A fixed 100 m
terrain radius cannot safely support cruise near the ground.

Each command follows one transaction path: validate against authoritative
cells, assign a monotonic sequence, append to a checksummed journal, commit,
publish authoritative change, then invalidate render/collider/light caches.
Local prediction can display a pending edit earlier; it remains marked pending
until a durability acknowledgement arrives. Native storage needs a documented
flush policy and crash recovery; browser storage needs an IndexedDB transaction
success boundary. A write queued in RAM is not a durable same-frame save.
Batch physical I/O only within an explicitly defined transaction without
acknowledging uncommitted edits. Background snapshots compact the journal and
can never substitute for recording accepted edits immediately.

Persist ship IDs, occupancy, cargo, damage, fuel, pose frame, and controller
mode through the same path. A dismounted ship remains a world entity or a
serialized unloaded entity. Far-away ships can sleep and resume under a
documented simplified policy; they do not acquire orbital mechanics on unload.

## 4. Bevy schedules and Avian physics

Use narrow component groups such as `BodyAnchor`, `ChunkHandle`, `ChunkRevision`,
`Ship`, `ControlIntent`, `FlightLimits`, `ReferenceFrameId`, and `PersistentId`.
The terrain array is a chunk-store resource with task ownership, not a million
entities or a global mutable singleton. Batch structural changes through
commands at explicit boundaries; avoid per-cell events, allocations, and
change-detection scans across all resident data. Dirty queues identify work.

The target fixed-tick pipeline is:

1. Consume input commands and advance the shared simulation tick.
2. Evaluate analytic anchors and perform scheduled frame/rebase transitions.
3. Apply committed edits; publish completed collider versions before contact generation.
4. Compute per-body gravity and bounded ship/character control.
5. Run Avian integration, broad phase, contacts, and constraints once.
6. Enforce documented emergency speed rules, resolve gameplay impacts, and capture authoritative poses.
7. Publish small render deltas and persistence/network outputs.

Avian 0.6 defaults to `FixedPostUpdate` and exposes physics schedules/sets.
Choose one schedule configuration, order systems using that version's actual
sets. Interpolation may be used for other actors' render transforms; the local
player camera deliberately uses immediate mouse orientation and the latest
physical position, without interpolation or easing. Never integrate
the same body once in a pure core update and again in Avian. Core returns control
accelerations or target velocities; the adapter applies them at the correct
physics boundary. See [Avian scheduling and interpolation](https://docs.rs/avian3d/0.6.1/avian3d/).

Start at 60 physics ticks/s, with limited substeps and a capped catch-up budget.
Use one simulation clock for orbits, flight, input replay, and scripted captures.
Render interpolation does not change authoritative state. GPU particle clocks
may be cosmetic; gameplay timers cannot read an unrelated wall-frame delta.

Set uniform Avian gravity to zero and supply the body's local gravity vector.
Use static colliders for resident terrain, primitive/compound hull proxies for
ships, and kinematic anchors for rails only where they actually participate in
the local physics world. Do not attach full detailed planet meshes to colliders.

Terrain colliders are generated asynchronously from the same CPU voxel snapshot
used by gameplay queries. Use merged solid spans/convex wedges for simple areas,
or chunk-local triangle soups for complex caves. A generic heightfield cannot
represent radial hex columns, caves, or overhangs. Dynamic ships use convex
pieces, not a constantly recooked concave triangle mesh. Quantify collision
memory and rebuild time independently of visual mesh time.

Swap a completed collider atomically at a safe physics boundary. For pending
edits near actors, temporary analytic cell collision must reflect newly placed
and removed blocks, or the command remains pending until the collider is ready.
Keeping an obsolete solid wall forever after mining is as wrong as falling
through a newly placed floor. Broad-phase invalidation, overlapping-character
depenetration, and no-hole swaps require integration tests. Tree trunks and
partial blocks need their own collision classes rather than the full-cube
occlusion predicate that caused upstream regressions.

Use swept tests/CCD for fast ships and projectiles, and explicit terrain sweeps
for character movement. Collision readiness gates entry into newly streamed
space. Sleeping and distance-based tick rates apply to dynamic props and fauna;
never sleep an object just because its renderer was culled.

## 5. Celestial rails and assisted Newtonian flight

### Rails are for anchors

Evaluate planet, moon, and station poses as closed-form functions of tick time
and stored orbital elements. Hierarchical parent orbits compose in f64; an
analytic derivative supplies anchor velocity. A circular/inclined rail is enough
for the initial game, with eccentric rails an optional later extension. A station
can have a prescribed orientation and artificial-gravity volume. No anchor
needs n-body integration, and save/load can evaluate the same epoch directly.

Station passengers integrate station-local positions and jumps. The station is
stationary in that simulation bubble, and composed world motion supplies the
orbit. As in Tenebris, omit Coriolis/centrifugal terms inside deck frames for
playability. This is an explicit gameplay approximation, not a physically exact
rotating frame. Leaving the volume transfers position and velocity using the
frame equations; re-entry uses hysteresis so the reference frame does not chatter.

### Ship gravity and limits

Ships never use patched conics, Kepler element propagation, sphere-of-influence
trajectory solvers, Hohmann transfer planning, or maneuver-node gameplay.
Outside dock/deck volumes they are free-flying local rigid bodies with thrust,
gravity, inertia, collisions, and configurable control assistance.

For a selected gravity source at center `c`, radius `R`, surface gravity `g0`:

```text
r = length(x - c)
g(x) = -normalize(x - c) * g0 * (R / max(r, R))^p * cutoff(r)
```

Choose `p = 2` initially and smooth `cutoff` from one to zero between authored
inner/outer field radii. Clamp inside the surface and handle `r = 0` without
NaNs. Gravity-source selection uses strongest acceleration with hysteresis;
blend source changes briefly when appropriate. Stations override this only
inside declared artificial-gravity volumes. This is a bounded gameplay field,
not a conserved orbital potential throughout the solar system.

In the current reference frame, use thrust input plus gravity and linear
velocity-error damping. Rotational input operates on body axes, with angular
velocity damping and quaternion orientation. Designer limits are magnitudes,
so diagonal input cannot gain `sqrt(3)` speed or acceleration. Controllers
work in physical units and use fixed `dt`; damping gains are per second.

| Initial profile | Linear limit | Controlled acceleration | Angular limit | Angular acceleration |
| --- | ---: | ---: | ---: | ---: |
| Surface flight | 120 m/s | 20 m/s^2 | 1.5 rad/s | 3 rad/s^2 |
| Space cruise | 600 m/s | 20 m/s^2 | 1.5 rad/s | 3 rad/s^2 |
| Dock approach | 15 m/s | 20 m/s^2 maximum, tuned lower for comfort | 1.5 rad/s maximum | 3 rad/s^2 maximum |

These are GDD tuning targets, not demonstrated flight feel. The production
controller should form a candidate velocity from total continuous acceleration,
project it onto the speed sphere, then limit the change from the previous valid
velocity by `a_max * dt`. The segment between two vectors inside a sphere stays
inside it, so this satisfies both speed and continuous acceleration limits when
the previous velocity is valid and the reference frame/cap is unchanged. Apply
the same construction to angular velocity. Feed the resulting delta once into
Avian's integration path. A speed clamp alone is not an acceleration limiter;
Avian's generic damping alone is not a maximum-speed controller.

When reducing the speed profile, anticipate the transition using braking
distance and ramp the target down. Collision impulses, teleport recovery, and
corrupt/out-of-range initial states cannot simultaneously obey an instantaneous
lower speed cap and a finite acceleration bound. Document them as discrete
events: impacts produce damage and an emergency post-contact speed clamp;
ordinary thrust, gravity compensation, and damping obey the continuous bounds.
Display/test this distinction instead of quietly violating the acceleration
contract. Caps are frame-relative, never the planet's absolute orbital speed.

Inertial dampeners can offer a player-selected coast mode, but safety envelopes
remain active. Rotation damping stabilizes physical ship attitude; it never
smooths or delays the player's mouse look. Hover requires adequate control
authority; tune gravity and available acceleration
together. Fuel/power loss behavior is a later explicit gameplay mode, not an
excuse to reintroduce orbital rocket simulation.

## 6. GPU terrain pipeline

### CPU/GPU division

The CPU owns compact occupancy, material IDs, topology, edit journals, and
collision snapshots. Upload changed compact slabs/halos, not expanded vertex
arrays. The GPU owns face enumeration, visual light volumes, visibility, draw
arguments, and the expanded positions produced by vertex pulling. Procedural
GPU terrain generation may later accelerate cosmetic detail, but authoritative
terrain must not depend on cross-vendor floating-point noise agreement.

The standalone volumetric shader contract uses 16-byte face descriptors
`{cell, face, material, light}`, 32-byte cell records, and 112-byte column records
containing six corner rays plus center/degree. These deliberately explicit
prototype layouts are documented in [shader-port.md](shader-port.md). Their
size is a reason to stream them, not to upload every column on the planet.
They are separate from the desktop surface renderer's currently integrated
128-byte column ABI and visibility-ID buffer described in the implementation
ledger. Supplying the desktop renderer does not integrate these volumetric
shaders automatically.
Keep Rust storage structs `repr(C)`, explicit padding, and asserted sizes; WGSL
`vec3` alignment cannot be inferred from a Rust `[f32; 3]` field.

Use per-page buffers or buffer ranges below the adapter's binding limits. Data
storage needs `STORAGE | COPY_DST`, generated face lists need `STORAGE`, and
GPU-written draw arguments need `STORAGE | INDIRECT`. Add `COPY_SRC` only for
diagnostics that actually read back. Allocation size, binding range, dynamic
offset alignment, workgroup size/count, texture dimensions/layers, and total
resident bytes must all be checked against the negotiated adapter limits.

### Ordered work

```text
CPU changed cells + topology halos
       -> upload -> reset counters -> light jobs -> face emission
       -> finalize valid generation -> visibility -> indirect terrain draw
       -> opaque depth/color -> water -> atmosphere/post composite -> UI
```

The exact water/atmosphere composition avoids duplicate fog and is established
by render tests; the diagram is the dependency order, not a promise that every
effect is a separate full-screen pass. In Bevy, extract small immutable deltas,
prepare allocations and bind groups in the render world, and order compute
before its consumers with explicit render-graph edges. Bevy's pinned
[compute example](https://github.com/bevyengine/bevy/blob/v0.18.1/examples/shader/compute_shader_game_of_life.rs)
provides the lifecycle pattern, not a ready-made voxel renderer.

1. **Reset:** Clear per-build counts, overflow flags, and indirect arguments.
2. **Light:** Run bounded dirty-region propagation dispatches, with halo epochs.
3. **Emit:** One invocation tests a cell's top, bottom, and valid 5/6 lateral
   neighbors. Use material face-occlusion rules, not merely `material != air`.
   Emit a compact record per exposed face. A cap expands to a 5/6-triangle fan;
   a side expands to two triangles. Degree-aware expansion can pad unused
   triangles degenerately for a fixed vertex count, then optimize by face class.
4. **Finalize:** A separate dispatch validates the face count/capacity and writes
   complete indirect arguments. A workgroup barrier cannot synchronize a global
   multi-workgroup emission and finalization; separate ordered dispatches do.
5. **Cull:** Frustum-test page/chunk bounds. Add conservative sphere-horizon
   rejection outside bodies. Later use previous-frame hierarchical depth only
   with conservative bounds, camera-motion slack, and visibility hysteresis;
   caves and near-plane intersections cannot use naive horizon culling.
6. **Draw:** Vertex pulling reconstructs positions and UVs directly from GPU
   records. No expanded mesh readback or full per-frame vertex upload occurs.

Face generation runs for dirty chunks, not all visible terrain every frame.
Camera culling runs each frame without remeshing. Opaque, cutout, and liquid
faces use separate queues/material rules. Pixel-art texture arrays avoid atlas
bleed; a packed atlas fallback needs padded tiles and appropriate mip generation.
Greedy merging on a cube grid is not directly valid on curved Goldberg wedges;
begin with exposed-face records, then benchmark merging only compatible coplanar
faces/spans where it preserves shape, material orientation, and baked light.

### Capacity, publication, and buffer lifetime

An isolated degree-six solid voxel can expose eight faces. Size pages for a
documented worst case or use a counting/prefix-sum prepass. Check capacity before
every write, count overflow, and ensure total attempted allocations cannot wrap
`u32`. An overflowing build must not publish a partial mesh or an indirect count
larger than storage. The prototype finalizer emits zero new instances on
overflow; the integrated renderer must retain the previous valid generation,
schedule a larger page/split, and display diagnostics. New chunks can use a
coarse safe visual until a valid generation exists.

Maintain current and pending mesh generations and a chunk revision/generation
table. Publish a pending page only once all required work and halos are valid.
Do not overwrite a page still referenced by an in-flight frame. GPU queue order
establishes compute-to-render visibility in one ordered submission; wgpu tracks
resource transitions. That does not make a CPU-side recycled range safe.
Retire allocations against submission completion notifications and recycle
them only after completion. A fixed count of three frames is not a fence.

Use a small ring of async `MAP_READ` staging buffers for overflow statistics and
optional timings. Never map the active storage buffer or synchronously wait for
geometry to drive gameplay. GPU records remain authoritative only for rendering.
Device loss discards all derived GPU state and rebuilds it from CPU chunk revisions.

### Capability tiers

| Tier | Geometry path | Requirements and fallback |
| --- | --- | --- |
| Portable WebGPU/native | Compute + vertex pulling + per-page indirect draws | Zero `first_instance`; CPU submits a bounded draw per page; GPU writes instance count |
| Enhanced native | Batched indirect submission and optional count-buffer draws | Enable only advertised features; native-only indirect-count is an optimization |
| Reduced compute device | Smaller pages, fewer resident chunks, simpler light/water | Lower negotiated limits and memory budget; same gameplay state |
| CPU diagnostic path on native/WebGPU | CPU chunk meshing with reduced view distance, if implemented | Reference/fallback path for testing geometry; not part of the current foundation |
| WebGL or no supported adapter | Unsupported-device screen | WebGL is outside the product scope; never pretend compute exists |

`INDIRECT_FIRST_INSTANCE` is optional; baseline records use zero and a page-base
uniform/storage indirection. `MULTI_DRAW_INDIRECT_COUNT` is native-only in the
pinned wgpu feature set. Do not require bindless arrays, subgroups, f64 shaders,
mesh shaders, float atomics, or timestamp queries for correct gameplay. See
[wgpu 27 feature definitions](https://github.com/gfx-rs/wgpu/blob/v27.0.1/wgpu-types/src/features.rs).
WebGPU support does not imply browser availability, identical limits, or all
native optimizations; probe the adapter and report the selected tier.

## 7. Lighting, water, atmosphere, and pixel art

### Baked voxel light

Retain Tenebris's sky and emissive-light separation, occlusion behavior,
straight downward daylight, and face-adjacent/corner sampling. Upstream stores
one byte with 4-bit sky and 4-bit block intensity; an optional RGB block-light
extension is a new design, not an existing Tenebris feature. The WGSL prototype
uses sky plus RGB channels and explicit neighbor tables. Its ping-pong Jacobi
steps derive from the same propagation rule but are not a line-for-line CPU BFS.

Daylight column seeding must be recomputed when an occluder changes; removal of
a torch needs invalidate/reset-and-reseed or a proper removal/addition queue.
Repeated max-propagation on the previous field alone leaves permanent light.
Bound updates per frame and exchange halos until a stable revision is reached.
Cross-chunk light cannot stop at a buffer edge. Freeze old valid lighting while
rebuilding or show a documented temporary approximation. Bake/sample visual
light independently of mesh topology so a torch does not require rebuilding
positions; the current face-record prototype may still need light-field refresh.

Ambient occlusion is local occupancy shading; baked sky is connectivity to the
sky; directional sunlight adds the current day/night angle. They are distinct
terms. Use identical opacity/emission tables in Rust and GPU data. If gameplay
uses darkness for spawning/growth, give it a CPU light authority or a conservative
CPU query with the same rules; it must not wait for GPU lighting readback.

### Water

Translate useful equations from the Rust client's GLSL: wave gradients,
Fresnel reflection, bounded screen refraction, depth-based absorption,
shallow/deep color, foam, and atmospheric distance fade. Preserve the declared
planet-local camera convention that fixed the upstream offset-world water bug.
WGSL has different binding, layout, texture, and clip-depth rules; raw GLSL
strings are reference artifacts only.

Water consumes resolved opaque scene color/depth from an earlier pass. Never
read a texture while also rendering into it. Reconstruct positions with the
actual Bevy projection/depth convention, handle reverse-Z and clear-sky depth,
MSAA resolve, foreground rejection, underwater view, and world-space thickness.
Clamp refraction offsets and reject samples in front of the water surface.
Do not double-apply atmospheric fog to refracted color already fogged upstream.
The initial shader port still needs this render-graph integration and visual
calibration; future clouds/reflections must obey the same pass contract.

Fluid simulation remains CPU-authoritative with low-frequency active-cell
updates, edit journaling, and finite budgets. Ocean render waves are cosmetic
displacements and buoyancy samples; they are not a global per-voxel fluid solver.

### Atmosphere

Retain single scattering, Rayleigh/Mie phase terms, shell intersection,
planet shadowing, scale height, and per-body wavelengths/sunset controls from
Tenebris's atmosphere shader. The source uses eight view samples and four light
samples; the integrated desktop preview uses sixteen/four to improve its
terminator. Keep a comparable reference path before introducing LUTs/half-resolution
passes and temporal reuse. Benchmark those later choices, including GPU cost
at the horizon and banding on the pixel-art palette.

One atmosphere definition drives the sky, terrain distance treatment, and water
fog. Airless bodies skip atmosphere, cloud, and precipitation passes. Sample
body-local rays with a shared sun direction, and test camera positions below,
within, and above the shell on an offset body. Multiple distant bodies can use
cheap shell/LUT appearances while the active body uses the detailed effect.

### Pixel-art presentation

Use the biome atlas manifest as the material authority. Pixel textures use
nearest/point sampling, deliberate 16/32-pixel motifs, readable value clusters,
and restrained emissive colors. Future distance tiers can use curated,
palette-aware mip levels selected without blending, alongside geometric LOD
and reduced distant detail. Preserve nearest filtering rather than introducing
bilinear/trilinear texture blur. Avoid temporal blur and photoreal roughness
detail that erase the pixel language.
The terrain shader should preserve material colors while receiving baked
light, atmosphere, weather wetness, and controlled directional shading.

## 8. Budgets, profiling, and acceptance gates

These are **initial budgets**, not performance measurements: 60 fps at 1080p
on a named midrange discrete GPU, 16.67 ms frame time with headroom, a 4 ms
fixed-simulation budget inside the GDD's 8 ms CPU critical-path target, and
bounded background/renderer queues. Reserve an initial 512 MiB-1 GiB GPU terrain
pool inside the GDD's provisional 2 GB total render residency on a 4 GB VRAM
tier, with at most 1 GB of CPU chunk/cache data. Configure lower tiers by observed
adapter behavior. Avoid interpreting maximum buffer size as available VRAM.
Account for current/pending mesh copies, light ping-pong buffers, halos,
texture arrays, staging, and transient render targets in the same budget.

A million 16-byte face records costs 16 MB before cell/topology/light data.
Even without CPU transfers, repeatedly writing/reading these records and
expanding triangles consumes bandwidth and vertex work. GPU residence removes
one bottleneck; face count, atomics, cache locality, transparent overdraw, and
draw submission still need measurement. Per-workgroup compaction, prefix sums,
occupancy masks, and compressed descriptors are later alternatives evaluated
against the simple reference, not speculative mandatory complexity.

Measure p50/p95/p99 frame time, fixed-tick time, generation/collider job latency,
uploaded bytes, resident bytes, visible faces, indirect draws, edit-to-display
latency, overflow events, and rebase spikes. GPU timestamps are optional;
without them label CPU submission timing accurately. Capture hardware, driver,
backend, build profile, resolution, seed, route, warm-up, and active tier.

| Gate | Evidence needed before declaring it complete |
| --- | --- |
| Topology | Euler/degree invariants, 12 pentagons, reciprocal neighbors, seam-equivalent IDs, stable save IDs, LOD boundary captures |
| Coordinates | Large offset planets, repeated rebases, station jump/departure, quaternion pole/roll traversal without pose discontinuity |
| Flight | Fixed-step rate comparisons, diagonal inputs, zero/invalid parameters, gravity transitions, braking profiles, linear/angular magnitude bounds |
| Persistence | Crash/reload after accepted edits, dismount/reboard/reload, interrupted journal recovery, generator-version migration |
| Physics | Fast landing, cell placement/removal under feet, cave ceilings, trees, chunk seams, collider swaps, CCD, origin-shift contacts |
| GPU geometry | Actual pipeline creation, readback comparison to a tiny CPU reference in tests only, pentagon caps, full/overflow pages, generation retirement |
| Lighting | Sealed cave, open skylight, torch removal, cross-chunk propagation, opaque/partial block differences, no permanent stale light |
| Water/atmosphere | Dry/underwater, offset planet, sunrise/sunset, inside/outside shell, airless world, reverse-Z and MSAA captures |
| Performance | Identical reproducible routes before/after on native hardware; browser tier separately profiled |

The technical sequence refines the GDD milestones: (1) tested coordinate/flight
foundation; (2) one streamed spherical patch with CPU-authoritative collision
and an edit journal; (3) integrated GPU face path and overflow/lifetime handling;
(4) voxel light, material arrays, and planetary LOD; (5) water and atmosphere;
(6) orbital anchor/frame transitions and multi-planet travel; (7) gameplay,
storage/recovery hardening, then multiplayer. Stages 2-3 together supply the
terrain laboratory gate, including save/reload and a pentagon/seam. Each stage
keeps a runnable diagnostic scene and clear pass/fail evidence.
The coarse desktop explorer is an intermediate rendering/flight prototype. Its
whole-globe display and tour implementation do not complete the terrain
laboratory gate's editable microvoxels, durable saves, or collider-swap evidence.
