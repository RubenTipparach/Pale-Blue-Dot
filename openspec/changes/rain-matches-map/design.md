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

# Part 2: half the rain, none on the desert

The owner, 2026-11-16 (through the coordinator): "50% less rain on the
planet, and avoid rain cells over desert".

## Measured before

`pbd-core`'s `examples/rain_budget.rs` (a measurement instrument): the
shipped settled climate (generator 10, level 5) run on for 30 world days
after a day to settle in, every cell sampled each 60 s, area-weighted.

| | Before | After | After / before |
| --- | ---: | ---: | ---: |
| Share of the planet raining (rate at or over `raining_rate`) | 19.79% | 9.58% | **0.48** |
| Mean rain rate, mm/h | 11.53 | 5.44 | **0.47** |
| Mean rain seen (what a player stands in), 0..1 | 0.231 | 0.119 | **0.51** |
| Mean cloud cover | 0.459 | 0.349 | 0.76 |
| Desert: share raining | **29.7%** | **0%** | |
| Desert: mean rain rate, mm/h | 20.05 | 0 | |
| Mean surface temperature, C | 15.10 | 15.06 | |

The desert rained MORE than the planet's average before: its wetness (0.1)
only slows what evaporates from it, and the air carried in from elsewhere
rained on it as on anything.

## Decisions

5. **Half the rain is half the water cycle.** Over time everything that
   rains evaporated first, so how MUCH it rains is set by `evaporation`:
   3.0e-4 -> 0.75e-4 (the deficit it works on grows as the air dries, so a
   quarter of the rate gives about half the rain). How OFTEN is set by the
   rain threshold: `rain_threshold_kg` 4 -> 8 kg/m^2, so a cloud holds twice
   the water before it rains. Trials over 5 days (from the settled state,
   with the virga below) chose the pair:

   | evaporation, threshold | raining | rate | rain seen | cover |
   | --- | ---: | ---: | ---: | ---: |
   | 3.0e-4, 4 (as shipped) | 19.5% | 11.17 | 0.233 | 0.488 |
   | 1.5e-4, 4 | 16.8% | 8.46 | 0.198 | 0.406 |
   | 1.0e-4, 4 | 15.4% | 7.17 | 0.179 | 0.365 |
   | 0.5e-4, 4 | 13.2% | 5.13 | 0.147 | 0.305 |
   | 3.0e-4, 6 | 16.3% | 11.22 | 0.206 | 0.522 |
   | 1.0e-4, 8 | 10.5% | 6.67 | 0.133 | 0.400 |
   | 0.6e-4, 7 | 10.0% | 5.36 | 0.125 | 0.353 |
   | **0.75e-4, 8** | **9.8%** | **5.95** | **0.125** | **0.379** |

   No scale on the value shown: the map, the player's rain, the wet ground
   and the puddles all read the simulation, so they all halve with it.
   Cloud cover falls by a quarter with the water: less vapour, less cloud.
6. **No rain on the desert: virga.** Rain falling out of a cloud over arid
   ground evaporates in the dry air under it before it lands. What falls
   out of a cell's cloud (by its threshold or a lightning strike) reaches
   the ground in proportion to a smoothstep of the ground's wetness between
   `virga_dry_wetness` 0.15 (none lands) and `virga_wet_wetness` 0.3 (all
   lands); the rest goes back to vapour and takes its latent heat from the
   air, so the water and the heat are both kept. The desert is the only
   biome under 0.3 (0.1); mountains and tundra are 0.3 and rain as before.
   Cloud still passes over the desert (its mean cover 0.39); it does not
   rain there. A rule of the ground, not a mask of the desert: a future
   biome as dry would get the same.
   - The rain seen is read at a place by interpolating the three cells
     round it, so within a cell's width (181 m) of a desert's edge a little
     of the neighbour's rain is still seen. At a desert cell itself it is
     nothing (the test).
7. **Saves.** A saved world keeps its weather (`weather.bin`) byte for byte
   and steps it on under the new rules from its next step; the saved state
   carries no settings, so nothing is reset. The shipped settled climates are
   remade on the new settings (`examples/settle_climate.rs`, every generator
   4-10 at levels 3 and 5), as every change of the weather's rules has done:
   their `.ron` records the settings, and the game refuses a state settled
   under other physics. They only start new worlds (and a world with no
   weather of its own); worlds already played are untouched.

# Part 3: snow in freezing air

The owner, 2026-11-16 (through the coordinator): "no rain in the cold
regions, snow only please".

## Measured

What falls was snow where the GROUND was below 0 C (`Sample::snow`). The
ground warms past zero under a low sun while the air over it stays freezing:
over two days from the settled climate (new settings), **0.87% of everything that fell was
rain in air at or below 0 C** (`rain_budget`'s "rain in air at or below
0 C"). And several of rain's effects read the rain, not the water: the
ground's puddle rings and wet sheen (`planet.rs`, `params.weather.y`) and
the rain fog (`planet_water.rs`, `haze_weather.y`) read `Weather.rain`, so
in snow they still rang the puddles and fogged the air.

## Decisions

8. **Snow is the air's call.** `Sample::air_c`, the air at the ground's own
   height, decides: at or below 0 C what falls is snow. The one rule every
   reader takes is still `shown_precipitation`, so the map's overlay (snow
   in its violet, rain in blue), the player's rain, the shafts and the rain
   volume all switch together. After it: 0.00% rain in freezing air.
   There is no sleet band: the model has one temperature per place and no
   melting layer, so the switch is clean.
9. **Snow is not water.** Everything that is rain's water reads
   `Weather::liquid`: the lens drops, the splashes, the puddle rings and the
   wet sheen, the rain fog, the sea's rings and the wet ground. Where it
   turns to snow, `liquid` eases out over `snow_switch_s` (3 s, e-fold, in
   `weather.ron`) and is nothing once it is snow; the shower draws flakes
   from the moment it is snow.
10. **Cold regions that are not freezing.** The tundra (the generator's
    biome, the white ground) is not the same as freezing air: on day 0
    (one pole's summer) half of what falls on tundra cells is in air above
    0 C (their air where anything falls: median -1.3 C, 75th percentile
    8.4 C). With the air's rule that falls as rain. If the owner means "no
    rain on the white ground, ever", that is a different rule (the ground,
    or its season) and is left as a question with screenshots.
11. **At the line itself** the map's rain and snow are separate channels of
    118 m texels, filtered, so within a texel of the freezing line the map
    shows both and may lean to rain where the player has snow
    (`docs/screenshots/rain-matches-map/snow-line-*`). Everywhere else the
    two agree (the agreement test runs at texel centres).

## Side effects, measured

- The start of a new world (the perf suite's `walk`) opens under a thinner
  sky: cover 0.56 where it was 0.69 (`tests/new_world_start.rs`, updated).
  Its frame time was not measured here.
- The remade settled climates' new worlds settle at 15.1 C (the target, 15).
