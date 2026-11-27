# Design: the rain volume is in the air, not in the sea

## 1. Where the rain pass sits

`WaterCompositeNode::run` draws, in order: the water compose (underwater fog,
distortion) and the sheet; the cloud march, puffs and resolve; the clouds
composite; the rain (`rain_half` into the small target and `rain_compose`
over the frame, or `rain` at full size); the lens. The rain reads the
finished frame as `scene_color` and writes `scene*a + light`. The clouds
composite and the lens already return early under water (`fx.y > 0.75`);
the rain had no such test, and its ray starts at the eye.

## 2. What an eye under water sees of the air

Every ray from an eye under the sea starts in the water. What it sees of
the air beyond the surface (Snell's window, the murk over the window, the
total internal reflection outside it) is the sheet's, already in the frame
the rain pass reads. The owner, with the volume off: "underwater is fine".
So under water the rain volume draws nothing:

- `water.wgsl`: `rain_under_water()` (`fx.y > 0.75`); `rain_far` returns no
  rain and `rain` returns the scene untouched when it holds.
- `planet_water.rs`: `rain_needed` is false when the camera's state is
  under, so neither rain pass runs there. The shader's test stands behind
  the CPU's, so a view whose state the CPU gets wrong cannot fog the water.

Above the sea and straddling the surface nothing changes: the rain already
stops at the scene's depth and at the mean sea sphere's near hit.

## 3. Tried and dropped: the rain seen through Snell's window

The first build of this change started the march where the ray leaves the
water, on the refracted direction, inside the window only, weighed by the
Fresnel transmission at the exit and the water's absorption on the way up
(at 5.4 m, looking straight up, about 0.17 / 0.81 / 0.92). The capture
(`docs/screenshots/rain-under-water/`, not kept as an after shot) showed the
window as a flat, bright teal disc with a hard rim, much brighter than the
same window with the volume off: the sheet darkens what it shows through the
surface by its own murk, and a separate rain term weighed by physics alone
does not agree with it. Doing it properly means the sheet refracting the
rain (the rain drawn before the sheet), which moves the pass order the
clouds and their history share. The rain in the window is a soft haze with
nothing sharp in it; not drawing it is the smaller error.

## 4. Cost

Under water the rain passes are skipped: less work, never more. Above water
nothing changes. No frame-time sitting (none may run on this Mac while
other agents capture).

## 5. Decisions

- Recommendation taken (ask only with screenshots): no rain volume under
  water, rather than the rain through the window (section 3); the shots are
  in the README for the owner to judge.

## 6. The splashes under a lake or river (owner, 2026-11-26)

The owner, from under water in rain with a rod in hand: "rain fx underwater
need fixin" (a scatter of white specks over the bed and bright streaks
across the far water). The rain volume is off under water (section 2), but
the splashes (`planet_splash.rs`, `weather-rain` 6.4) have a gate of their
own, and it knows only the sea: `surface_height(up) < 0 && radius <
PLANET_RADIUS`. Under a lake, a river or a pool, where the ground's height
is above zero, the splashes still draw: every crown on the bed's column tops
and on the water's top, seen from below. The near shower closed the same
hole in `ocean-environments` B17 by reading the column's water at the eye
(`EyeWaterState`); the splashes never took it.

The fix is one rule for both: the splashes and the shower ask
`audio::hooks::eye_under_water`, which already reads `EyeWaterState` (a
river, pool or lake) and falls back to the sea's sheet, and draw nothing
when it says the eye is under. The shower's own copy of that test goes, so
the two cannot drift again.

Still to confirm in the game once the owner's session is closed: the owner's
place in their shot (sea or inland water), and whether the far streaks are
the splashes or the sheet's rain rings at Snell's window (task 3.3).
