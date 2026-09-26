# Design: horizon-haze

## 1. The air

A new WGSL module, `pbd::air` (`assets/shaders/air.wgsl`), imported by the
terrain, water, clouds and sky shaders. It has one uniform block's worth of
inputs, fed from one Rust source (the constants now in `planet_water.rs` and
as literals in `planet_surface.wgsl`):

| Input | Unit | Today |
| --- | --- | --- |
| density at the sea | 1/m | 0.00036 (`FOG_DENSITY_PER_M`) |
| scale height | m | 1050 (`FOG_HEIGHT_M`) |
| day and night colours | linear RGB | `FOG_DAY_SKY`, `FOG_NIGHT_SKY` |
| terminator band | sun elevation | `TERMINATOR` |
| cover greying and dimming | | `rain[5]` in the terrain's params |
| dusk tint | linear RGB | the sky shader's `(2.1, 0.65, 0.24)` mix at 0.65 |

```text
air_depth(eye, dir, d)  = d * density * simpson5(exp(-altitude / scale_height))
air_colour(point, sun)  = mix(night, day, elevation) greyed and dimmed by cover,
                          times the dusk tint where the sun is low
```

Everything that hazes calls these. The clouds' `cloud_haze` multiplies
`air_depth` for clouds only, as it does today, until the owner settles it.

## 2. Clouds

`clouds` in `water.wgsl`, where it composites:

```text
T      = exp(-cloud_haze * air_depth(eye, dir, at))
behind = scene * (1 - a) + T * cloud.rgb + a * (1 - T) * air_colour(eye + dir*at, sun)
```

`cloud.rgb` is premultiplied and `a` is the cloud's coverage. Today the
whole premultiplied cloud, coverage included, is scaled by `T`. The near
field and the upsampling are unchanged.

## 3. The sky

`sky_atmosphere.wgsl`, after its scattering: `sky = mix(sky, air_colour(...),
1 - exp(-air_depth(camera, dir, end)))` with `end` the shell's far side or
the ground. Opacity rises with it, so where the haze covers the sky, stars do
not show through it. Low down, near the horizon, the ray runs through
kilometres of air and the sky is the haze colour. Overhead, and from high
up, the air is thin and the sky is as today.

If the owner keeps the ground-to-space transition where it is (proposal,
decisions), the haze's density above the shell's top is tapered to zero with
the sky's own `outer_taper`, and the veil's fade is left alone.

## 4. Terrain and water

`planet_surface.wgsl` and `distance_fog` in `water.wgsl` replace
`(1 - exp(-d * density)) * exp(-eye_altitude / H) * 0.55` with
`1 - exp(-air_depth(eye, dir, d))` toward `air_colour`. The 0.55 cap exists
because the old form had no altitude along the path; it is kept as the fog's
maximum (`fog_max`, `limits.x`) so the near ground keeps its contrast.
The sky's own mask (`skylight`, no haze in a cave) stays.

## Verification

- Stills, before and after, at: the owner's ring frame (`--route clouds
  --frames 1300`); 1500 m looking at the limb (`--view column --height 1500
  --pitch -32`); orbit (`--view orbit`); the deck at dusk (`--time 20.5`);
  the ground under broken cloud (`--route clouds --frames 200`); the ground
  at noon. The far deck from inside the layer reads as pale haze, and at the
  limb a cloud and the sky behind it are within a few grey levels of each
  other across the fade.
- `cloud_haze` at 1, 2 and 3 on the ring frame and the 1500 m still, for the
  owner.
- A GPU test that the Rust constants and the WGSL `pbd::air` inputs agree
  (the layout test pattern `the_uniform_matches_the_wgsl_struct_size` uses).
- The perf suite, old against new, on `clouds`, `cloud-hop`, `far-side` and
  `walk`: the sky pass gains five exponentials a pixel, and the terrain loses
  none.
