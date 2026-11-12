# Design: bevy-0-20

## Context

Sources read on 2026-11-12:

- **Bevy's own pages:**
  - https://bevy.org/learn/migration-guides/0-18-to-0-19/
  - https://bevy.org/learn/migration-guides/0-19-to-0-20/
  - https://bevy.org/news/bevy-0-19/
  - https://bevy.org/news/bevy-0-20/
  - the v0.20.1 tag (https://github.com/bevyengine/bevy/releases/tag/v0.20.1;
    its notes are only a diff link);
  - crates.io for dependency versions.
- **Avian:** https://github.com/avianphysics/avian/releases/tag/v0.8.0.
- **Crab Breeze `main` `8f9ed158`**, read through `git show`, as the worked
  example of the 0.20 APIs.

Nothing was built. Anything marked **unconfirmed** is settled during the
work, not assumed.

## 1. Versions

| Item | Now | Target | Note |
| --- | --- | --- | --- |
| bevy | =0.18.1 | =0.20.1 | 0.20.0 was released 2026-10-08, 0.20.1 on 2026-10-28 |
| wgpu, naga (direct deps) | 27 | 30 | `bevy_render` 0.20.1 needs `wgpu ^30`. Crab Breeze's lock has wgpu 30.0.1 and naga 30.0.1 |
| avian3d | =0.6.1 | =0.8.0 | Needs bevy ^0.20. The `f32` and `simd` features are gone, so `["3d","f32","parry-f32","parallel"]` changes |
| glam (`pbd-core`) | 0.30.10 | 0.33.2 | What `bevy_math` 0.20.1 requires |
| rust-version | 1.89 | 1.97.1 | Bevy 0.20.1's minimum |
| Bevy features | | | Every feature `pbd-app` names still exists |
| WESL test compiler | | `wesl =0.6.0` (`naga-ext`), dev-only | As Crab Breeze pins it |

Avian 0.7 targets Bevy 0.19, so the upgrade can stop at 0.19 (section 4).

## 2. What changes in Pale Blue Dot (read)

Effort: **T** trivial, **M** mechanical, **R** real work.

| Change (version, PR) | Where in PBD | Effort |
| --- | --- | --- |
| Render graph replaced by systems in `Core3d`, taking `ViewQuery` and `RenderContext`, ordered with `.after`, `.before` and `Core3dSystems` sets (0.19, #22144) | `bloom.rs:24,53,70-77,240-250`; `planet_water.rs:26,1625,1649-1659,2070-2077`; `planet_ssr.rs:20,44,57-64,152-163`; `planet.rs:63,306-308,1761-1828` (the cull pass and `shadow::draw_cascades`, before the camera driver) | R |
| `FullscreenMaterial::node_edges`/`run_in` replaced by `schedule_configs` (0.19, #23786) | `tone.rs:44-55` | M |
| Render sets order weakly (0.20, #25128) | The cloud timer read-back in `Cleanup` (`planet_water.rs:2062`); the chain `upload_planet` → `prepare_views` → `prepare_water_views` (`planet.rs:296-301`, `planet_water.rs:2064-2068`) | R (audit) |
| Sorted phases: `add` becomes `add_transient`/`add_retained` (0.19, #22966) | `planet.rs:1617` | T |
| HDR format constants deprecated; use `ExtractedView::target_format` in keys (0.19, #23734, #22683) | `planet.rs:1172,1190-1192,1615`; `planet_water.rs:1485-1488`; `bloom.rs:159` | M |
| `ViewDepthTexture` → `ViewDepthStencilTexture` (0.20, #24725) | `planet_water.rs:36,1652`; `planet_ssr.rs:30,155` | M |
| `depth_write_enabled`/`depth_compare` become `Option` (wgpu 29) | 8 sites, e.g. `planet.rs:1203-1207`, `planet_shadow.rs:257-262`, `planet_water.rs:762`, `sky.rs:270-271` | M |
| `ShaderStorageBuffer` → `ShaderBuffer`, typed `new`, no `set_data` (0.19/0.20) | `field_light.rs:15,123`; `planet_shadow.rs:22,79,101,133-137,381`; `planet_water.rs:1957` | M |
| `#[extract_app(RenderApp)]` on extract derives (0.20, #24419) | 3 `ExtractComponent`, 21 `ExtractResource`. Whether the attribute is required or has a default is **unconfirmed** | T |
| Shaders to WESL (0.20, #25088): `import …;`, `@if()`, `constants::X`, no `#define_import_path`, `.wesl` files, renamed Bevy modules (e.g. `bevy_pbr::render::forward_io`) | 12 of 19 shaders: `field_lit` (16 directives), `water` (7), `sky_atmosphere` (4), `ssr_capture` (3), `planet_surface` (2), and one each in `bloom_lite`, `tone_grade`, `map_image`, `map_live`, `clouds`, `sea`, `sun_shadow`. Unconfirmed: the WESL paths of the fullscreen vertex shader and the UI vertex output, and whether the keep-alive module loads (`sky.rs:194`, `sea.rs:92`, `planet_shadow.rs:223`) are still needed | R |
| naga_oil gone, naga 30 | `shader_tests.rs:10-60` hand-splices `#import`; `include_str!` shader tests; four raw-wgpu GPU test files (86 `wgpu::` uses); `WgpuSettingsPriority::Compatibility` → `WebGPU` (`planet_visibility_tests.rs:45`) | R |
| wgpu 27 → 30, used directly | `planet_cloud_timer.rs:21` (`QuerySet`, `RenderPassTimestampWrites`) | M |
| `Tonemapping::None` is a full passthrough; `Linear` keeps the old behaviour (0.20, #25499) | `tone.rs:225` | M plus a look check |
| Bevy `Bloom` luma in linear space (0.19, #22561) | `tone.rs:185-192` (the non-lite option only) | T plus a look check |
| `shadows_enabled` → `shadow_maps_enabled` (0.19) | `desktop/scene.rs:25` | T |
| Parley text: `font_size` → `FontSize::Px`, `TextLayout::new_with_*` renamed (0.19/0.20) | 39 `font_size` sites in 15 files; `compass_bar.rs:111`, `map_sites.rs:259` | M plus a layout check on every screen |
| `BorderRadius` 2D and non-const; `Interaction` deprecated (0.20) | 7 and 30 sites (`map_sites.rs:191-239`, `menu.rs`) | T to M |
| Resources are components (0.19) | Every `Query<Entity…>` already has a `With<>` filter | T |

Not used by Pale Blue Dot, so no work: Bevy's atmosphere, fog,
transmission, TAA, skybox, glTF scenes and picking.

## 3. Crab Breeze as the worked example (read, `main`)

- **A pass as a render system:** `crates/cb-app/src/water_render.rs:256-275`.
  - `water_fog.in_set(Core3dSystems::MainPass).after(main_transmissive_pass_3d).before(main_transparent_pass_3d)`.
  - `water_post.in_set(Core3dSystems::PostProcess).before(tonemapping)`.
  - Body at `:1063-1077`:
    `fn water_post(view: ViewQuery<(&ViewTarget, &ViewDepthStencilTexture, &WaterPostGpu)>, …, ctx: RenderContext)`.
  - Pipeline keyed on `view.target_format` (`:894`).
  - This is the shape the water composite and the SSR capture take.
- **A second pass:** `sun_flare.rs:110-120` (post-process, before
  tonemapping) and `:590-593`. This is the shape `LiteBloom` takes.
- **Ordering against Bevy's private systems:** `water_render.rs:1110-1140`,
  `FogAfterSky`. It is a schedule build pass that adds an edge by system
  name, because `render_sky` is private and weak ordering let the fog run
  first. The result was a one-frame sky flicker, 50 times in 4.4 minutes.
  Pale Blue Dot's water composite sits in the same place, between the main
  pass and post-processing.
- **Material extension and `ShaderBuffer`:** `building_state.rs:19,36-47,70,105,184`;
  `people_vat.rs:45,72,312`.
- **WESL:**
  - `assets/shaders/building_state.wesl`, which imports
    `bevy_pbr::render::forward_io` and uses
    `@group(constants::MATERIAL_BIND_GROUP)` and `@if(VERTEX_UVS_B)`;
  - `water_post.wesl` (`import super::water_common::…`, `@if(MULTISAMPLED)`).
- **The shader test harness:** `crates/cb-app/src/shader_check.rs`. It
  compiles each shader with `wesl` and Bevy's options, then runs naga's full
  validation. It resolves `bevy_pbr::…` from Bevy's sources in Cargo's
  cache. It replaces `shader_tests.rs`'s splicing.
- **Core3d order in 0.20.1** (`bevy_core_pipeline/src/core_3d/mod.rs:145-167`):
  - prepasses in `Prepass`;
  - opaque and transparent in `MainPass`;
  - `tonemapping` in `PostProcess`;
  - `upscaling` after it.

  `PlanetComputeNode` (the GPU cull) moves into the `RenderGraph` schedule
  (`RenderGraphSystems::{Begin, Render, Submit, Finish}`), before
  `camera_driver`. Whether `camera_driver` is public to order against is
  **unconfirmed**.

## 4. Order of work

Go in two hops. Each hop builds, passes the tests and gets its captures
before the next one starts.

0. **Prepare** (0.5 day).
   - Rust 1.97.1.
   - Run and record every generator version's ground digest on 0.18.
   - Take the before captures: water (dry, under, straddling), the SSR
     puddles, bloom on and off, tone, clouds, the night lamps, the map, the
     menus and the HUD.
   - Build the 0.18 baseline exe and keep it for the A/B.
1. **Bevy 0.19, Avian 0.7, wgpu and naga 29.** WGSL keeps its naga_oil
   syntax in this hop, so render breakage is isolated from the shader port.
   - The four passes become systems, plus `ToneGrade`'s ordering (2 days).
   - Phases, HDR keys, depth `Option`s, `ShaderBuffer` and
     `shadow_maps_enabled` (1 day).
   - Parley text (1 day).
   - Captures against the before set (0.5 day).
2. **Bevy 0.20.1, Avian 0.8, wgpu and naga 30, glam 0.33.**
   - WESL for the twelve shaders (1.5-2 days).
   - Port `shader_check.rs` and rework the GPU tests (1.5-2 days).
   - `extract_app`, the depth-stencil renames, `BorderRadius`, the
     `Tonemapping::None` decision, and the weak-ordering audit with an
     explicit edge for every pass (1 day).
   - Avian's features, then walking, swimming, boats and flight on their
     scripted runs (0.5 day).
3. **Measure** (1 day). See section 6.

Total: 9-12 working days, about 2.5 weeks with risk buffer. **Estimate.**

## 5. Risks

- **Saved games.** glam 0.33 or a changed float path in a dependency could
  move generation by an ulp. CLAUDE.md requires every existing world to keep
  its version cell for cell. The digest tests are the gate. A moved digest
  stops the upgrade and is written up and asked about. It is never re-pinned.
- **Pass order.** Weak ordering plus passes-as-systems can reorder the
  water composite, the SSR capture and bloom against Bevy's private systems
  (Crab Breeze's `FogAfterSky`). Every pass gets an explicit edge.
  `FogAfterSky`'s by-name edge is the fallback where Bevy keeps a system
  private. A flicker test is a `--capture` sequence across a weather change,
  checked frame by frame.
- **Shader port size.** `field_lit` follows Bevy's renamed PBR modules and
  prepass conditionals. `planet_surface` and `water` are large, but only
  their imports change. The test harness is mostly rewritten.
- **Look changes to capture:**
  - Parley text layout;
  - `Tonemapping::None` (negatives no longer clamped before `tone_grade`);
  - the dimmer Bevy `Bloom`;
  - Avian 0.8 contacts and CCD under the walker and the boats.

  Any visible difference goes to the owner with shots before it lands.
- **A possible regression** (0-18-to-0-19 guide): every system now gets a
  task even when it would skip. Pale Blue Dot has many `Option<Res>` and
  `Single` systems. If the trace shows it, run conditions are the fix.
- **The loading screen and `lighting-perf`** must have landed first, or
  their passes get ported twice.

## 6. Measurement

The upgrade is its own A/B, run on the owner's desktop on a quiet machine
(CLAUDE.md, the `perf-measure` skill).

- **Variants:** `before` is the last 0.18 exe on its own assets. `after` is
  the first 0.20.1 exe, with the same game logic and assets.
- **Scenarios:** `clouds`, `storm`, `far-side`, `walk`, `rain-stroll`,
  `town`. Three repeats, interleaved, uncapped.
- **Read:**
  - p50, p95, p99 and the over-16.7 ms count;
  - GPU total;
  - the town walk's trace (`--features trace`) for Extract,
    `prepare_erased_assets`, `batch_and_prepare` and `prepare_windows`.
    This is where Bevy's 0.19 and 0.20 render-world work should show if it
    helps Pale Blue Dot.
- **Report:** `docs/benchmarks/<date>-bevy-0-20/`, with the HTML page
  published and linked from CLAUDE.md.
- **Re-run the frame-latency A/B** (`crab-breeze-lessons` rank 1) on 0.20.
  The swapchain path changes with wgpu 30.

What Bevy claims is not counted as Pale Blue Dot's. Pale Blue Dot's terrain
is a custom phase item with its own GPU cull, and gains nothing from
Bevy's mesh work. Only the towns, the vehicles, the fish and the UI can.
Expected, an estimate: 0.5-2 ms on `town`, about 0 elsewhere.

## Performance-relevant changes in 0.19 and 0.20

| Change | Source | Could reach PBD through |
| --- | --- | --- |
| Render big scenes faster: change lists, sparse mesh uniforms, parallel collection, GPU bin unpacking (`bevy_city` 19.3 → 11.8 ms) | bevy-0-19 (#22966, #22988, #23242) | Town meshes; Extract (1.45 ms on the walk) |
| Partial bindless, less bind-group overhead (Bistro +46 % on a 5060 Ti) | bevy-0-19 (#23436) | `FieldLit` and the standard materials |
| `BindGroupBuilder`, fewer allocations preparing materials | 0-19-to-0-20 guide | `prepare_material_bind_groups` |
| Weakly ordered render sets | bevy-0-20 (#25128) | Render-thread overlap (8.1 ms on the walk) |
| Per-column change ticks (132x in GPU mesh extraction) | bevy-0-20 (#25157) | Extraction |
| Retained UI rendering | 0-19-to-0-20 guide | The HUD, compass bar (about 40 nodes a frame), map and menus |
| Avian 0.8: contact recycling, contiguous solver bodies | avian v0.8.0 | Physics (0.7 ms on the walk) |
| Mesh shaders in the pipeline cache; automatic mips and BCn in the asset processor | bevy-0-20 (#25627, #24223) | Nothing for free; later options |
