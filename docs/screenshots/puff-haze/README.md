# puff-haze: overlapping puffs without an outline

The owner, 2026-11-13: "overlapping clouds have a weird outline". The write-up
is `openspec/changes/puff-haze/`.

All shots are fixed-step captures (`--capture`, 240 frames) on the M4 Max Mac
(Apple M4 Max, Metal), 2880 x 1800, each in a fresh world deleted before and
after the run (`./saves/<world>` under the repository root). Same arguments
and the same frame before and after. The before is `claude/perf-tuning`
a8376bd; the after is the same commit with this change. A capture is
deterministic: the ground shot taken twice is identical, the high shot
differs in 14 pixels by one level. The change was then rebased onto
538751c, whose two commits touch only the lens drops; the three after shots
taken again there match these within one level.

## Where

Every shot is at the default spawn's ground, read back by the game:

| shot | `--at` | map | elevation | above ground | `--time` | sun | weather |
| --- | --- | --- | --- | --- | --- | --- | --- |
| north | 24.117 -2.964 | 24.117S 2.964W | 65.6 m | 1.6 m | 12 | `--yaw -174.8 --pitch 61.4` (61.4 degrees up) | cover 0.55, rain 0.01 |
| south | 24.117 -2.964 | 24.117S 2.964W | 65.6 m | 1.6 m | 12 | the same | cover 0.55, rain 0.01 |
| high | 24.117 -2.964 | 24.117S 2.964W | 964.0 m | 900.0 m | 12 | the same | cover 0.55, rain 0.01 |

From the log: `PLACE capture <png> at 24.117 -2.964 (map 24.117S 2.964W) |
65.6 m elevation | 1.6 m above ground` (964.0 m and 900.0 m for the high
shot), `sun from the spawn at --time: --yaw -174.8 --pitch 61.4`, and
`Weather here: cover 0.55, rain 0.01`. `--weather-at 1800` starts the
weather field half an hour into its own time, which gives this spot a half
cover and no rain.

## Launch lines

Run from the repository root, `<w>` a fresh world name:

```text
north:  target/fast/pbd-app --world <w> --walk --at 24.117 -2.964 --time 12 --pitch 35 --yaw 90 --weather-at 1800 --frames 240 --capture north.png
south:  target/fast/pbd-app --world <w> --walk --at 24.117 -2.964 --time 12 --pitch 35 --yaw 270 --weather-at 1800 --frames 240 --capture south.png
high:   target/fast/pbd-app --world <w> --view column --at 24.117 -2.964 --height 900 --pitch -36 --yaw 90 --time 12 --weather-at 1800 --frames 240 --capture high.png
```

North looks 35 degrees up toward compass north (Tahal 2.2 km on the compass
bar), south toward compass south. High stands a camera 900 m over the spawn's
ground looking compass north and 36 degrees down, onto the deck along the
planet's limb.

## Files

For each of `north`, `south` and `high`:

- `<shot>-before.jpg`, `<shot>-after.jpg`: the frame, downscaled to 1440 x 900.
- `<shot>-crop-before.png`, `<shot>-crop-after.png`: the outline at full
  resolution. North is cropped at 1950,850 (700 x 500), south at 1500,950
  (900 x 500), high at 500,540 (900 x 400).
- `<shot>-crop-before-after.jpg`: the two crops side by side.
- `<shot>-difference.jpg`: after minus before in brightness, times six; red
  brighter, blue darker. Nothing brightens (at most +1, +1 and +3 levels);
  the far puffs inside near puffs' footprints darken (at most -18, -27 and
  -51), taking their own haze.
- `<shot>-haze-off.jpg`, `<shot>-crop-haze-off.png`: the diagnostic. The
  before build with `cloud_haze: 0.0` in `assets/config/weather.ron`, the
  same launch line: no outline anywhere, so the outline is the haze.

The first try (design 3b), hazed puffs laid plainly "over", not shipped:

- `try1-north.jpg`: its north frame.
- `try1-<shot>-difference.jpg`: it against the before, as above. Every stack
  of puffs brighter inside (red), by up to 48, 44 and 32 levels.

## What to look at

- `north-crop-before-after.jpg`: before, a front puff's silhouette runs from
  the top right to the bottom middle as a hard line with a lighter fringe
  inside it; after, the overlap is soft. Brightness along row 350 of the crop,
  every third pixel from x 270:
  - before `174 174 176 176 176 177 177 177 178 178 178 178 179 179 179 172 165 164 165 165 164 165 165 165`
  - after `161 161 162 162 162 163 163 163 164 164 163 163 164 164 164 165 165 164 165 165 164 165 165 165`
- `south-crop-before-after.jpg`: the same, several times, and as a texel
  staircase where a silhouette runs level.
- `high-crop-before-after.jpg`: at the limb, a nearer puff stood out of the
  hazed deck with a hard edge, and a corner was cut; after, the deck fades
  into the haze.
