# Tasks

Starts after `lighting-perf` and the loading screen have landed.

## 0. Read (done, 2026-11-12)

- [x] 0.1 Migration guides, release notes, crates.io versions, and
      Crab Breeze's 0.20 code read. Each Pale Blue Dot site affected is
      listed (`design.md` section 2).

## 1. Prepare

- [ ] 1.1 Rust 1.97.1 toolchain. `rust-version` updated in the workspace.
- [ ] 1.2 Record every generator version's ground digest on 0.18, and
      confirm a digest test pins each one.
- [ ] 1.3 Before captures: water dry, under and straddling; SSR puddles;
      bloom on and off; tone; clouds; night lamps; the world map; the
      menus; the HUD. Keep the 0.18 release exe as the A/B baseline.

## 2. Hop 1: Bevy 0.19, Avian 0.7, wgpu and naga 29

- [ ] 2.1 `LiteBloom`, the water composite, the SSR capture, the planet
      cull and the shadow cascades become render systems with explicit
      edges. `ToneGrade` uses `schedule_configs`.
- [ ] 2.2 Sorted phases, HDR format keys, depth `Option`s, `ShaderBuffer`
      and `shadow_maps_enabled`.
- [ ] 2.3 Parley text, `FontSize` and `TextLayout`.
- [ ] 2.4 `cargo fmt`, `clippy -D warnings` and `cargo test --workspace`
      pass. The digests are unchanged. Captures match 1.3, or the
      differences are shown to the owner.

## 3. Hop 2: Bevy 0.20.1, Avian 0.8, wgpu and naga 30, glam 0.33

- [ ] 3.1 The twelve shaders go to WESL (`.wesl`, `import`, `@if`,
      `constants::`). Confirm the paths of the fullscreen and UI modules,
      and whether the keep-alive module loads are still needed.
- [ ] 3.2 Port Crab Breeze's `shader_check.rs` and replace the splicing in
      `shader_tests.rs`. Bring the four GPU test files to wgpu 30.
- [ ] 3.3 `extract_app`, `ViewDepthStencilTexture`, `BorderRadius` and
      `Interaction`. Decide on `Tonemapping::None` versus `Linear` from
      captures.
- [ ] 3.4 The weak-ordering audit: an explicit edge for every pass and for
      the cloud timer read-back. Run a flicker test, a `--capture`
      sequence across a weather change, checked frame by frame.
- [ ] 3.5 Avian 0.8 features. Scripted walking, swimming, boat and flight
      runs, compared with 0.18.
- [ ] 3.6 All checks and digests pass, and captures match 1.3. Update
      `openspec/config.yaml` and the CLAUDE.md lines that name Bevy and
      Avian versions.

## 4. Measure

- [ ] 4.1 `perf_suite.py`, before (0.18) against after (0.20.1), on
      `clouds`, `storm`, `far-side`, `walk`, `rain-stroll` and `town`.
      Three repeats, interleaved, on a quiet machine.
- [ ] 4.2 A trace of the town walk on both, comparing Extract,
      `prepare_erased_assets`, batching and `prepare_windows`.
- [ ] 4.3 Re-run the frame-latency A/B (`crab-breeze-lessons` 1.2) on 0.20.
- [ ] 4.4 Report in `docs/benchmarks/<date>-bevy-0-20/`, the HTML page
      published, its link added to CLAUDE.md.
- [ ] 4.5 The owner's gate: screenshots before and after. The batch video
      task stays open (CLAUDE.md).
