# high-haze: before and after

The `high-haze` change (`openspec/changes/high-haze`): the atmosphere shell's
column of in-scattered light was laid over the ground in the one frame the eye
left the shell, 960 m above the sea, as a milky blue-white veil.

## How they were taken

- 2026-11-14, on the M4 Max Mac (Metal, the GPU), 2880 x 1800, the fast build
  (`cargo build --profile fast -p pbd-app`) at `3b44fb5` plus the change.
  Fixed-step stills: `--capture <png> --frames 200`.
- Every shot a new world (`--world hh-<tag>-<height>-<n>`), at the owner's spot:
  `--view column --at 16.86 57.63 --height <m> --pitch -42 --time 11 --rain 0
  --weather-at 3600`. The ground there is 76 m above the sea, so the shell's
  top is 884 m above the ground. The log said `Weather here: cover 0.00,
  rain 0.00` for every shot. (`--weather-at 0` there is a storm, cover 1.00.)
- Orbit: the same with `--height 9000 --pitch -70`.
- "Before" is the shader at `3b44fb5`; "after" is the change. The after shots
  at 850, 895, 1000, 2000, 3000 m and orbit were the first trial
  (`GROUND_GAIN_FROM` 0.3), which was kept unchanged.
- Sheets built with `tools/contact_sheet.py`.

## The sheets

| File | What it shows |
| --- | --- |
| `ladder.jpg` | 600-3000 m above the ground, before and after, two pairs a row. Before: identical to after up to 875 m, then the veil from 895 m on. After: one continuous climb. |
| `crossing.jpg` | The crossing up close: 875 m (951 m above the sea, inside the shell), 895 m (971 m, outside) and 1000 m, before and after. |
| `isolation.jpg` | 1000 m as shipped, with `PBD_NO_SKY` (the shell unspawned: the veil is gone) and with `PBD_NO_CLOUDS` (unchanged: not the clouds). |
| `orbit.jpg` | 9000 m: before, the land washed blue-white; after, the land reads and fades into the limb's blue. |

## The ground's numbers

A patch of the frame (15-85% across, 55-88% down: ground, clear of the HUD),
mean sRGB and blue's spread (its texture):

| Above ground | Before R, G, B | Before blue spread | After R, G, B | After blue spread |
| ---: | --- | ---: | --- | ---: |
| 600 m | 92, 114, 90 | 33 | same | 33 |
| 800 m | 109, 129, 101 | 32 | same | 32 |
| 875 m | 105, 130, 101 | 32 | same | 32 |
| 895 m | 111, 147, 164 | 11 | 104, 130, 101 | 32 |
| 950 m | 108, 146, 163 | 12 | 101, 129, 99 | 31 |
| 1000 m | 105, 145, 161 | 13 | 98, 128, 97 | 31 |
| 1200 m | 94, 138, 156 | 18 | 86, 122, 93 | 28 |
| 2000 m | 70, 117, 156 | 29 | 66, 106, 112 | 45 |
| 3000 m | 52, 93, 130 | 64 | 49, 86, 108 | 62 |

`PBD_NO_SKY` at 1000 m: 98, 129, 97, spread 31. Past 1200 m the patch takes in
more sea, so its blue rises with the view, not the haze.

## Not shown

- An overcast or dusk climb (the change does not read the cover or the hour,
  but it was not captured).
- Real-time smoothness: a fixed-step capture shows the look, not the frame
  time. The frame cost was not measured (no perf runs on this Mac today).
