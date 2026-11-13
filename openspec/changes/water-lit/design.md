# Design: the water is lit, not luminous

## 1. Crab-breeze's water, read (2026-11-13, read-only)

Crab-breeze ported this water in `829f79bf` and changed its look in
`2751b86c` (the owner: "darker and greener"), `c1381050` (a dark navy open
sea that greys to the horizon, teal shallows), `88c61f2f` and `b5033bd3`.
Its `water-look` design: real deep ocean returns about 2-3 % of the blue
light; from the air its colour is mostly the reflected sky.

- Body: `deep_lit = body * light` (`water_surface.wesl:311`), light = the key
  light plus the ambient the ground takes; murk `deep * light * exp(-a d)`
  (`water_common.wesl:84-86`).
- Deep (0.004, 0.018, 0.040); shallow (0.020, 0.090, 0.075) easing to deep by
  14 m of water under the pixel; absorption (0.32, 0.035, 0.012) per m (Pope
  and Fry, blue-green halved after Lee 2015).
- Fresnel `(0.02 + 0.98 (1 - n.v)^5) * mix(0.7, 1, |r.y|)`: its horizon
  strength 0.7 (0.22 left the far sea black).
- Reflection: the camera's sky cube, scaled by the hour.

## 2. What this engine has instead

No sky cube; the terrain's L2 sky (`sky_sh_pair`, nine coefficients clear and
under a full deck, in the local frame of `sky_frame`) with `sh_radiance`, the
sky's radiance toward a direction, which the wet film already mirrors. The
terrain lights a face `albedo * (sh_irradiance(n) + sun_air(e) * n.l * sun_dim)`.

## 3. The change

Computed on the CPU each frame, at the camera (`prepare_water_views`):

- the nine coefficients mixed by the cover over the camera;
- `water_light` = `sh_irradiance(up)` + `sun_air(e) * max(e, 0) * (1 -
  overcast_sun_dim * cover)`: the light on open water, the terrain's for a cap
  under open sky;
- the haze's weather: cover, rain, `cloud_fog_add`, `rain_fog_mult`,
  `overcast_sky_blue_cut`, `overcast_sky_dim`.

In `water.wgsl`:

- `murk(d)` = `deep * water_light * exp(-a d)`.
- The sheet: body = `mix(shallow, deep, smoothstep(0, 14 m, depth under the
  pixel)) * water_light * sky_light`; `transmitted = mix(body, scene,
  exp(-a path))`, the scene already lit; the reflection `sh_radiance` of the
  reflected ray where the screen trace finds nothing; foam and glint lit by
  `water_light` and the sun as it reaches the water.
- `distance_fog` = the terrain's `finish` haze: density `0.00036 (1 + cover
  cloud_fog_add) mix(1, rain_fog_mult, rain)`, toward `ground_sky(e, cover)`.

In `planet_surface.wgsl`, under the new model: the seabed's tint takes the
deep colour times the face's own light.

## 4. Data

`water.ron`: `deep_color` (0.004, 0.018, 0.040), new `shallow_color` (0.020,
0.090, 0.075) and `shallow_depth_m` 14, `absorption_per_m` (0.32, 0.035,
0.012), `sky_horizon_strength` 0.7. `sky_horizon_color`, `sky_zenith_color`,
`night_sky_color` and `night_floor` go: the sky's own light replaces them.

## 5. As built

- The sun on the water is dimmed by the cover over the camera, `1 - 0.85 *
  cover` (the rain's `rain_sun`), not the terrain's per-point cloud shadow
  (`cloud_sun`), which needs the weather map in the water pass.
- The glint takes `rain_sun` (the sun after the air and the cloud) in place
  of the old daylight floor.
- Measured at night (README): the sea 0.0091 to 0.0055 in linear luminance,
  under the beach's 0.0137; still about twice the night dome over the
  horizon (task 3.3).
