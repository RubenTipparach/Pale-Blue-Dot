# night-lights-from-space: before and after

The `night-lights-from-space` change (`openspec/changes/night-lights-from-space`):
settlements drawn as warm clusters of light on the night side from afar, the
player's lights past a threshold likewise, and the glowing flowers a quarter as
many at a quarter of the brightness.

## How they were taken

- 2026-11-14, on the M4 Max Mac (Metal, the GPU), 2880 x 1800, fixed-step
  stills (`--capture <png> --frames 200`), the fast build
  (`cargo build --profile fast -p pbd-app`).
- "Before" is `2c17e84` (`claude/perf-tuning`), built from a `git archive` of
  that commit so it read that commit's shaders and configs. "After" is this
  branch.
- Every shot a new world (`--world nl-<before|after>-<shot>-<n>`). `--time` is
  the clock at longitude 0.
- The sheets are laid out by `tools/contact_sheet.py`, before on the left.
- Frame cost was not measured (no perf sitting in this change).

| Shot | Arguments | Notes |
| --- | --- | --- |
| `orbit-9km`, `orbit-20km` | `--view column --at 29.70 1.36 --height 9000 / 20000 --pitch -70 --time 23 --rain 0 --weather-at 3600` | The home continent on the night side (sun 23 degrees down at the spot) |
| `town-dusk-3km` | `--view column --at 29.47 8.48 --height 3000 --pitch -89 --time 20.6 --rain 0 --weather-at 14400` | Straight down on Linenleigh (walled) at dusk, the sun about 4 degrees down, clear (`cover 0.00`) |
| `town-night-3km` | the same, `--time 23` | Night |
| `town-day-3km` | the same, `--time 11` | Day: no lights (the town is under fair-weather cumulus) |
| `town-night-900m` | the same, `--height 900 --pitch -60 --time 23` | Inside the near fade: the standing town's own lamps (bottom) and no far points over them; a town further off shows its far light |
| `deck-9km`, `deck-3km` | `--view column --at 19.70 59.20 --height 9000 / 3000 --pitch -70 / -89 --time 19 --weather-at 0` | Ashenstead (walled) under a storm deck at night (`cover 1.00`) |
| `flowers-night`, `flowers-night-b` | `--walk --at 30.60 1.37` / `--walk --at 28.60 0.40 --yaw 40`, `--pitch -14 --time 23 --rain 0 --weather-at 3600` | Meadows outside Holbrook at night |

## The sheets

| File | What it shows |
| --- | --- |
| `orbit.jpg` | 9 and 20 km: before, the night side is dark everywhere; after, each settlement is a warm spark, the walled towns biggest. |
| `town.jpg` | Dusk (the lights partly up), night (Linenleigh in the middle, Holbrook above it, two more), day (nothing), and 900 m (the far light faded out over the standing town). |
| `town-night-crop.jpg` | Linenleigh from 3 km at full size: ragged points over a faint warm patch. |
| `deck.jpg` | Under the storm deck Ashenstead (centre of the 3 km shot) is down to a few faint sparks; the towns in the clear beside it are whole. |
| `flowers.jpg` | Before, the meadow and the bank beside it are lit by the glowing flowers' pools of light; after, a quarter of them glow, at a quarter of the head's brightness and level 3 instead of 6, and the meadow is dark but for a few faint patches. (The eye adapts, so the whole frame is not exactly a quarter.) |

## Not shown

- The player's far lights. The threshold is pinned by tests
  (`night_lights::tests::the_players_far_light_appears_past_the_threshold` in
  `pbd-app`, `the_players_lights_show_once_a_group_reaches_the_threshold` in
  `pbd-core`); the capture's scripted `--lamps` row places fewer lamps than the
  threshold, so no capture shows one.
- `LATE_PIPELINE`: the set of pipelines a capture logs after `READY` is the
  same 19 names before and after (none added by this change). They are there
  at `2c17e84` already and are not this change's.
