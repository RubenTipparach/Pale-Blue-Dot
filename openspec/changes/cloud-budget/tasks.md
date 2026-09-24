# Tasks

## 1. Measure
- [x] Bench at 200/450/650/900 m: shipped, clouds off, rain off (proposal).
- [x] Break the clouds pass down: Worley off, Worley off in the light march,
      no light march, 8 steps.

## 2. Rain
- [x] Streak coordinates fixed in the world, two scales blended by distance.

## 3. Clouds
- [x] Cellular noise baked into a tileable 3D texture; test that it tiles.
- [x] Light march without the cellular texture.
- [x] March by length, with a step cap and the footprint-or-step LOD.
- [x] Reproject the history by the cloud's coverage-weighted depth.
- [x] March at `cloud_render_scale`, composite at full resolution with the
      pixel's own span test.
- [x] `cloud_render_scale`, `cloud_step_m`, `cloud_max_steps` in
      `weather.ron`, validated.

- [x] Cloud past the base's horizon is marched (the gap under the base).
- [x] Depth-aware upsample: cloud distance and march limit per texel.
- [x] Light march on the coarse shape, 4 + 2 steps.

## 4. Verify
- [x] Re-run the bench; choose defaults for 120 fps at the dearest height.
- [x] Captures at the same heights; the seam and the fur by eye.
- [x] fmt, clippy, tests, `openspec validate --all`.

## 5. Not done
- [ ] The rain fix judged in a moving capture. The capture harness takes stills,
      and a still cannot show the fault. The fix is argued from the
      coordinate: every term of the streak noise's input is now a function
      of the point and the clock.
- [ ] Requirements into `openspec/specs/`: they are verified by captures and
      the bench, not yet by a test, so they stay in this change.
