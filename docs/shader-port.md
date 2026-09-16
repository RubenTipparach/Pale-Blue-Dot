# Shader port and GPU interfaces

The repository includes five standalone WGSL modules, semantically validated with Naga **27.0.3**, the version in the source `swarm-demo` lockfile alongside Bevy 0.18.1 and wgpu 27.0.1. They contain working shader algorithms and explicit interfaces. **They are not registered in a Bevy render graph, dispatched by the current application, visually matched against Tenebris, or GPU benchmarked.** The engine foundation can run without a window or render dependencies. Shader validation proves language and interface correctness, not rendered output or frame rate.

## Source traceability

Reference revisions inspected:

- Tenebris: [`ef9865166349c7a3c4ae09acdd20640daf85b4fd`](https://github.com/RubenTipparach/tenebris/tree/ef9865166349c7a3c4ae09acdd20640daf85b4fd).
- Swarm demo: [`625596b437fec454412b3c7083a3a04e510617f1`](https://github.com/RubenTipparach/swarm-demo/tree/625596b437fec454412b3c7083a3a04e510617f1).

Paths below are relative to those repositories. No top-level `LICENSE` or `COPYING` file was found in either inspected checkout; these ports do not assign a new license to the source mathematics or upstream expression. The upstream rain-ripple code specifically attributes Zavie / Ctrl-Alt-Test's “H - Immersion”; that externally attributed kernel is **not copied** into these assets. Repository access is not itself a license grant. Preserve this provenance when distributing adaptations.

| New asset | Upstream file and exact mathematical source | Implemented changes |
| --- | --- | --- |
| `assets/shaders/water.wgsl` | `tenebris-rs/crates/tenebris-client/src/shaders/water.fs.glsl`: `gn_fade`, `gn_hash`, `gn_grad`, `gnoise3`, `fbm3`, `main`; `water.vs.glsl`: three-sine displacement in `main` | Named WGSL variables replace generated `_NNN` identifiers; preserves hash constants, gradient noise, FBM weights/frequencies, slope cap, Schlick reflection, absorption, foam, specular, underwater window, refraction rejection and fog ceiling. Uses inverse projection and reverse-Z instead of OpenGL depth algebra. |
| `assets/shaders/atmosphere.wgsl` | `tenebris-rs/crates/tenebris-client/src/shaders/atmosphere.fs.glsl`: `raySphereIntersect`, `getDensity`, `computeOpticalDepth`, `rayleighPhase`, `miePhase`, `main`; `atmosphere.vs.glsl`: camera-basis ray reconstruction | Retains eight view steps, four light steps, planet occlusion, Rayleigh/Mie terms and configurable dusk colors. Guards empty shell, zero scale height and near-singular Mie denominator. Outputs linear scattering for final Bevy tone mapping. |
| `assets/shaders/hex_terrain.wgsl` | `tenebris-rs/shaders/hex.glsl`: vertex terminator and baked-light interface; `tenebris-rs/crates/tenebris-client/src/shaders/hex.fs.glsl`: `hex_bayer4`, base `lit` expression, underwater attenuation, atmospheric limb block | Preserves hard pixel cutouts/dither, day/night ambient, Lambert sun times sky visibility, underwater absorption and altitude-gated limb. Expands scalar torch lighting to RGB baked light and replaces atlas constants with texture-array layers. Adds original vertex pulling. |
| `assets/shaders/voxel_light.wgsl` | `tenebris-rs/crates/tenebris-core/src/world_light.rs`: `voxel_neighbours_with_cost`, `sky_seed_one_column`, `bfs_one_pop`, incremental/full rebuild semantics | New parallel Jacobi implementation of seeded, attenuated neighbor propagation. RGB + sky is an intentional extension beyond upstream `(sky << 4) | block`. Removal requires reseeding, not repeated maximum against stale results. CPU BFS/priority scheduling itself is not translated into this shader. |
| `assets/shaders/hex_faces.wgsl` | Architectural reference: `swarm-demo/crates/swarm_app/src/swarm.rs`, `SwarmPlugin::build`, `init_tick_pipeline`, `SwarmTickNode::run`; `assets/shaders/swarm.wgsl` storage layout | Original topology-aware face extraction, bounded writes, counters, indirect arguments and vertex expansion. Adopts persistent GPU buffers and ordered compute-before-draw dispatches. Does not transplant swarm movement, combat or density algorithms. |

The source `world_light.rs` introduction says “6-neighbour,” but the actual surface is a Goldberg grid: a column has five or six horizontal neighbors plus two radial neighbors. The port uses an explicit eight-slot topology table and supports pentagons; it does not infer spherical adjacency from flat axial coordinates.

## Validation

From the repository root:

```powershell
cargo run --manifest-path tools/validate_shaders/Cargo.toml --locked
cargo test --manifest-path tools/validate_shaders/Cargo.toml --locked
```

The utility has its own workspace and lockfile. It parses all five modules, runs all Naga validation flags with no optional capabilities enabled, checks declared bindings, checks entry-point stages/workgroup sizes, and asserts important struct spans/member offsets. Negative tests demonstrate rejection of a type error, an invalid uniform-array layout, and the 16-to-32-byte padding mistake caused by substituting `vec3<u32>` for three scalar pads. It does not create a GPU adapter, compile a backend pipeline, check texture contents, or exercise read/write ordering on hardware.

## Coordinate and color contract

Authoritative positions and celestial ephemerides remain CPU `f64`. Before extraction, compute `planet_center = body_world - local_origin` and `camera_local = camera_world - local_origin` in `f64`, then convert the differences to `f32`. Terrain and water receive the same origin and orientation for these vectors and both matrices. Never subtract a rebased planet center from an unre-based camera.

Geometry stores normalized planet-local corner rays and radial distances in metres. The initial vertex expansion evaluates `planet_center + corner * radius` in `f32`; this is appropriate for the specified kilometre-sized bodies, not an assertion of millimetre precision for astronomical radii. For larger bodies, introduce chunk-relative corner positions and a split high/low translation before increasing planet scale. Rotation from a body's local axes into the active local frame must already be applied consistently to corner rays, sun vectors and shader inputs, or supplied by a future explicit body transform. The current geometry interface assumes aligned axes.

Atmosphere receives camera position relative to the body center and camera/sun vectors in that body's axes. Water carries body-local positions separately for noise so a floating-origin shift does not shift wave phases. Bound the input range and periodically rebase animation time with a continuous phase scheme before long-running sessions; the current shader does not provide phase-continuous time wrapping.

Texture-array albedo should use an sRGB texture view so sampling yields linear color. Lighting, water scene color and scattering are linear; tone mapping belongs to the final camera pass. Use nearest texture filtering for terrain, optional nearest mip selection, and edge-safe tile layers. The terrain alpha output is ordinary opaque alpha; Tenebris's repurposed wetness alpha channel is not carried into Bevy's scene color.

## Opaque face extraction and vertex pulling

Storage structures use tightly specified WGSL layouts. Array stride equals the stated span.

| Struct | Size | Fields with byte offsets |
| --- | ---: | --- |
| `Cell` | 32 B | `column: u32` 0; `material: u32` 4; legacy/reserved `light: u32` 8; `flags: u32` 12; bottom/top `radii: vec2<f32>` 16; padding 24 |
| `Column` | 112 B | six normalized `corners: vec4<f32>` at 0, stride 16; normalized center ray xyz and degree 5/6 in `center_degree.w` at 96 |
| `Neighbors` | 32 B | eight `u32` IDs at 0: top, bottom, sides 0–5 |
| `Face` | 16 B | cell ID 0; face selector 4; material ID 8; packed exposed-neighbor light 12 |
| `Params` for face extraction | 16 B | owned count 0; face capacity 4; scalar padding 8,12 |
| `Counts` | 16 B | atomic requested count 0; atomic overflow flag 4; scalar padding 8,12 |
| `DrawIndirect` | 16 B | vertex count 0; instance count 4; first vertex 8; first instance 12 |

`hex_faces.wgsl` uses bind group 0:

| Binding | Buffer | Access/usage |
| ---: | --- | --- |
| 0 | `Params` | uniform |
| 1 | cells, owned prefix followed by halo | storage read |
| 2 | neighbor records | storage read |
| 3 | column geometry | storage read |
| 4 | output faces | storage read/write; reused as read-only by draw |
| 5 | counts | storage read/write; optional asynchronous diagnostics |
| 6 | draw arguments | storage read/write **and INDIRECT** usage |
| 7 | final RGB/sky light array | storage read; direct reference to finished bake buffer |

This uses seven storage bindings, within the target baseline of eight. Check actual adapter limits before pipeline creation; buffer binding sizes and total resident allocation still require budgets. Use `COPY_DST` only for buffers uploaded/cleared that way and `COPY_SRC` only when diagnostic readback requires it.

The owned prefix prevents halo voxels from drawing duplicate faces. Cell flags: bit 0 means the volume covers its neighbor's opaque face; bit 1 means the cell is owned and drawable. Material 0 is air. The engine foundation currently defines Stone=1, Soil=2, Grass=3, Water=4 and Ore=5 on axial chunks; an extraction adapter must map those CPU chunks to this shader's explicit spherical column/neighbor data. Water=4 must not receive the opaque owned-drawable flag. No separate procedural generation happens on the GPU, so the shaders do not introduce a second seed/noise algorithm. A topology ID of `0xffffffff` means unknown/unavailable and stays closed; `0xfffffffe` explicitly means known open sky and contributes sky level 15. Do not substitute open sky for an unloaded neighbor. Pentagon neighbor slot 7 is unused. This first extractor handles opaque/cutout surfaces; water and refractive material boundary rules need their own classified face list before integration.

The host must validate finite radii/rays, positive shell thickness, degree exactly 5 or 6, consistent winding, valid material layers, valid neighbor references, owned count and equal snapshot revisions. Bounds guards prevent shader memory overruns but do not repair malformed topology.

Issue these dispatches, in order, before rendering:

1. `reset`, workgroup size 1, dispatch `(1,1,1)`.
2. `emit`, workgroup size 64, dispatch `(ceil(owned_count/64),1,1)`.
3. `finalize`, workgroup size 1, dispatch `(1,1,1)`.
4. Bind faces and geometry to the terrain pipeline and issue `draw_indirect`.

The work must be ordered by command encoding/render-graph dependencies, never by a workgroup barrier pretending to synchronize the entire dispatch. Each extracted visible face requests an atomic slot; only slots below both the configured capacity and actual buffer length are written. Owned count is capped below the threshold where eight requests per cell can wrap a `u32`. `finalize` writes `(18, face_count, 0, 0)` on success, or zero instances when overflow occurs. `first_instance` is always zero, so the path does not require indirect-first-instance support.

Overflow is not permission to present holes. Allocate into a replacement chunk slot, retain the previous valid render allocation, and publish the new allocation only when the job succeeds. The renderer's revision check, stale-job cancellation, allocation retirement and successful-publication logic are future integration work. Rendering a failed replacement directly would make that chunk disappear.

`hex_terrain.wgsl` bind group 0 is `TerrainView` (208 B), faces, cells and columns at bindings 0–3. Group 1 contains a `texture_2d_array<f32>` and nearest sampler at bindings 0–1. `vertex` expands each face descriptor into triangles; no expanded vertex/index buffer returns to the CPU. Cap triangles use the supplied normalized corner rays and support degree 5/6. Side triangles use the corresponding two rays and bottom/top radii. Corners must be counterclockwise when viewed from outside. Face indices 0,1,2…7 mean top,bottom,side0…5.

For this correctness-first format all instances draw 18 vertices. A hex cap uses all 18, a pentagon cap uses 15 plus a degenerate triangle, and a side uses 6 plus degenerates. This avoids multiple draw bins but wastes vertex work on sides. Profile before splitting cap/side lists into separate indirect draws or adding GPU compaction; no performance advantage is claimed without measurement.

Material `m >= 1` maps to texture-array layers `(m-1)*3 + {0 top,1 side,2 bottom}`. Asset import must build those layers from the biome tile manifests. PNG sheets are not directly bindable texture arrays, and that importer is not implemented here. The old atlas's `93/108` scaling is deliberately absent.

## Baked RGB and skylight

Both terrain and light compute use `u32` values whose low 16 bits are `R | (G << 4) | (B << 8) | (sky << 12)`, each channel 0–15. This format is an extension, not bit-compatible with Tenebris's two-channel byte. Migration from old data maps old block intensity through the old torch tint into RGB and keeps the sky nibble. Display intensity is shader-configured; the normalized channel value is not a physical lux measurement.

Light bindings in group 0: parameters (0), `LightCell` metadata (1), neighbors (2), previous values (3), next values (4). `LightCell` is 16 bytes: emission RGB nibbles at offset 0, flags at 4, scalar padding at 8/12. Flags are opaque bit 0, unobstructed direct sky bit 1, frozen halo bit 2. Neighbor ordering matches the face shader.

CPU authoritative terrain/topology determines the direct-sky seeds: a column receives 15 only along its unobstructed path from the build cap. The shader does not scan an entire column or determine occlusion outside the loaded region. `initialize` writes seeds; `relax` writes the channelwise maximum of seeds and neighboring previous values minus one, with saturating subtraction. Opaque cells block propagation but may emit. No invocation writes another voxel, so no atomics are needed in the relaxation pass.

For initial generation or an edit that removes a source/blocks daylight, reseed **both** ping-pong buffers in the affected region before relaxation. Reusing stale light as a maximum would retain phantom light. Supply the same valid, frozen boundary halo in both buffers. CPU seed uploads can initialize known external boundaries; production cross-chunk visual-light halos should use GPU buffer copies/compute exchange without reading whole grids back. The current shader exposes a caller-provided frozen halo and does not implement that cross-chunk exchange or convergence scheduler. The dirty region must encompass the entire changed light influence (15 graph steps for unit attenuation), and direct-sky seed changes may dirty the whole affected shaft. Global edits may require region/chunk expansion, not just the edited voxel.

Run up to 15 ordered relaxation dispatches, swapping distinct previous/next buffers after each. With seeds already initialized, channel values bounded by 15 and unit attenuation, this covers the possible nonzero influence radius; preserve frozen boundaries. The final buffer feeds face extraction binding 7 directly. GPU values remain a visual cache; networking/gameplay/collision must query authoritative CPU state. CPU removal queues and budgeted authoritative light updates remain separate work. Face light currently samples one exposed neighbor, so there is no per-corner light smoothing or ambient occlusion term yet.

## Water integration contract

`WaterView` is 352 bytes: two 64-byte matrices, then fourteen 16-byte vectors in declaration order. Group 0 binding 0 is the uniform. Group 1 bindings 0,1,2 are resolved opaque scene color, resolved depth and a sampler. Resolve/prepare those textures before water and do not read/write the same scene-color attachment in one pass.

Vertex inputs are body-local position at location 0, geometric face normal at 1, and normalized baked RGB/sky at 2. These currently expect a water vertex stream. A water-specific face extraction/vertex-pulling adapter remains to be implemented; the opaque face draw does not automatically draw water.

The WGSL uses reverse-Z: near=1, clear sky=0; opaque geometry covers the water when its depth is larger. Reconstruction applies `local_from_clip` to WebGPU's `[0,1]` depth, with image Y inverted into NDC. It does not reuse Tenebris's OpenGL `[-1,1]` linear-depth formula. For samples at infinite sky depth it uses a configurable maximum water path instead of dividing an infinite-far homogeneous point.

Refraction rejects samples that cross the sky/terrain boundary or pull geometry in front of the water surface. Offset length and normal slope are independently capped. Absorption is exponential in reconstructed path length; Schlick uses the upstream 0.02 water reflectance. The sky reflection is the original gradient approximation, not screen-space reflection or ray tracing. Back faces apply the original underwater attenuation/window approximation. Fog is capped below one to keep distant water distinct from atmosphere.

Differences pending visual review: gradient projection uses the geometric face normal, displacement is gated to upward faces, RGB replaces scalar torch light, and scene reconstruction replaces the upstream depth approximation. Foam retains height/slope response. Waterfalls' flow UV advection, rain impacts, screen-space caustics, shoreline foam and multi-layer transparent sorting are not implemented. No promise of complete visual parity is implied.

## Atmosphere integration contract

`AtmosphereView` is 176 bytes: eleven vec4 values, group 0 binding 0. `vertex` emits a fullscreen triangle from `vertex_index`; `fragment` integrates the body-local shell. `radii.z` is dimensionless scale height normalized to the shell thickness, matching the original function, not metres. Nonpositive/empty shells return transparent. Use nonzero camera basis/sun vectors; an atmosphere center is not a valid camera position for this model.

Draw only where reverse-Z scene depth equals its clear value 0, depth writes disabled. Composite the returned scattering as premultiplied RGB with source factor One and destination factor OneMinusSrcAlpha. RGB is the integrated in-scattered radiance; alpha approximates average extinction for the background. The original shader's `1-exp(-color)` mapping and scatter-luminance alpha were replaced to avoid tone-mapping twice in Bevy and to give background attenuation a consistent meaning. There is no physically exact spectral multiple scattering.

The source clamps ground-hit rays and can output opaque atmosphere over ground. This module is deliberately **sky-only**; terrain aerial perspective, partial terrain-depth ray truncation and shared water/terrain fog color evaluation still need a dedicated depth-aware composite. The terrain limb term is ported independently. Multiple visible atmospheres must be composed in distance order with an agreed transmittance model; no planet-order solution is hidden in the shader.

## Renderer integration and acceptance work

Use the source swarm's `RenderApp` extraction/prepare/render separation: extract immutable chunk snapshots and revisions; prepare persistent buffers/bind groups in render schedules; queue ready pipelines; encode light and mesh jobs before camera draws. Create custom render pipelines for these standalone modules; they do not use Bevy's `#import` material interface and cannot simply be attached as a `StandardMaterial`.

Before claiming this path is ready, implement and verify:

- Buffer allocation/reclamation, bind-group layouts, topology/texture import, adapter limits and a CPU fallback for unsupported adapters.
- Revision-safe chunk publication, old-allocation retirement, device-loss recreation and bounded outstanding work.
- GPU tests comparing isolated hex/pentagon, stacked cells, neighbor occlusion, seam halos, zero capacity, overflow and stale edits against a CPU face oracle.
- RGB/sky addition/removal and chunk-boundary parity tests, including a deep vertical shaft and an opaque edit that closes it.
- Reverse-Z water occlusion/refraction captures, underwater views, origin rebases, pole/terminator views and sky/terrain composition tests.
- GPU timing and memory/triangle/face counters for target 2–12 km diameter bodies, travel/streaming and repeated terrain edits; asynchronous readback only for small diagnostics, never expanded terrain geometry in the main frame loop.

WGSL layout and validation rules: [W3C WGSL specification](https://www.w3.org/TR/WGSL/). The pinned parser is [Naga 27.0.3](https://crates.io/crates/naga/27.0.3); the executable's lockfile makes the language-validation result reproducible.
