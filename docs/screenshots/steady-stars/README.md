# steady-stars: stars that do not flicker

Six consecutive fixed-step frames (120 to 125) of one night launch on the
Mac (`--profile fast`): `--at 20 40` (map 20.000S 40.000E), on foot, eye
48.6 m above sea level and 1.6 m above the ground, 02:00 (sun 30.5 degrees
under), `Weather here: cover 0.00, rain 0.00`, looking 60 degrees up and
turning 3 degrees a second.

`pbd-app --world <new> --walk --at 20 40 --time 2 --yaw 60 --pitch 60 --turn 3 --frames <120..125> --capture <png>`

Each star found in one frame is matched to the nearest in the next (within 5
px) and its light summed over 5 x 5 pixels in linear light
(`tools/star_flicker.py`):

| | Stars followed | Gone the next frame | Light changed > 25 % | Median change | All stars' light, frame to frame |
| --- | ---: | ---: | ---: | ---: | ---: |
| before (quads in metres) | 663 | 83 (12.5 %) | 24 % | 1 % | 3.4 % (worst 4.8 %) |
| after (Gaussians in pixels) | 729 | 0 | 0 % | 0 % | 0.4 % (worst 1.5 %) |

`before-after-crop.jpg`: the same patch of sky at frame 120, before left,
after right.
