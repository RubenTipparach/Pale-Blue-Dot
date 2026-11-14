# Tasks

## 0. Planning (this step)
- [x] Port the UI skills from Fleet Command (`a4a02f6`): `game-ui-frontend`,
      `ui-ux-pro-max`, `blender-ui-panels`, `blender-organic-panels`,
      `panel-textures`, each with an "In Pale Blue Dot" note and a description
      that triggers here; provenance in `.claude/skills/PROVENANCE.md`.
- [x] Check what runs: `ui-ux-pro-max`'s search (Python 3.13, standard
      library); Blender through the `bpy` 4.5.16 wheel under Python 3.11 (CPU,
      no GPU in the cloud container).
- [x] Study the UI as it is (`desktop/*.rs`, `loading.rs`, the screenshots) and
      the mockups' panels that will become screens.
- [x] Draw the kit: `tools/art/ui_art.py` into `docs/mockups/ui/` with
      `slices.json` (40 pieces, every colour Sweaft-64, contrast measured).
- [x] Render the frame and the rock slab in Blender and pixelate them:
      `tools/art/ui_panels_blend.py`.
- [x] The mockup `docs/mockups/ui.html` ("Stone and Timber"): kit, HUD by day
      and night before and after, pack, pause and settings, map, station,
      talk and trade, loading, phone width.
- [x] Verify it in headless Chromium (SwiftShader, no GPU): no console errors,
      every tab draws, no WebGL context, no horizontal scroll at 390 px, every
      9-slice and pixel image at a whole-number scale and pixelated. Shots in
      `docs/screenshots/mockup-ui/`.
- [ ] Publish the page, add it to the mockup tracker, put Q1 and Q2 in the
      survey with their screenshots.
- [ ] The owner answers Q1 and Q2 and approves the mockup for build.

## 1. The kit in the game (waits for the owner's go-ahead)
- [ ] Confirm the Bevy 0.18.1 names (`ImageNode`, `NodeImageMode::Sliced`,
      `TextureSlicer`, `SliceScaleMode`, `UiScale`) against the crate.
- [ ] `assets/ui/` (the kit's PNGs and `slices.json`, copied from
      `docs/mockups/ui/`), `assets/fonts/`
      (Pixelify Sans, VT323, their OFL texts).
- [ ] `desktop/ui_kit.rs`: `UiKit` (handles, nearest sampler, margins read
      from `slices.json`), the tokens, `nine(piece)`.
- [ ] Integer `UiScale` from the window height; native-pixel constants.
- [ ] Test: no two opposite margins in `slices.json` add up to more than the piece; every PNG
      is on the palette with alpha 0 or 255 (the art file itself, not a copy).
- [ ] Capture: a frame at scale 2 draws its corners 28 px, pixel exact.

## 2. Screens, one at a time, each with day and night captures
- [ ] Pause, settings, saves (`menu.rs`), plaques in every state; delete the
      palette copies as each file moves to the tokens.
- [ ] Slots, tool slot, pack (`slots.rs`, `equipment.rs`, `pack.rs`), tooltips.
- [ ] Compass bar (`compass_bar.rs`).
- [ ] Map frame and overlay drawer (`map_screen.rs`, `overlay_ui.rs`).
- [ ] Loading screen (`loading.rs`), its images and fonts in the prewarm;
      no `LATE_PIPELINE`.
- [ ] Guide, time and weather bars (`guide.rs`, `time_ui.rs`, `weather_ui.rs`).
- [ ] `perf_suite.py` on the owner's desktop, before and after.

## 3. Video (batched with the others)
- [ ] The step's video, when the owner calls for the batch.
