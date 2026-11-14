# rain-under-water: the rain volume is in the air, not in the sea

Fixed-step captures on the Mac (M4 Max, Metal, `--profile fast`), frame 200,
each in a world made fresh for the shot. Before = `claude/perf-tuning` at
`3b44fb5`; after = this change.

Under water: `--at 12.748 9.455` (map 12.748S 9.455E), the walker 5.4 m
under the sea over a 7 m deep floor, a moored boat floating overhead (the
dark hull and the sail in every frame), heading west. Rain was not forced
(`--rain 1` by day panics here, see below); the weather's own rain was 0.54
at night and 0.91 by day, cover 1.00.

`pbd-app --world <new> --walk --at 12.748 9.455 --pitch <20|45|70> --time <22|11> --rain-shadows <on|off> --frames 200 --capture <png>`

- `compare-t22.jpg` (night, brightened 2.5x) and `compare-t11.jpg` (day):
  rows look up 20, 45 and 70 degrees; columns are before with the rain
  volume on, after with it on, and before with it off.
- `before-t<hour>-p<pitch>-rs<on|off>.jpg`, `after-t<hour>-p<pitch>-rson.jpg`:
  each frame on its own.
- `before-beach-above-water.jpg`, `after-beach-above-water.jpg`: the owner's
  beach (`--at 12.948 8.135 --yaw 180 --pitch -8 --time 11 --rain-shadows
  on`, rain 0.93): the rain above water, unchanged.

Mean absolute difference per channel, 0-255, full frames:

| Shot | before, on vs off | after, on vs off | after off vs before off |
| --- | ---: | ---: | ---: |
| night, 20 deg | 12.55 | 0.00 | 0.00 |
| night, 45 deg | 16.85 | 0.00 | 0.00 |
| night, 70 deg | 14.06 | 0.00 | 0.00 |
| day, 20 deg | 63.72 | 0.00 | 0.00 |
| day, 45 deg | 82.51 | 0.00 | 0.00 |
| day, 70 deg | 71.42 | 0.00 | 0.00 |
| beach above water, before vs after (on) | | | 0.00 |

Before, the rain volume laid a grey fog over the whole underwater frame,
and looking up 20 degrees the shore's trees stood in it where, with the
volume off, the surface past Snell's window is a dark mirror that hides
them. After, the frame with the volume on is the frame with it off, pixel
for pixel; above water the rain is drawn exactly as before.

Seen in these shots, not changed here (the change's tasks, 3.3): at the
window's edge the sheet's rain rings print as dark rings over what is seen
through the surface (the hull), with the volume off too; the moon and the
stars through the window are sharp.
