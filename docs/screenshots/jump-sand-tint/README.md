# The jump over a low beach: the dry sand's blue-green tint, before and after (ocean-environments B21)

The owner, 2026-11-16: "dry ground had the weird water blur pass that we
fixed earlier. This only happens when you jump above water", "you can see the
subtle sand tinted blue green". On 2026-11-17: "I need screenshots proving the
jump over water with low layer blocks fix", and the method: "You could try
lifting the player gradually until you see a change in the sand color".

## What the stills show

The eye is held over a dry sand beach beside shallow sea at low tide, and
raised 1 cm a frame from standing (1.60 m over the sand) to 4.00 m. The
camera's look and the clock stay fixed, so the two builds draw the same
frames. Below 2.00 m the two builds are identical to the bit. From exactly
**2.00 m over the sand** the before build tints the whole dry beach in front
of the eye blue-green and blurs it, the owner's bug. The after build does not.

2.00 m is where the eye leaves the high tide's reach: the sea's level plus the
tide's high water, 2/3 of the 3 m range. That reach is the rule in
`eye_water_at`. Below it the eye is over the sea's water and the composite
measures the waterline from the tided sea. Above it the eye is in air:
- **before**, `sea_frame.y` was 0 and the shader fell back to the sea's MEAN
  sphere, which at low tide lies over the dry sand, so the sand was drawn as
  under water;
- **after**, it is the tided sea, as it is for the eye below (`composite_surface_radius`).

A jump's apex is about 2.8 m, so every jump from this sand crosses the
trigger.

| File | What it is |
| --- | --- |
| `threshold-sheet.jpg` | Rows are before and after; columns are the eye at 1.99 m and 2.00 m (one frame apart). Each tile is captioned with whether the eye counts as over the sea's water or in air, and the 3x crops of the boxed sand are below. **Start here.** |
| `jump-sheet.jpg` | The same rows, with the eye standing (1.60 m) and at a jump's apex (2.80 m). |
| `jump-sheet-bright.jpg` | The same as `jump-sheet.jpg` under a bright sky (`--weather-at 450`). The bottom-left of the frame lies in a tree's shadow there. |
| `sand-vs-eye-height-overcast.png`, `-bright.png` | The sand box's mean R and G against eye height for both builds, with the before-minus-after difference underneath. |
| `sand-vs-eye-height-*.tsv` | The numbers behind the charts: one row per build per cm. |
| `waterline-sheet.jpg` | The eye at the waterline, 9 m out from the beach, looking back at the dry bank: 5 cm over the tided sea, at it, and 2 cm under. |
| `rise.path`, `waterline.path` | The `PBD_EYE_PATH` files. |

## Numbers

These are the mean colour of the sand box: x 20-320, y 440-520 of the 960x540
frame, the lower left, dry sand in every frame of the rise. The values are
0-255 sRGB.

| Sky | Build | 1.99 m to 2.00 m (one frame) | Before minus after at 2.00 m | Before minus after at the apex, 2.80 m |
| --- | --- | --- | --- | --- |
| overcast (`--weather-at 200`) | before | **R -9.10, G -2.27, B -1.19** | R -9.19, G -2.39, B -1.38 | R -10.29, G -2.71, B -1.60 |
| overcast | after | R +0.09, G +0.12, B +0.19 | | |
| bright (`--weather-at 450`) | before | **R -11.32, G -2.47, B -0.97** | R -11.54, G -2.67, B -1.11 | R -13.77, G -3.19, B -1.38 |
| bright | after | R +0.22, G +0.20, B +0.14 | | |

- **The step is the largest change of the whole rise** in the before build.
  Elsewhere the box's R moves a median 0.07 (overcast) or 0.14 (bright) per cm,
  as the view rises. The after build's largest one-frame change anywhere on the
  rise is under 0.7.
- **Red falls about four times as far as green.** The sand loses its warm
  yellow and turns blue-green. Measured over a whole tile, before against after
  at 2.00 m differs on about 10% (overcast) to 23% (bright) of pixels by more
  than 8 levels. Those pixels are the near beach and the far terraces. The
  edges between sand grains also smear, which is the wobble.
- **Read the after curve as smooth.** It rises steadily with the eye, because
  a higher eye sees the sand at another angle and more of its lit top. It has
  no step at 2.00 m or anywhere else.
- **Below 2.00 m, before minus after is 0.00 in every channel at every
  frame.** The builds differ only in this rule and in what the eye sees under
  water (B20), so nothing else moved the dry beach.

## The eye at the waterline

`waterline-sheet.jpg` shows the dry sand bank on the shore, seen from the eye
in the sea at the waterline:
- 5 cm over the sea, and exactly at it: clean and identical in both builds,
  with no tint or wobble on the dry sand;
- 2 cm under the sea: the bank is behind the surface. The before build
  mirrors the sea floor there; the after build shows the murk ceiling. That
  is B20 (no mirror under the surface), not this fix.

## Where and how

- **Where:** a jungle beach at `--at 39.53 14.026`, looking east (yaw 172)
  and 30 degrees down across the dry sand to the sea. The dry sand is the
  beach's lowest step, and the sea at low tide is 3.10 m under the standing
  eye. This is the beach of the earlier `underwater-waterline` jump video,
  3 m further south, so the moored boat stays out of the frame.
- **Launch:**
  - `--fly --at 39.53 14.026 --height 2 --yaw 172 --pitch -30 --tide low --time 14 --weather-at 200 --frames 660`;
  - `PBD_WINDOW=960x540`;
  - `PBD_EYE_PATH=rise.path`: the eye is held at 1.6 m for 400 frames, then raised to 4.0 m over 240 frames;
  - `PBD_CAPTURE_RUN=<dir>,395,250`: every frame from 395 on.
- **Frame to height:** a frame's eye height is `1.6 + (frame - 400) / 100` m.
  The label in each tile's corner gives the frame (f439, f440, ...), and the
  `EYE_PATH` log lines give the eye's altitude and the renderer's own call:
  - `judged Some(-1.5)` means over the sea's water, with the tided sea at -1.5 m;
  - `judged None` means in air.
- **Machine:** the Mac (M4 Max, Metal, the GPU), `--profile fast`, with
  the capture's fixed 60 fps step. These are looks, not frame times.
- **Builds:**
  - **before** is `4573767d`, `claude/env-v2` just before the tided waterline
    for an eye in air landed;
  - **after** is `9019207b`, `origin/claude/env-v2` now.

  Each binary ran with its own `assets/`.

**Which commit is "before":** the line that made an eye in air use the tided
sea came into `claude/env-v2` with PR #94 (`02060dea`, the fishing line's
`80c27125`, for the float at low tide), one merge ahead of PR #91
(`8fa25d65`). PR #91 then put it into `composite_surface_radius` with its
test. So the first parent of #91's merge already has the fix, and draws the
after picture. The real before is #94's first parent, `4573767d`.

## Not covered

- A real jump played through the walking controller was not filmed. The
  rise is the same eye heights held still, and the owner's method. The
  trigger is the eye's height, not its speed, and the 1 cm steps bracket it.
- These sheets do not measure why this sand counted as under the mean sphere,
  for example how far the sand's drawn top lies under it. They show only that
  it did, and from what eye height.
