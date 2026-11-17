# The sea's horizon line (`horizon-haze` 4)

The owner, 2026-11-16: "what is this black line on the horizon?", and on
the fishing stills: "Now that horizon line on the water needs fixing".

- **Where:** open sea at `52.053 -118.900`, facing west (yaw 0, pitch 0),
  the camera on the capture-only eye path (`PBD_EYE_PATH`, `--fly`), the
  eye 1.7, 10 and 100 m over the sea's level. `fish` is the coordinator's
  view, `--fish --tide low --time 11` (24.000 122.460 E, eye 2.6 m,
  looking east).
- **Conditions:** noon `--time 1`; dusk `--time 7 --tide low`; full cover
  `--time 1 --weather-at 20000` (cover 1.00) with `PBD_PASS_OFF=lens`;
  `lowtide-10` and `hightide-10` at noon with `--tide low|high`.
- **Machine:** Mac M4 Max, Metal (the GPU), 2880x1800, `--frames 120`
  (`fish` 180), `--profile fast`.
- **Builds:** before is `origin/claude/env-v2` at `953d0d34`; after is
  `claude/sea-horizon-line`.
- **Files:** `<condition>-before.jpg` and `-after.jpg` are the frame at half
  size; `<condition>-crop4x.png` is 240x45 px of the full frame round the
  horizon, 4x nearest, before over after.

## Measured

The gap is every pixel a measurement line in the sky dome painted magenta
(a sky ray meeting the solid sphere with no sea drawn over it), in 183
columns across the open sea (347 in `fish`). Mean sRGB of those pixels,
of the sky 3 px over them and of the sea 3 px under them.

| Condition | Gap px (median) | Gap before | Gap after | Sky over it (after) | Sea under it |
| --- | ---: | --- | --- | --- | --- |
| noon 1.7 m | 11 | (16, 65, 105) | (122, 185, 247) | (122, 184, 247) | (98, 127, 124) |
| noon 10 m | 7 | (42, 104, 158) | (124, 187, 248) | (124, 186, 248) | (100, 129, 127) |
| noon 100 m | 4 | (81, 158, 230) | (129, 192, 249) | (129, 191, 249) | (104, 134, 134) |
| dusk 1.7 m | 11 | (63, 27, 2) | (64, 51, 52) | (66, 51, 52) | (7, 31, 49) |
| dusk 10 m | 6 | (53, 22, 7) | (59, 52, 54) | (62, 53, 55) | (7, 29, 47) |
| dusk 100 m | 2 | (29, 26, 33) | (48, 59, 64) | (50, 60, 65) | (11, 38, 56) |
| cover 1.7 m | 9 | (19, 34, 51) | (93, 123, 163) | (93, 123, 163) | (126, 132, 130) |
| cover 10 m | 4 | (27, 48, 73) | (92, 123, 164) | (93, 124, 165) | (127, 133, 132) |
| cover 100 m | 2 | (41, 74, 110) | (82, 121, 168) | (82, 121, 168) | (128, 134, 134) |
| low tide 10 m | 8 | (40, 102, 157) | (124, 187, 248) | (124, 186, 248) | (100, 129, 127) |
| high tide 10 m | 1 px in 4 columns | - | - | - | - |
| fish 2.6 m | 9 | (25, 43, 63) | (95, 124, 161) | (92, 120, 155) | (118, 125, 122) |

The sea under the gap is the same before and after in every row: only the
gap changed. The "sky over it" before is the same as after except under
cover, where the weather differs a little run to run.
