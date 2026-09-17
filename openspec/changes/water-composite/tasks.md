# Tasks

## 1. The node
- [ ] `WaterCompositeNode` between `EndMainPass` and
      `StartMainPassPostProcessing`, taking `ViewTarget` and `ViewDepthTexture`,
      with a `MULTISAMPLED` shader def keyed on the view's sample count.
- [ ] Compose pipeline (fullscreen triangle, `water_composite.wgsl`) writing
      the post-process destination from the source and depth.
- [ ] The water cap draw in the same destination after compose, reading the
      same source and depth; no depth attachment.
- [ ] Lens pipeline on a second swap, run only when rain or emerge is non-zero.

## 2. Submersion
- [ ] `WaterCamera` state (dry / straddle / under, emerge window) computed on
      the CPU from the camera's body-local position and `surface_height`,
      extracted to the render world; a test pins the three states and the
      2.6 s dry-off.

## 3. The terms
- [ ] Underwater fog with the analytic sky exit distance, the waterline mask,
      distortion and the depth blur, each read from `water.ron`.
- [ ] Heartfelt droplets and emerge drips, `rain_lens_*` from `weather.ron`.

## 4. Proof
- [ ] Captures: `--view dive` (eye 3 m under), `--view wade` (eye in the
      band), the shore series, and a rain frame once `weather-rain` lands.
- [ ] Move the requirements below into `openspec/specs/planet/water/` in the
      commit that makes them true.
