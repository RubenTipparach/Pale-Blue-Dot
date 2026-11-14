# Design: sun-in-space

## 1. Measured: the sun has a pixel only where the shell does

Fixed-step stills, the M4 Max Mac (Metal), 2880 x 1800, the fast build at
`d43d018`, `--view column --at 0 0`, a new world each. The sun's direction for
a clock hour comes from `pbd_core::daylight::Clock::sun` (day 0, the northern
solstice; the yaw and pitch that face it are in
`docs/screenshots/sun-in-space/README.md`).

| Shot | Eye | Sun under the horizontal | Shell's rim / limb | Before |
| --- | --- | ---: | --- | --- |
| `orbit-through-air` | 6 km | 60.6 deg | 57.8 / 63.6 | whole, behind the air |
| `orbit-on-rim` | 6 km | 57.8 deg | 57.8 / 63.6 | cut flat along the rim |
| `orbit-above-rim` | 6 km | 55.7 deg | 57.8 / 63.6 | gone; the lower glow only |
| `crossing-inside` | 900 m | 5.3 deg | (inside) | whole |
| `crossing-outside` | 1050 m | 5.3 deg | 6.7 / 34.1 | gone |

The last pair is the whole story: 150 m of climb, out through the shell's top
at 960 m, and the sun vanishes, because from outside the dome's fragments cover
only the directions that meet the shell.

## 2. A sun of its own, drawn from outside

The dome keeps drawing the sun from inside the shell, unchanged, so the ground's
sun is the same pixels as before. From outside, a second draw takes over:

- **Geometry.** One quad, four corners in -1..1, built in the vertex shader:
  `ray = sun + (right * x + up * y) * tan(SUN_REACH)` with `right`, `up` an
  orthonormal pair across the sun direction, and the clip position of
  `camera + ray * SUN_DISTANCE_M`. `SUN_REACH` = 0.12 rad (6.9 degrees): where
  the glow `0.55 * cos^1800` has fallen to 2e-6 of the sun's radiance, so the
  quad's edge is never seen. `SUN_DISTANCE_M` = 150 km: in front of the stars
  (180 km), behind the moon's orbit (26 km) and the planet, so the depth test
  hides it behind both as it hides a star. From inside the shell the vertex
  shader collapses the quad to a point: no fragments.
- **Fragment.** The direction is `normalize(ray)` (interpolated in the
  camera's frame, not the 150 km world position, so f32 keeps the disc's edge
  sharp). The light is the dome's own sun term, one function both call:
  `SUN_COLOUR * sun.w * (disc * limb + glow) * sun_visibility(camera, sun)`.
- **Through the air.** Along the ray, the optical depth through the shell
  (sixteen midpoint steps of `air_density`, the dome's integration) gives the
  dome's own extinction, `T = exp(-(beta_r + beta_m) * depth)`, per channel:
  blue goes first, so a sun seen through the limb's air is dimmer and redder.
  A ray that meets the sea sphere is behind the planet and gives nothing.
  `T` comes in over the first 200 m past the shell's top
  (`mix(1, T, handover)`, the `high-haze` hand-over), so in the frame the eye
  leaves the shell the sun is the inside sun's, unattenuated, and nothing
  steps. (From inside, the dome's sun is not attenuated by the air; that is
  the look the owner has, kept.)
- **Blend.** Additive (`AlphaMode::Add`, alpha 0): light added over whatever
  the dome left, as the sun's light is. It is sorted to draw just after the
  dome (its entity sits at the planet's centre with the dome, and a
  `depth_bias` of 1 m puts it behind in the back-to-front order), so the
  dome's column does not cover it a second time.
- **The dome, outside.** It no longer adds the sun from outside, or the glow
  along the rim would be counted twice (the dome's and this one's).

### Why not the alternatives

- *A full-screen fallback in the dome when outside*: the dome's mesh has no
  fragments off the shell; a full-screen pass is a new pass in the graph for
  one disc. The quad is twelve degrees of screen, depth tested for free.
- *Drawing the sun under the dome and letting its opacity dim it*: the dome's
  premultiplied blend dims by one scalar, so no reddening; and from inside its
  daylight veil (0.985) would hide the sun.

## 3. One implementation

The new draw is `sky_atmosphere.wgsl` compiled with `SUN_DISC` defined, by a
`SunDiscMaterial` whose uniform is the dome's `SkyParameters`: the air's
density, the extinction coefficients, the disc's constants and
`sun_visibility` exist once. Its parameters are copied from the dome's
material each frame by one system, after the dome's own are written
(`follow_weather`, `position_atmosphere`), so the two cannot disagree about
the sun, the weather or the planet's centre.

## 4. What stays

- From the ground and in cloud: the dome draws the sun as before; the new quad
  is collapsed. Before and after stills must match.
- Clouds: the clouds pass (`water.wgsl`) composes after the main pass, over
  both draws, so a cloud hides the sun from anywhere.
- Exposure and bloom: the same radiance (`SUN_COLOUR * 3.2` at the disc's
  centre) as the dome's sun, above white, so it blooms as before.
- The moon: a lit sphere mesh (`scene.rs`, `MoonOrbit`), drawn wherever it is;
  not affected, nothing to change. It occludes the sun by depth.
- The loading screen: the quad is always visible and never culled
  (`NoFrustumCulling`), so its pipeline is built on the first frames behind
  the loading screen, like the dome's. A capture's log is checked for a
  `LATE_PIPELINE` naming it.
