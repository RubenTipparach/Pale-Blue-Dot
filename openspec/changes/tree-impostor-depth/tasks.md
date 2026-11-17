## 1. Measure

- [x] 1.1 How an impostor is lit (design 1): the terrain's fragment, one normal and one shade per tree
- [x] 1.2 Before stills: the owner's view, 120 m up, a low sun, evening, night, the fields, and the hand-off rings forced at noon and 07:30

## 2. Core

- [x] 2.1 `tree_bake::bake_with_relief`: a normal (in the view's level frame) and AO strip from the same rasterisation; `bake` unchanged
- [x] 2.2 `tree_kit::build_kit`: each strip followed by its relief, `IMPOSTOR_RELIEF` words on
- [x] 2.3 Test: the relief covers exactly the drawn pixels, with unit normals, outward on an oak and a kapok, skyward on the whole

## 3. App

- [x] 3.1 `planet_surface.wgsl`: the impostor's vertex normal is its view's level direction; `impostor_relief` rebuilds the pixel's normal and shade
- [x] 3.2 `tree_impostor_relief` in `scatter.ron` and `ScatterSettings` (0..1, validated), in `light_ease.w`
- [x] 3.3 Test: `IMPOSTOR_RELIEF` is the same in the shader and the core

## 4. Verify

- [x] 4.1 After stills of the same frames, side by side, `docs/screenshots/tree-impostor-depth/`
- [x] 4.2 A launch to READY with no `LATE_PIPELINE`; the capture runs' late pipelines unchanged
- [ ] 4.3 `perf_suite.py` before and after on the release build (owed: the Mac was busy, design 4)
- [ ] 4.4 The owner judges the look; then the requirement moves to `openspec/specs/planet/flora`
