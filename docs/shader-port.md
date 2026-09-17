# Shader port and GPU interfaces

The repository contains **eight shader assets**: five standalone ports for the planned voxel engine, two standalone WGSL modules wired into the desktop planet pipeline, and one Bevy-composed atmosphere material. Seven standalone modules have passed semantic/interface validation with Naga **27.0.3**, the version in the source `swarm-demo` lockfile alongside Bevy 0.18.1 and wgpu 27.0.1. Final native orbit, coast, surface, night, pole and survey-tour captures of the revised **655,362-column** scene have been inspected without reported GPU pipeline errors. Those captures demonstrate actual compute/indirect rendering and Bevy sky composition; they do not establish complete Tenebris parity or all target-device performance budgets. Shader validation and runtime evidence are recorded separately.

| Assets | Current application connection | Validation/status boundary |
| --- | --- | --- |
| `hex_faces.wgsl`, `hex_terrain.wgsl`, `voxel_light.wgsl`, `water.wgsl`, `atmosphere.wgsl` | Original standalone ports; no desktop dispatch/material binding | Naga semantics and interfaces passed. These do not become active simply because the desktop prototype exists. |
| `planet_visibility.wgsl`, `planet_surface.wgsl` | `PlanetPlugin` creates persistent storage, bind groups, visibility compute and indirect surface draw | Naga semantics/interfaces passed. Subdivision-8 rendering and altitude-dependent draw budgets exercised in native captures. |
| `sky_atmosphere.wgsl` | `SkyPlugin` registers `SkyMaterial` and spawns a shell mesh through Bevy's material pipeline | Bevy imports/substitutions compiled and rendered with the revised shell/cloud parameters. Explicitly deferred by the standalone validator; night-side review motivated the 16-step/soft-penumbra refinement. |

For a term-by-term diff of the three families against the Tenebris originals,
and what the live shader drops, see the measured shader comparison in
[tenebris-comparison.md](tenebris-comparison.md).

The headless physics/core path remains available without the desktop feature. Detailed interfaces below distinguish the live desktop prototype from the original volumetric/optical ports.

## Source traceability

Reference revisions inspected:

- Tenebris: [`ef9865166349c7a3c4ae09acdd20640daf85b4fd`](https://github.com/RubenTipparach/tenebris/tree/ef9865166349c7a3c4ae09acdd20640daf85b4fd).
- Swarm demo: [`625596b437fec454412b3c7083a3a04e510617f1`](https://github.com/RubenTipparach/swarm-demo/tree/625596b437fec454412b3c7083a3a04e510617f1).

Paths below are relative to those repositories. Tenebris's Rust Cargo metadata declares MIT and swarm-demo declares MIT OR Apache-2.0; no top-level `LICENSE` or `COPYING` file was found in either inspected checkout. These ports preserve provenance rather than assigning a new license to upstream art or third-party content. The upstream rain-ripple code specifically attributes Zavie / Ctrl-Alt-Test's “H - Immersion”; that externally attributed kernel is **not copied** into these assets.

| New asset | Upstream file and exact mathematical source | Implemented changes |
| --- | --- | --- |
| `assets/shaders/water.wgsl` | `tenebris-rs/crates/tenebris-client/src/shaders/water.fs.glsl`: `gn_fade`, `gn_hash`, `gn_grad`, `gnoise3`, `fbm3`, `rain_ripple_grad`, `main`; `water.vs.glsl`: three-sine displacement in `main`; `composite.fs.glsl`: the compose and lens passes, `rd_*` droplets, `rd_blur` | **Bound.** Named WGSL variables replace generated `_NNN` identifiers; preserves hash constants, gradient noise, FBM weights/frequencies, rain ripples, flow advection, slope cap, Schlick reflection, absorption, foam with its per-term weights, sun-tinted specular, underwater window, refraction rejection and fog ceiling. The vertex stage pulls the cap from the planet's `Cell` record instead of a water vertex stream. The same file carries the `compose` and `lens` fragment entry points for the composite. Uses inverse projection and reverse-Z instead of OpenGL depth algebra. |
| `assets/shaders/atmosphere.wgsl` | `tenebris-rs/crates/tenebris-client/src/shaders/atmosphere.fs.glsl`: `raySphereIntersect`, `getDensity`, `computeOpticalDepth`, `rayleighPhase`, `miePhase`, `main`; `atmosphere.vs.glsl`: camera-basis ray reconstruction | Retains eight view steps, four light steps, planet occlusion, Rayleigh/Mie terms and configurable dusk colors. Guards empty shell, zero scale height and near-singular Mie denominator. Outputs linear scattering for final Bevy tone mapping. |
| `assets/shaders/hex_terrain.wgsl` | `tenebris-rs/shaders/hex.glsl`: vertex terminator and baked-light interface; `tenebris-rs/crates/tenebris-client/src/shaders/hex.fs.glsl`: `hex_bayer4`, base `lit` expression, underwater attenuation, atmospheric limb block | Preserves hard pixel cutouts/dither, day/night ambient, Lambert sun times sky visibility, underwater absorption and altitude-gated limb. Expands scalar torch lighting to RGB baked light and replaces atlas constants with texture-array layers. Adds original vertex pulling. |
| `assets/shaders/voxel_light.wgsl` | `tenebris-rs/crates/tenebris-core/src/world_light.rs`: `voxel_neighbours_with_cost`, `sky_seed_one_column`, `bfs_one_pop`, incremental/full rebuild semantics | New parallel Jacobi implementation of seeded, attenuated neighbor propagation. RGB + sky is an intentional extension beyond upstream `(sky << 4) | block`. Removal requires reseeding, not repeated maximum against stale results. CPU BFS/priority scheduling itself is not translated into this shader. |
| `assets/shaders/hex_faces.wgsl` | Architectural reference: `swarm-demo/crates/swarm_app/src/swarm.rs`, `SwarmPlugin::build`, `init_tick_pipeline`, `SwarmTickNode::run`; `assets/shaders/swarm.wgsl` storage layout | Original topology-aware face extraction, bounded writes, counters, indirect arguments and vertex expansion. Adopts persistent GPU buffers and ordered compute-before-draw dispatches. Does not transplant swarm movement, combat or density algorithms. |
| `assets/shaders/planet_visibility.wgsl` and `planet_surface.wgsl` | New prototype code informed by the same swarm compute/draw lifecycle and Tenebris cap/side, terminator, skylight and rim concepts above | Horizon-culls CPU-generated surface columns, compacts IDs (terrain, nearby foliage and water cells, three indirect draws), draws caps/skirts/cosmetic trees by vertex pulling, absorbs submerged terrain with the water's own constants and carries the `hex.fs` rain-wetness block. Water cells draw their seabed here; the sheet is `water.wgsl`'s. This is not the standalone voxel face kernel. |
| `assets/shaders/sky_atmosphere.wgsl` | Adaptation of the documented `atmosphere.wgsl` port and its upstream 8-view/4-sun integration | Actual Bevy material interface, front/inside shell selection, 16-view/4-sun midpoint sampling, 36–60 m soft planetary penumbra, premultiplied scattering, warm dusk, daylight veil and stylized cloud noise. Does not reproduce upstream cube clouds or a full scene-depth atmospheric composite. |

The source `world_light.rs` introduction says “6-neighbour,” but the actual surface is a Goldberg grid: a column has five or six horizontal neighbors plus two radial neighbors. The port uses an explicit eight-slot topology table and supports pentagons; it does not infer spherical adjacency from flat axial coordinates.

## Validation

From the repository root:

```powershell
cargo run --manifest-path tools/validate_shaders/Cargo.toml --locked
cargo test --manifest-path tools/validate_shaders/Cargo.toml --locked
```

The utility has its own workspace and lockfile. It requires seven explicitly named standalone modules, runs all Naga validation flags with no optional capabilities enabled, checks declared bindings, checks entry-point stages/workgroup sizes, and asserts important struct spans/member offsets. It explicitly defers `sky_atmosphere.wgsl` because its Bevy imports and material substitutions require Bevy's shader composer. Unknown shader names remain errors. All seven standalone checks and three negative tests passed: the tests demonstrate rejection of a type error, an invalid uniform-array layout, and the 16-to-32-byte padding mistake caused by substituting `vec3<u32>` for three scalar pads. The tool does not create a GPU adapter, compile a backend pipeline, check texture contents, or exercise read/write ordering on hardware.

The final Rust verification passed 35 tests. Two scripted circumnavigation checks completed with zero protection events, and all six native capture views were rerun with the final 16-view/4-sun sky material. These validate the integrated prototype; the five unbound standalone ports still require their own GPU dispatch, image and readback acceptance checks when connected.

## Desktop pipeline currently wired in code

`pbd-app/src/desktop.rs` adds `PlanetPlugin` and `SkyPlugin` to its Bevy/Avian application. These use separate interfaces from the original five port assets. The default scene is one 4 km-radius planet at the local origin; no multi-planet streamed voxel renderer is implied.

`planet_topology.rs` constructs a closed dual of a subdivided icosahedron, including twelve pentagons and shared corner rays; `planet_lattice.rs` addresses the same construction at any level by `(face, level, i, j)` and builds the dual over a cap, and `planet_lod.rs` assembles the resident set. The shipped body is a 4,800 m sea-level radius with a level-7 base for the whole globe (163,842 cells, 45 m tiles) and levels 8 to 11 resident as bands of 2,400, 1,200, 600 and 300 m around the player, so the tile underfoot is the 2.833 m gold standard: about 160,000 fine records, 59 MiB in all at 192 bytes per record, plus visible IDs per view. `planet_terrain.rs` supplies CPU heights/biomes, compressing the raw generator by 0.17 above the sea and 0.12 below before 1 m quantization, retaining the coast mask; the sampled summit is about 147 m and the floor about 62 m down, pinned by a test. `PlanetPlugin` uploads the base once and rewrites the four fine regions of the same buffer when a new set is published (whole, versioned, never partial). Otherwise only view/time uniforms change each frame. Neighbor-height occlusion is baked on the CPU into one scalar sky-visibility value; the RGB propagation kernel is not running in this path.

| Desktop struct | Span | Members/offsets |
| --- | ---: | --- |
| `Cell` / Rust `GpuCell` | 192 B | center ray xyz and height w at 0; six corner rays with adjacent edge height w at 16 (stride 16); degree (low byte) and level (above it)/biome/scalar sky/stable ID at 112; the two owner directions at 128 and 144 with the fine floors of sides 0 and 1 in their w; fine floors of sides 2 to 5 at 160; spare at 176 |
| `Params` / Rust `PlanetParams` | 112 B | clip-from-world matrix at 0; camera vec4 at 64; sun vec4 at 80; radius/count/time/vertices-per-cell vec4 at 96 |
| `DrawArgs` | 16 B | vertex count at 0; atomic instance count at 4; first vertex at 8; first instance at 12 |

Both planet modules use bind group 0. The compute layout binds parameters (0), read-only cells (1), read/write visible IDs (2), and read/write indirect arguments (3). The surface layout binds parameters (0), read-only cells (1), read-only visible IDs (2), and the `fields.png` texture view (3). Texture reads use `textureLoad` on a defined nearest 32×32 source grid per sheet tile; there is no sampler or texture-array importer in this prototype.

`PlanetComputeNode` runs before `CameraDriverLabel`: `clear_indirect` dispatches one workgroup, then `compact_visible` dispatches `ceil(cell_count/128)` workgroups. Each cell can append at most one ID. The host allocates exactly `cell_count` ID slots, which is the capacity invariant this particular compactor relies on; it does not implement the standalone face kernel's overflow/replacement protocol. Draw arguments use zero first vertex/instance. The terrain draw emits 60 vertices per visible cell (cap, six walls and the cut wall of a split midpoint cell); a separate foliage draw emits 108 from vertex 60 for the finest-level cells the compute pass selected within 300 m. A record draws only when its level's band does not cover it and its owner at the level below is drawn at its level; a midpoint cell with one fine owner draws and is split per fragment by nearer owner. Degenerate triangles cover inactive pieces. None of this changes the authoritative topology resolution.

The custom planet draw is queued at the start of Bevy's `Transparent3d` phase but its pipeline uses opaque color output and reverse-Z depth writes. This is an explicit prototype integration detail, not an assertion that it participates in the opaque prepass or supplies an already-resolved scene-color/depth texture to the water port. `SkyMaterial` then uses normal Bevy mesh/view bindings and premultiplied blending with depth writes disabled. Native captures of the revised resolution, terrain and sky confirmed their composition in orbit and near the ground; the optical water path remains separate.

Water cells in `planet_surface.wgsl` draw their **seabed** at `R + height` with the real neighbour heights in the corner record, and the shader applies the port's `exp(-absorption * depth)` tint below the sheet radius. The sheet itself is drawn by `water.wgsl` inside the water composite node after the main pass, reading the resolved scene colour and the main-pass depth. There is no second live water look.

`sky.rs` supplies a five-vec4 (80 B) material uniform containing center/solid radius, atmosphere radius/density coefficients, sun vector/radiance, scatter ratios/anisotropy, and cloud settings. The revised shell radius is **4,800 m**, with clouds at **4,600 m** and exponential density scale height **176 m** (`0.22 × 800 m`). Density tapers smoothly over the outer 28% of the shell. Sun radiance is 3.2, Rayleigh/Mie scales are 0.30/0.018, RGB scattering ratios are `(0.16, 0.52, 1.30)`, and cloud opacity is capped at 0.52; these are artistic parameters, not calibrated physical units. The shader imports `bevy_pbr::forward_io::VertexOutput` and `bevy_pbr::mesh_view_bindings::view`; it uses the default mesh vertex shader and therefore cannot be passed directly to the standalone validator. Near-surface/daylight star suppression fades out by the outer atmosphere boundary. No automatic floating-origin update is installed for the shell: a moved planet must update the mesh transform and center uniform together.

Night-side capture review exposed discrete bands from the original eight midpoint samples combined with binary planet-shadow rejection. The integrated material now uses sixteen view samples and smooth sun visibility based on the sun ray's closest radius to the planet, with an artistic 36–60 m penumbra. Shadowed segments still accumulate extinction. The native night recapture confirmed that the diagonal shadow wedges were removed. The original standalone atmosphere port remains at its documented eight-view/four-sun interface.

Runtime comparisons, including actual Tenebris reference screenshots and the distinction between its prebuilt binary and inspected source revision, are tracked in [tenebris-comparison.md](tenebris-comparison.md).

## Coordinate and color contract

Authoritative positions and celestial ephemerides remain CPU `f64`. Before extraction, compute `planet_center = body_world - local_origin` and `camera_local = camera_world - local_origin` in `f64`, then convert the differences to `f32`. Terrain and water receive the same origin and orientation for these vectors and both matrices. Never subtract a rebased planet center from an unre-based camera.

Geometry stores normalized planet-local corner rays and radial distances in metres. The initial vertex expansion evaluates `planet_center + corner * radius` in `f32`; this is appropriate for the specified kilometre-sized bodies, not an assertion of millimetre precision for astronomical radii. For larger bodies, introduce chunk-relative corner positions and a split high/low translation before increasing planet scale. Rotation from a body's local axes into the active local frame must already be applied consistently to corner rays, sun vectors and shader inputs, or supplied by a future explicit body transform. The current geometry interface assumes aligned axes.

Atmosphere receives camera position relative to the body center and camera/sun vectors in that body's axes. Water carries body-local positions separately for noise so a floating-origin shift does not shift wave phases. Bound the input range and periodically rebase animation time with a continuous phase scheme before long-running sessions; the current shader does not provide phase-continuous time wrapping.

Texture-array albedo should use an sRGB texture view so sampling yields linear color. Lighting, water scene color and scattering are linear; tone mapping belongs to the final camera pass. Use nearest texture filtering for terrain, optional nearest mip selection, and edge-safe tile layers. The terrain alpha output is ordinary opaque alpha; Tenebris's repurposed wetness alpha channel is not carried into Bevy's scene color.

## Standalone voxel face extraction and vertex pulling

The following contract belongs to `hex_faces.wgsl`/`hex_terrain.wgsl`, which the current desktop surface-column pipeline does not dispatch.

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

## Standalone baked RGB and skylight

This section describes `voxel_light.wgsl` and its associated face format. The desktop prototype currently uses the separate CPU scalar sky-occlusion value described above.

Both terrain and light compute use `u32` values whose low 16 bits are `R | (G << 4) | (B << 8) | (sky << 12)`, each channel 0–15. This format is an extension, not bit-compatible with Tenebris's two-channel byte. Migration from old data maps old block intensity through the old torch tint into RGB and keeps the sky nibble. Display intensity is shader-configured; the normalized channel value is not a physical lux measurement.

Light bindings in group 0: parameters (0), `LightCell` metadata (1), neighbors (2), previous values (3), next values (4). `LightCell` is 16 bytes: emission RGB nibbles at offset 0, flags at 4, scalar padding at 8/12. Flags are opaque bit 0, unobstructed direct sky bit 1, frozen halo bit 2. Neighbor ordering matches the face shader.

CPU authoritative terrain/topology determines the direct-sky seeds: a column receives 15 only along its unobstructed path from the build cap. The shader does not scan an entire column or determine occlusion outside the loaded region. `initialize` writes seeds; `relax` writes the channelwise maximum of seeds and neighboring previous values minus one, with saturating subtraction. Opaque cells block propagation but may emit. No invocation writes another voxel, so no atomics are needed in the relaxation pass.

For initial generation or an edit that removes a source/blocks daylight, reseed **both** ping-pong buffers in the affected region before relaxation. Reusing stale light as a maximum would retain phantom light. Supply the same valid, frozen boundary halo in both buffers. CPU seed uploads can initialize known external boundaries; production cross-chunk visual-light halos should use GPU buffer copies/compute exchange without reading whole grids back. The current shader exposes a caller-provided frozen halo and does not implement that cross-chunk exchange or convergence scheduler. The dirty region must encompass the entire changed light influence (15 graph steps for unit attenuation), and direct-sky seed changes may dirty the whole affected shaft. Global edits may require region/chunk expansion, not just the edited voxel.

Run up to 15 ordered relaxation dispatches, swapping distinct previous/next buffers after each. With seeds already initialized, channel values bounded by 15 and unit attenuation, this covers the possible nonzero influence radius; preserve frozen boundaries. The final buffer feeds face extraction binding 7 directly. GPU values remain a visual cache; networking/gameplay/collision must query authoritative CPU state. CPU removal queues and budgeted authoritative light updates remain separate work. Face light currently samples one exposed neighbor, so there is no per-corner light smoothing or ambient occlusion term yet.

## Water integration contract (live)

`water.wgsl` is bound by `planet_water.rs` and runs inside `WaterCompositeNode`, registered on `Core3d` between `EndMainPass` and `StartMainPassPostProcessing`. Design: `openspec/changes/water-composite/design.md`.

`WaterView` is 448 bytes: two 64-byte matrices, then twenty 16-byte vectors in declaration order; `the_uniform_matches_the_wgsl_struct_size` reads the count off the shipped shader and pins the Rust size to it. Group 0 is the uniform, the planet's `Cell` storage buffer, the visibility pass's water cell IDs and a per-cell `vec2` flow buffer (zero-filled; `openspec/changes/water-flow`). Group 1 is the resolved scene colour (filterable float), the main-pass depth (`texture_depth_2d`, or `texture_depth_2d_multisampled` under the `MULTISAMPLED` shader def when the camera runs MSAA) and a linear clamping sampler. The camera must request `TEXTURE_BINDING` on its depth texture (`configure_camera` does).

The node takes `ViewTarget::post_process_write` once: the `compose` entry (fullscreen triangle) writes every pixel of the destination from the source with the underwater fog, distortion and depth blur, then the cap draws into the same destination with `draw_indirect` at offset 32 (the third `DrawArgs`), reading the same source. The cap has no depth attachment: its own reverse-Z discard against the sampled depth is the whole occlusion test a single sheet needs. When rain or the emerge window is non-zero the node takes a second swap and the `lens` entry refracts droplets through the post-water image. All three pipelines are single-sample whatever the main pass's MSAA.

The vertex stage pulls the cap: instance `i` is `water[i]`, eighteen vertices fan the cell's corner rays at the sheet radius (sea level less `depth_offset_m`), a pentagon's sixth triangle collapses to the centre, and the three-sine swell displaces every vertex radially in body-local metres. The camera's side of the surface is `fx.y`, decided on the CPU by `submersion` as 0 / 0.5 / 1 over a band of `swell_amplitude_m + partial_band_m`; `emerge` keeps the drip window. Both are pure and tested.

Reverse-Z: near=1, clear sky=0; opaque geometry covers the water when its depth is larger. Reconstruction applies `local_from_clip` (inverted in f64 on the CPU) to WebGPU's `[0,1]` depth, with image Y inverted into NDC. For samples at infinite sky depth it uses `max_path_m` instead of dividing an infinite-far homogeneous point.

Deliberate differences from `water.fs.glsl`, all recorded in `docs/tenebris-comparison.md`: the fbm gradient is projected off the radial as Tenebris does (the earlier unbound port used the face normal); the foam per-term weights and the specular `sun_tint` are restored (the unbound port had dropped them); torch light has no source here and the term is omitted; the distance fog is fed the terrain shader's own constants from Rust rather than a composite's `FRAME_FOG`; `shoreline_fade_m` and the mobile `horizon_fade_*` are not ported. Not implemented: screen-space caustics (Tenebris has none either), multi-layer transparent sorting, the distant storm shafts. Visual parity against a Tenebris daylight capture has not been judged by the owner; see the comparison document for what the captures show.

## Standalone fullscreen atmosphere integration contract

This interface belongs to `atmosphere.wgsl`; the integrated shell material uses the separate 80-byte `SkyParameters` contract described above.

`AtmosphereView` is 176 bytes: eleven vec4 values, group 0 binding 0. `vertex` emits a fullscreen triangle from `vertex_index`; `fragment` integrates the body-local shell. `radii.z` is dimensionless scale height normalized to the shell thickness, matching the original function, not metres. Nonpositive/empty shells return transparent. Use nonzero camera basis/sun vectors; an atmosphere center is not a valid camera position for this model.

Draw only where reverse-Z scene depth equals its clear value 0, depth writes disabled. Composite the returned scattering as premultiplied RGB with source factor One and destination factor OneMinusSrcAlpha. RGB is the integrated in-scattered radiance; alpha approximates average extinction for the background. The original shader's `1-exp(-color)` mapping and scatter-luminance alpha were replaced to avoid tone-mapping twice in Bevy and to give background attenuation a consistent meaning. There is no physically exact spectral multiple scattering.

The source clamps ground-hit rays and can output opaque atmosphere over ground. This module is deliberately **sky-only**; terrain aerial perspective, partial terrain-depth ray truncation and shared water/terrain fog color evaluation still need a dedicated depth-aware composite. The terrain limb term is ported independently. Multiple visible atmospheres must be composed in distance order with an agreed transmittance model; no planet-order solution is hidden in the shader.

## Remaining volumetric renderer integration and acceptance work

The desktop planet plugin already follows the source swarm's `RenderApp` extraction/prepare/render separation for an immutable surface globe. Extend that wiring to immutable chunk snapshots/revisions, per-chunk buffers, light/face jobs and revision-safe publication. The original five standalone ports still require their own corresponding custom pipelines; they do not use Bevy's `#import` material interface and cannot simply be attached as a `StandardMaterial`.

Before claiming this path is ready, implement and verify:

- Chunk buffer allocation/reclamation, the standalone contracts' bind-group layouts, full biome/texture-array import, adapter limits and a CPU fallback for unsupported adapters.
- Revision-safe chunk publication, old-allocation retirement, device-loss recreation and bounded outstanding work.
- GPU tests comparing isolated hex/pentagon, stacked cells, neighbor occlusion, seam halos, zero capacity, overflow and stale edits against a CPU face oracle.
- RGB/sky addition/removal and chunk-boundary parity tests, including a deep vertical shaft and an opaque edit that closes it.
- Reverse-Z water occlusion/refraction captures, underwater views, origin rebases, pole/terminator views and sky/terrain composition tests.
- GPU timing and memory/triangle/face counters for target 2–12 km diameter bodies, travel/streaming and repeated terrain edits; asynchronous readback only for small diagnostics, never expanded terrain geometry in the main frame loop.

WGSL layout and validation rules: [W3C WGSL specification](https://www.w3.org/TR/WGSL/). The pinned parser is [Naga 27.0.3](https://crates.io/crates/naga/27.0.3); the executable's lockfile makes the language-validation result reproducible.
