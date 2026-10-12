# Design: one node, three passes, one submersion state

## Where it sits in Bevy's graph

Bevy 0.18's `Core3d` graph runs `MainOpaquePass`, `MainTransparentPass`,
`EndMainPass`, then post-processing. The planet, sky and water all draw
opaque colour with reverse-Z depth in the main pass. `WaterCompositeNode` is
registered between `EndMainPass` and `StartMainPassPostProcessing`, so it sees
the finished scene colour and depth and runs before bloom and tone mapping,
exactly where Tenebris's compose sits relative to its swapchain.

`ViewTarget::post_process_write` hands the node a source (the scene) and a
destination. Tenebris needs the scene readable while the water writes; the
swap gives that for free: compose and the cap draw both **read the source**
and **write the destination**. The lens pass needs the post-water image, so it
takes a second swap and reads what the first wrote. When rain and emerge are
both zero the second swap is skipped and the node costs one blit.

## Depth

The main pass depth is `ViewDepthTexture`, multisampled when MSAA is on (the
cameras here run `Msaa::Sample4`). The passes sample it with
`texture_depth_multisampled_2d` under a `MULTISAMPLED` shader def, sample 0,
and `texture_depth_2d` otherwise; the pipeline key carries the sample count.
The water draw has **no depth attachment**: `water.wgsl` already discards
where the scene is nearer, which is the whole of the occlusion test a single
sheet needs, and it means the same texture is never bound as attachment and
sampler at once.

## Submersion, on the CPU

Tenebris asks the camera's voxel (`Block::Water`) and a band around the wave
height at the eye. This world has no water voxels yet, so the equivalent is
the pure functions it does have: the camera is over water when
`surface_height(direction) < 0`, and its radius against the sea radius `R`
says how deep. The state is:

| state | value | condition |
| --- | ---: | --- |
| dry | 0 | not over water, or `r > R + band` |
| straddling | 0.5 | over water and `abs(r - R) <= band` |
| under | 1 | over water and `r < R - band` |

`band` is `swell_amplitude + partial_band_m`, so the analytic wave function is
not written a second time in Rust; the surface band is simply wide enough to
contain it. A cave below sea level cannot exist in a heightfield, so the
"caves stay dry" clause of Tenebris's rule is satisfied by construction and is
noted here so it is not lost when the voxel engine arrives.

## The compose pass, term for term

Ported from `composite.fs.glsl` with `fx_params.z` the tri-state above:

- **Underwater fog**: `mix(deep, scene, exp(-absorption * travel))` weighted by
  `wet`. Travel is the view distance to geometry, reconstructed through
  `local_from_clip` (reverse-Z, `[0,1]` depth) rather than Tenebris's
  OpenGL linearisation. For sky pixels it is the analytic exit distance
  `gap / d_up` off the mean sea sphere, saturating to a far constant for rays
  that never rise, which is the rule that keeps the far underwater from
  reading as a bright hole near the surface.
- **Waterline mask**: geometry pixels are wet only below the sea radius,
  smoothed over `partial_band_m`; straddling, sky pixels murk by ray rise so
  only the near-level rays that travel metres of water do.
- **Distortion**: the sin/cos UV wobble, `underwater_distortion`, on wet pixels.
- **Atmospheric fog gating**: this project's air fog lives in the terrain
  shader, not the composite, so the gate is applied the other way round: the
  compose pass does not add air fog, and the water fog simply owns its pixels.
- **Depth blur**: the 17-tap two-ring blur on the distant background, `wet_blur`,
  active while rain or the emerge window is.

## The lens pass

Martijn Steinrucken's "Heartfelt" droplets as Tenebris carries them: static
drops, two falling layers with trails, refraction only, masked above the
waterline by ray direction while straddling, skipped fully under. Emerge drips
are the same kernel at fixed layer weights for `DRY_SECONDS` (2.6 s) after the
camera leaves the water, re-armed while straddling. The `rain_lens_*` knobs
are the weather field's and are read from `weather.ron`.

## Configuration

`water.ron` carries the compose knobs beside the cap's: `deep_color`,
`absorption`, `underwater_distortion`, `wet_blur`, `partial_band_m`. A missing
field inherits the code default (`#[serde(default)]`); a present zero is zero.
