# Proposal: Bevy 0.18.1 to 0.20.1

## Why

Pale Blue Dot is on Bevy 0.18.1, Avian 0.6.1 and wgpu 27. Crab Breeze, the
same owner's other game, is on Bevy 0.20.0, Avian 0.8.0 and wgpu 30, and
runs well on the same desktop (`crab-breeze-lessons`). Bevy 0.20.1 was
released on 2026-10-28.

The two releases since 0.18 carry render-world speedups aimed at the cost
Pale Blue Dot's towns pay:

- Bevy 0.19, "Render big scenes faster": change lists instead of tick
  scans, sparse mesh-uniform uploads, and parallel mesh collection. The
  `bevy_city` static scene went from 19.3 to 11.8 ms.
- Bevy 0.19, partial bindless and less bind-group overhead.
- Bevy 0.20: `BindGroupBuilder`, per-column change ticks, weakly ordered
  render sets, and retained UI rendering.

The town walk measured Extract at 1.45 ms and
`prepare_erased_assets<FieldLit>` + `prepare_material_bind_groups` at
1.45 ms (`town-frame-time/design.md`).

The upgrade also keeps Pale Blue Dot within reach of the ecosystem
Crab Breeze already uses (Avian 0.8, `bevy_hanabi` on 0.20) and of the
shader language, WESL, that Crab Breeze's shaders are written in.

These are Bevy's numbers on Bevy's scenes. Nothing here is a Pale Blue Dot
speedup until the A/B says so.

## When

After `lighting-perf` and the loading-screen work land. Then the
upgrade's own A/B is not confounded with theirs, and the loading screen's
pipeline handling is ported once rather than twice. It is measured as its
own sitting: the last 0.18 exe against the first 0.20 exe, same commit of
game logic.

## What changes

- **Dependencies:**
  - `bevy` 0.20.1;
  - `avian3d` 0.8.0, whose features lose `f32` and `simd`;
  - `wgpu` and `naga` 30 (the timer and the GPU tests use them directly);
  - `glam` 0.33.2 in `pbd-core`;
  - `rust-version` 1.97.1.
- **The four custom render passes become render systems.** Bevy 0.19
  replaced the render graph (`ViewNode`, `add_render_graph_node`, `Node3d`
  edges) with systems in the `Core3d` schedule. The four are:
  - bloom;
  - the water composite;
  - the SSR capture;
  - the planet's cull pass and the shadow cascades.

  `ToneGrade`'s `FullscreenMaterial` ordering moves to `schedule_configs`.
- **Twelve shaders move to WESL** (Bevy 0.20 drops naga_oil): import
  syntax, `@if` conditionals, `constants::` defines and Bevy's renamed
  module paths. The seven shaders with no directives stay plain WGSL.
- **The shader test harness is replaced** by a port of Crab Breeze's
  `shader_check.rs`, which compiles WESL with Bevy's options and then
  validates with naga. The four raw-wgpu GPU tests are brought to wgpu 30.
- **Mechanical renames** across the crates. Among them:
  - depth-stencil types;
  - `ShaderStorageBuffer` to `ShaderBuffer`;
  - HDR format keys;
  - `shadow_maps_enabled`;
  - `#[extract_app(RenderApp)]`;
  - `FontSize`;
  - `TextLayout`;
  - `BorderRadius`;
  - `Option` depth-compare fields.
- **Ordering made explicit.** Bevy 0.20 orders its render sets weakly, so
  every pass and every render-world system whose order matters gets an
  explicit edge. Crab Breeze needed one (`FogAfterSky`) after a one-frame
  sky flicker.

## What does not change

- **No picture change is intended.** Three places where Bevy changed
  behaviour are checked with before-and-after captures:
  - text layout under Parley;
  - `Tonemapping::None`, now a full passthrough;
  - Bevy's `Bloom`, whose luma is now linear (only the non-lite option).

  Any difference goes to the owner with the shots.
- **Saved games and generator versions.** Every generator version's ground
  digest must be bit-identical after the upgrade, including the move to
  glam 0.33. A digest that moves blocks the upgrade. It is written up, not
  re-pinned (CLAUDE.md "Saved games survive every change").
- **Physics feel.** Avian 0.8 reworks contacts and CCD. Walking, swimming,
  the boats and flight are checked against the same scripted runs.

## Impact

- `Cargo.toml`, `crates/*/Cargo.toml`.
- `crates/pbd-app/src/{bloom,tone,planet,planet_ssr,planet_water,planet_shadow,planet_cloud_timer,field_light,sky,sea,desktop}.rs`,
  the UI modules under `desktop/`, and `shader_tests.rs` and the `*_gpu_tests.rs`.
- Twelve files in `assets/shaders/`, renamed to `.wesl`.
- `openspec/config.yaml` (it names the Bevy and Avian versions).
- Estimated effort: 9-12 working days, about 2.5 weeks with risk buffer
  (`design.md` section 5).
