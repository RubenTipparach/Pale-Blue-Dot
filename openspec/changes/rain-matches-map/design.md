# Design: the rain on the map is the rain you stand in

## What each side reads

| | Reads | Where |
| --- | --- | --- |
| The rain the player stands in (`Weather.rain`: the streaks, the lens drops, the wet ground) | `Atmosphere::rain_seen` at the camera's direction, through `weather::cloud_cell`: the cover where the rain rate is over `raining_rate`, built in over `rain_rise_s` (5 s) and let go over `rain_fall_s` (20 s) (`smooth-weather` decision 1) | `weather::sample_field` |
| The map's "Clouds and rain" layer | `WeatherMaps.cloud[..][2]`: the simulation's rain RATE in mm/h, averaged over a 90 m disc, drawn at `0.55 * mmh / 8` | `map_screen::cloud_bytes`, `map_live.wgsl` |
| The map's (and the globe's) RAIN overlay | `Sample::rain_rate` in mm/h on a -10..10 ramp, fully drawn from 1.5 mm/h | `pbd_core::overlay::Overlay::texel`, `overlay::overlay_rgba` |

Coordinates, frames and the projection are not the cause: the map's cubes are
built with `cube_direction` from body-local directions, the player's sample
is the camera's body-local direction, and `the_map_shader_projects_as_geo_does`
already holds the map's projection to `geo`. Both sides read the same
published atmosphere (the map the newest, the player the shown pair, at most
`MAX_BLEND_S` = 4 s apart). Nothing reads a forced `--rain` or `wetness` for
streaks: the streaks and lens follow `Weather.rain`.

**The cause is that they read two different fields.** The player stands in
the rain SEEN, which is smoothed; the map draws the rain RATE, which is not.
The simulation's rain is convective and comes in bursts: a place rains for a
median 25 s and is dry for a median 23 s between bursts (`smooth-weather`'s
measurement). The rain seen bridges those gaps on purpose, so the streaks do
not switch on and off every half minute. The rate does not: in every gap it is
zero, and the map shows no rain there. A paused map at a gap shows the
player standing in a hole in the blue, with the rain's blobs nearby, which is
the owner's screenshot.

## Measured

`atmosphere::tests::map_rain_against_local` (an `#[ignore]`d instrument), on
the shipped settled climate, 3,000 places sampled every 10 s through ten
minutes, 44,859 samples where the player stands in rain of 0.3 or more:

| Where the player stands in rain of 0.3 or more | Share |
| --- | ---: |
| The rain rate is under the raining threshold (0.72 mm/h): between bursts, the rain seen still letting go | **18.4%** |
| Rain rate 0.72-1.5 mm/h | 2.3% |
| Rain rate 1.5-3 mm/h | 3.9% |
| Rain rate 3-8 mm/h | 10.1% |
| Rain rate over 8 mm/h | 65.3% |
| The clouds layer's blue under 0.05 (none to the eye) | **14.3%** |
| The clouds layer's blue under 0.1 | 17.0% |
| The RAIN overlay's blue under 0.05 | **18.8%** |
| The RAIN overlay's blue under 0.1 | 19.9% |

(The rate's median there is 19 mm/h, its 90th percentile 99 mm/h.) So about
**one moment in six that a player stands in rain, the map shows none where
they stand**. The smoothed layer hides a little of it (14% against 18%: its
90 m disc reaches a neighbouring burst); the overlay hides none of it.

The owner's own place (frame latitude 26.562, longitude -14.111) rains in the
instrument's run from the 130th second, so it is a place that rains, in a
storm that moves; the exact moment of the screenshot (the save's own weather)
cannot be rebuilt here.

## Decisions

1. **One field: the rain seen.** The map draws what the player stands in.
   One function in `pbd_core::weather`, `shown_precipitation(&Sample)`,
   says what is falling as a person sees it: the rain seen, negative where it
   snows, nothing under `RAIN_SEEN_MIN`. `cloud_cell` (hence `Weather.rain`,
   the rain shafts and the rain volume's `precipitation_map`), the map's clouds
   layer and the RAIN overlay all read it. The rate stays the physics' own
   number, read by the simulation and the GPU's cloud maps, and no longer by
   anything that shows rain on a map.
   - Not the other way round (the player following the rate): the smoothing
     is what `smooth-weather` built so rain does not switch on and off every
     half minute. The map loses nothing true by showing it; the gaps it
     showed were never visible on the ground.
2. **The clouds layer reads a texel of its own.** The GPU's cloud map keeps
   the rate in its third channel (the shaders read it), so `WeatherMaps`
   gains `rain_seen`, one signed value per texel, made off the main thread
   with the others from the same state at the texel's own direction, unsmoothed
   as the player's is. `cloud_bytes` draws it straight: 0..1 is already the
   layer's scale, where 8 mm/h used to be.
3. **The RAIN overlay is in percent of a full rain.** Its scalar is the rain
   seen times 100 (snow negative), its range -100..100, its unit `%`, and it
   is fully drawn from 15%, the fade every fading overlay already uses. The
   legend reads "100 snow, 0, 100". The rate in mm/h was the only reading of
   the physics' rate on screen; it goes, because a rate that says "dry" under
   falling rain is the bug. Recommendation taken (ask only with screenshots):
   the before-and-after captures go with the change.
4. **The test** (`weather` in `pbd-app`): at sampled places and times of the
   shipped climate, stepping it, the map's clouds-layer value at a place, the
   RAIN overlay's value there and `Weather.rain` as `sample_field` computes it
   agree within a byte. The map is built from the newest state and the player
   reads the shown pair: that is a blend of at most 4 s, inside the 5 s rise,
   and is left as it is.

## Not changed

- `rain_rise_s`, `rain_fall_s`, `raining_rate`: untouched. The drops on the
  lens and the streaks still let go over seconds after the rain seen falls
  (`claude/rain-artifacts` is easing those in and out).
- Saves: `rain_seen` is derived and never saved; nothing saved changes.
