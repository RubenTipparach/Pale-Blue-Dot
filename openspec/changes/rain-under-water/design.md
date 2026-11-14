# Design: the rain volume is in the air, not in the sea

## 1. Where the rain pass sits

`WaterCompositeNode::run` draws, in order: the water compose (underwater fog,
distortion) and the sheet; the cloud march, puffs and resolve; the clouds
composite; the rain (`rain_half` into the small target and `rain_compose`
over the frame, or `rain` at full size); the lens. The rain reads the
finished frame as `scene_color` and writes `scene*a + light`.

Moving the rain before the sheet would let the sheet refract it, but the
sheet would then also refract the rain into the window's compressed view of
the sky, and the pass order is shared with the clouds and the cloud history.
The rain is a soft volume with no sharp feature to refract, so this change
keeps the order and makes the rain's own ray what an eye under water sees.

## 2. The ray from under the sea (`rain_seen`)

Called by `rain_half` and `rain`, in body-local metres, for the eye `e` and
the pixel's direction `d`. Dry or straddling (`fx.y <= 0.75`): today's
march, from the eye to the scene's depth, the sea sphere's near hit, the
cloud base or the range, weight one.

Under (`fx.y > 0.75`):

1. The exit: `s = cloud_sphere_hit(e, d, R_sea).y`, the far root, which from
   inside the sphere is ahead. The surface normal there is `n = (e + d s) /
   |e + d s|`, and `cos_i = d . n`. No exit (`s <= 0`) or a ray that does not
   rise (`cos_i <= 0`): no rain.
2. Something in the water first: the scene's depth along the pixel is nearer
   than `s` (a hull, the seabed, a wall): no rain.
3. Snell: `sin^2 t = (1.333)^2 (1 - cos_i^2)`. At or past one, the ray is
   totally reflected (outside the window, 48.6 degrees off the normal): no
   rain.
4. The air ray: `refract(d, -n, 1.333)`, from `e + d s` lifted 5 cm along
   `n`, to the cloud base, the range, or what the scene shows past the
   surface (its depth less `s`).
5. The weight: the unpolarised Fresnel transmission `T = 1 - (r_s^2 +
   r_p^2)/2` at `cos_i`, `cos_t`, times `exp(-absorption s)` per channel,
   the water's own absorption (`view.absorption`) over the path to the
   surface.

The gathered light `L` and transmittance `a` become `L * w` and `1 - mean(w)
(1 - a)`: the rain darkens and lights only by the share of it the eye
receives through the surface. At 5.4 m under, looking straight up, `w` is
about (0.17, 0.81, 0.92): what the rain does to the frame is a fifth in red
and most of it in blue, inside the window, and none outside it.

## 3. Cost

Under water the march now starts at the surface instead of at the eye, the
same sixteen steps or the same shells; outside the window it is skipped.
Above water nothing changes. No frame-time sitting (none may run on this Mac
while other agents capture); the change cannot add work to a dry frame.

## 4. Decisions

- Recommendation taken (ask only with screenshots): the rain is seen faintly
  through the window rather than not at all under water; the physics says it
  is there, and the before/after shots show its weight.
- The full-size `rain` and the small `rain_half` share `rain_seen`, so the
  two renderers cannot disagree about the water.
