# weather-rain: splashes where rain lands (tasks 6.4, 6.6)

`openspec/changes/weather-rain`, "Splashes where rain lands", the approved M32
mockup (`docs/mockups/rain-splashes.html`) ported into the game.

## How they were taken

- An M4 Max Mac, Metal, on the GPU, 2880 x 1800, the fast build
  (`cargo build --profile fast -p pbd-app`) of `claude/rain-splashes` at
  `49ceaa5` (the shader's lamp term was raised once more before the shots,
  and that is the committed shader).
- Fixed-step captures, frame 200, each in a world made fresh for the shot,
  at Holbrook's south path (the `rain-stroll` perf scenario's start):
  `pbd-app --world <new> --walk --at 29.69673 1.19287 <view> --rain 1 --frames 200 --capture <png>`.
  Every run logged `Weather here: cover 1.00, rain 1.00`.
- **Before and after are the same binary**: "before" ran with
  `PBD_PASS_OFF=splash`, which skips only the splash draw, so the two frames
  differ in the splashes and nothing else (13,151 / 4,031 / 11,969 / 3,423
  pixels differ by more than 8 levels of 5.18 M, path day / meadow day /
  path night / meadow night).
- Full frames are JPEG (quality 90), downscaled to 1440 x 900; the crops are
  PNG, 450 x 300 px of the full frame enlarged three times, nearest
  neighbour. Each crop is the 450 x 300 window with the most changed pixels
  (the magenta box on the sheet).

| Files (`splash-<shot>-before/after.jpg`, `-crop-x3.png`) | View flags | What it shows |
| --- | --- | --- |
| `path-day` | `--yaw 179.4 --pitch -25 --time 11` | The puddled dirt path by day. After: small grey-white square droplets in clusters of five to eight over the puddles (crowns) and lower ones on the wet dirt, beside the rings the ground already had. |
| `meadow-day` | `--yaw 90 --pitch -25 --time 11` | The meadow by day. The grass spray is sparse and faint: a few clusters of one- or two-pixel droplets, nearly the grass's own colour. The changed pixels the crop finds are a crown on the path at the bottom of the frame. |
| `path-night` | `--yaw 179.4 --pitch -22 --time 22 --lamps 5` | The path at night under the six test lamps. The droplets near the lamps are warm and pale; away from them they fade into the dark. |
| `meadow-night` | `--yaw 90 --pitch -25 --time 22 --lamps 4` | The meadow at night by the lamps: as by day, the grass takes very few visible droplets; the crop's change is a lamp-lit crown on the path's edge. |

`splash-sheet.jpg` is all four: before, after, and the two enlarged crops.

## What they show, honestly

- On the path the splashes read as the mockup's: crowns of pixel squares over
  the puddles, no outline, coloured between the wet ground and the grey sky.
  At 2880 x 1800 a droplet is two to four pixels near the feet and one at
  the reach, so a full frame shows them only as a faint speckle; the crops
  are where they are seen.
- On grass they are barely there. That matches the approved mockup's softer
  spray (half the rate, four small droplets, 3-5 cm high), but in the game's
  bright, busy grass it is close to invisible in a still. If the owner wants
  the meadow to read, `splash_keep[2]` and `splash_size_m[2]` in
  `weather.ron` are the knobs.
- A still cannot show motion; the splashes live about a third of a second.
- The lens drops (the large white rounded marks) are the existing lens
  effect, unchanged.

## Not shown

- No video: screenshots stand in until the owner's batch (CLAUDE.md).
- The frame cost is in `docs/benchmarks/2026-11-15-rain-splashes/`.
