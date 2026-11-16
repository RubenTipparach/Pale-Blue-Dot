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
