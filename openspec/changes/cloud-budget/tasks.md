# Tasks

## 1. Measure
- [x] Bench at 200/450/650/900 m: shipped, clouds off, rain off (proposal).
- [x] Break the clouds pass down: Worley off, Worley off in the light march,
      no light march, 8 steps.

## 2. Rain
- [ ] Streak coordinates fixed in the world, two scales blended by distance.

## 3. Clouds
- [ ] Cellular noise baked into a tileable 3D texture; test that it tiles.
- [ ] Light march without the cellular texture.
- [ ] March by length, with a step cap and the footprint-or-step LOD.
- [ ] Reproject the history by the cloud's coverage-weighted depth.
- [ ] March at `cloud_render_scale`, composite at full resolution with the
      pixel's own span test.
- [ ] `cloud_render_scale`, `cloud_step_m`, `cloud_max_steps` in
      `weather.ron`, validated.

## 4. Verify
- [ ] Re-run the bench; choose defaults for 120 fps at the dearest height.
- [ ] Captures at the same heights; the seam and the fur by eye.
- [ ] fmt, clippy, tests, `openspec validate --all`.
