# Rain artifacts (owner, 2026-11-16)

The owner's pictures (`owner-*`) and the captures that reproduce, measure
and (after the fix) show each artifact. Fixed-step `--capture` shots on the
Mac (M4 Max, Metal, `--profile fast`, 2880x1800). Looks only: frame time was
not measured. The write-up is in `flora-v2` design 13 (the trees),
`weather-rain` design "Rain artifacts" (the streaks, the tufts, the lens) and
`water-flow` design "Rain rings that alias" (the river).

## Where each was taken

| Spot | Flags |
| --- | --- |
| The valley (terraced, river, mixed forest) | `--walk --at -34.6830 101.4869 --yaw 14 --pitch -6 --time 11 --day 27`, `--rain 1` for rain |
| The valley's tufts | the valley, `--yaw 75 --pitch -25` |
| The jungle | `--walk --at 28.706 3.031 --yaw 0 --pitch 3 --time 11 --rain 1` |
| Looking up into a tree | the jungle, `--yaw 200 --pitch 60` |
| The river from the air | `--fly --at 24.2400 3.8543 --yaw 149 --pitch -35 --height 40 --time 12 --rain 1` (it rains here unforced too) |
| The valley's river from 25 m | `--fly --at -34.6830 101.4869 --yaw -15 --pitch -55 --height 25 --time 11 --day 27`, `--rain 1` for rain |
| The far trees, night and day | `--walk --at 28.2398 -19.596 --yaw 330 --pitch -5`, `--time 2` and `--time 12` (it rains here at noon unforced) |

## Before

| File | What it shows |
| --- | --- |
| `before-valley-dry.jpg`, `before-valley-rain.jpg` | One frame, the forcing the only change. In rain every mid and far tree is a pale grey-blue blotch (the wet ground's puddle mirror on the impostor cards), and the near shower's streaks are bright blue lines, (110, 128, 180) over a wall of (50, 60, 60). |
| `before-jungle-rain.jpg`, `before-jungle-rain-clouds-off.jpg` | Pale checker patches behind the near kapoks: the trees' cross-fade dither, standing in its distance ring. Still there with the clouds pass off (`PBD_PASS_OFF=clouds`). |
| `before-tufts-rain.jpg` | Tufts and blades under the wet ground's sheen. |
| `before-tree-lookup-rain.jpg` | Looking up into a kapok in rain: leaf cards with rings and sheen, dithered cards past it. |
| `before-river-air-rain.jpg` | The river from 40 m up, covered in per-pixel speckle. |
| `before-river-25m-dry.jpg`, `before-river-25m-rain.jpg` | The valley's river, dry and in rain: the rain turns its waves into speckle. |
| `measure-river-25m-rain-composite-off.jpg`, `measure-river-25m-rain-rings-off.jpg` | Instruments, not the game: the rain, lens and shower passes off (`PBD_PASS_OFF=rain,lens,shower`), then the same with the water's rain rings zeroed by a temporary shader edit (since reverted). Speckle measure (`hf.py`, mean absolute Laplacian of the luma over x 1250-1450, y 1050-1210): dry 19.0, rain 36.5, rings off 27.1, rings and the bed's wet block off 26.0. |
| `before-far-trees-night.jpg`, `before-far-trees-day.jpg` | Near kapoks and the far band in one frame. At night the far band's mean sRGB is (0.4, 12.9, 14.1) against the near crowns' (0.5, 9.5, 12.4): a third brighter and flat. By day (raining there) the far trees are pale ghosts. |

More spots, after the first set:

| Spot | Flags |
| --- | --- |
| A beach inlet in rain (the owner's "distant ripples") | `--walk --at 20.0 26.0 --yaw 0 --pitch -10 --time 11 --rain 1` |
| Toward and away from a low sun, the ground soaked under a clear sky | the valley, `--yaw -7` (toward) or `--yaw 173`, `--pitch 6 --time 12 --day 27` (sun 16 degrees up); MEASUREMENT: both builds' `planet_surface.wgsl` temporarily took `wet_amt` from 1 instead of the wetness (put back after), so wet ground and sun come together as in the owner's shot |
| The far trees in rain by day | `--walk --at 28.2398 -19.596 --yaw -16 --pitch 5 --time 17` (it rains there) |
| A treeline at night in rain | the valley, `--yaw 14 --pitch 2 --time 23 --day 27 --rain 1` |

The befores of these were taken with `claude/env-v2` at `f7c63a8` built
beside this branch (`--profile fast`), at the same spot and frame count.

## After (branch `claude/rain-artifacts`)

| File | What it shows |
| --- | --- |
| `after-valley-dry.jpg`, `after-valley-rain.jpg`, `crop3x-valley-trees.jpg` | The mid and far trees keep their greens in rain, a little darker than dry (`rain_plant_darken` 0.88). |
| `crop3x-valley-streaks.jpg` | A near streak was (110, 128, 180) over a wall of (45, 53, 56); now (67, 81, 104) over the same wall: rain over the scene, not a glowing line. Its length is unchanged (1.5 m, Tenebris's). |
| `after-jungle-rain.jpg`, `crop3x-jungle-ghosts.jpg` | The pale checker behind the near kapoks is gone: the ring dither no longer stands round a still camera, and no card mirrors the sky. |
| `after-tufts-rain.jpg`, `crop3x-tufts.jpg` | Tufts and blades: no rings or sheen; the cap under them keeps its own. |
| `after-tree-lookup-rain.jpg`, `crop3x-leaves.jpg` | Leaf cards and bark with no rings, sheet or sheen; the drops on the lens stay. |
| `after-river-air-rain.jpg`, `crop3x-river-air.jpg`, `after-river-25m-rain.jpg`, `crop3x-river-25m.jpg` | The river's speckle is gone. Speckle measure over the valley river's patch: dry 19.0, rain before 30.2, rain after 20.8. |
| `before-beach-rain.jpg`, `after-beach-rain.jpg`, `crop3x-beach-far.jpg`, `crop3x-beach-near.jpg` | Far water: speckle to calm waves (measure 42.3 to 24.8 over x 1500-2300, y 700-1000); near water keeps its rings (27.7 and 27.3). |
| `before/after-far-trees-night.jpg`, `crop3x-far-trees-night.jpg` | Leaf pixels' mean sRGB (`leafmean.py`): far band 15.6 to 13.8 green against the near crowns' 16.5: the far band now sits a little under the near trees, lit by a standing normal. |
| `before/after-far-trees-rain-day2.jpg` | The pale ghost forest behind the near kapoks is green. |
| `before/after-soaked-toward-sun.jpg`, `crop3x-soaked-toward-sun.jpg`, `before/after-soaked-away-from-sun.jpg` | Soaked ground and a low sun: before, every mid and far tree toward the sun is a pale sky-coloured sheet (the wet film's mirror and glint on cards lit as floors); after, they are trees, darker toward the sun (backlit by their own normals), none bleached either way. |
| `before/after-treeline-night-rain.jpg` | Night in rain: no pale dither patches round the crowns; the streaks are dim. |

### Sequences

Made with `PBD_WEATHER_SCRIPT` (frames: `cover`, `open`, `dry`, `rain`) and
`PBD_CAPTURE_RUN=<dir>,100,<count>,30` (every half second), the valley in
rain; crops x 1000-1700, y 250-700 so the lens drops show. Times are from
the event. The traces (`trace-*.txt`, a line every quarter second) give the
numbers.

| File | What it shows |
| --- | --- |
| `seq-lens-into-cover.jpg` | Walking under cover (the instrument's `cover`): the drops stop where they are and fade over about 2.5 s (`lens_wet` 0.80 at the event, 0.27 at +2.75 s, the drops' clock stopped at 4.98). The near shower thins over 0.4 s (`shower_open` 0.96, 0.51, 0.28, 0.15 a quarter second apart), the shafts out in the open stay. |
| `seq-lens-out-of-cover.jpg` | Back out: the drops build up over about 3 s (`lens_wet` 0.02, 0.10, 0.17 ... 0.50 at +2 s, 0.96 at +10 s) and run again. |
| `seq-rain-stops.jpg`, `trace-cover-open-stop.txt` | The rain stopping (`dry`): the rain under the player dies away on the field's own 20 s; the ground's wetness and the shower's streak count follow it with no step (`streaks` 3179, 3158, 3138 ...). |
| `seq-rain-starts.jpg`, `trace-rain-starts.txt` | The rain starting (`rain`): rain builds on the field's 5 s, the ground's wetness after it on 1.6 s, the lens after it on 3 s (`lens_wet` 0.18 at +3 s, 0.53 at +7 s, 0.89 at +15 s), the streaks from 1482 (the shafts) up with it. Nothing jumps. |
| `seq-tree-fade-still-camera.jpg` | `PBD_TREE_ARRIVE=30@3` at the jungle: the trees decided as if the camera had just come 30 m and stopped. At 0.1 s and 0.3 s the trees it carried across a threshold are mid-dither; at 2 s none is. |

### Not shown or not done

- No walk sequence for pops: the fade's logic (every change of a tree's
  representation fades over its time, `the_trees_fade_by_time_and_a_still_camera_is_whole`)
  stands in for it; a capture walk was not taken.
- The terrain's own band cross-fade (`distance-lod-fade`'s rings, which the
  trees on a coarser band also ride as they grow) is still by distance; see
  `flora-v2` 13.3, "What stays".
- The splashes (merged since) fade out by `rain_detail_reach_m` (20 m) in
  their own shader and are listed only within it plus 2 m: no speckle from
  them past it.
- A normal launch (no flags) reached READY with 108 pipelines and no
  `LATE_PIPELINE`. Captures print `LATE_PIPELINE` lines as captures always
  have.
