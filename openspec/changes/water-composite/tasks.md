# Tasks

## 1. The node
- [x] `WaterCompositeNode` between `EndMainPass` and
      `StartMainPassPostProcessing`, taking `ViewTarget` and `ViewDepthTexture`,
      with a `MULTISAMPLED` shader def keyed on the view's sample count.
- [x] Compose pipeline (fullscreen triangle, the `compose` entry of
      `water.wgsl`, one file for the whole water system) writing the
      post-process destination from the source and depth.
- [x] The water cap draw in the same destination after compose, reading the
      same source and depth; no depth attachment.
- [x] Lens pipeline on a second swap, run only when rain or emerge is non-zero.

## 2. Submersion
- [x] The submersion state (dry / straddle / under, emerge window) computed on
      the CPU from the camera's body-local position and `surface_height`, in
      the render world's prepare so it needs no extraction; `submersion` and
      `emerge` are pure and tested for the three states and the 2.6 s dry-off.

## 3. The terms
- [x] Underwater fog with the analytic sky exit distance, the waterline mask,
      distortion and the depth blur, each read from `water.ron`.
- [x] Heartfelt droplets and emerge drips, `rain_lens_*` from `weather.ron`.

## 4. Proof
- [x] Captures: `--view dive` (eye 3 m under), `--view wade` (eye in the
      band), the shore series, and a rain frame once `weather-rain` lands.
- [x] The tri-state requirement moved into `openspec/specs/planet/water/`
      with its tests. The fog and lens requirements stay here: they are
      validated by capture on lavapipe and pinned by no unit test, and the
      owner has not yet judged the look.
