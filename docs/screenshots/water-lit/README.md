# water-lit: the water is lit, not luminous

Fixed-step captures on the Mac (`--profile fast`), frame 240, each in a
world made fresh for the shot, at the owner's beach: `--at 12.948 8.135`
(map 12.948S 8.135E), on foot, eye 2.6 m above sea level and 1.6 m above the
ground, heading 180, 8 degrees down. Before = `claude/perf-tuning` at
`65d414a`; after = this change.

| Shot | `--time` | Sun | `Weather here` |
| --- | ---: | ---: | --- |
| noon | 12 | 68.2 deg up | cover 1.00, rain 1.00 |
| dusk | 19.5 | 3.0 deg up | cover 0.74, rain 0.00 |
| night | 22 | 28.9 deg under | cover 1.00, rain 0.77 |
| under (`--swim`: it stood the walker at the waterline, 0.6 m above the sand at 12.936 8.523, map 12.936S 8.523E, not under water) | 12 | 68.2 deg up | cover 1.00, rain 1.00 |

`pbd-app --world <new> --walk --at 12.948 8.135 --yaw 180 --pitch -8 --time <t> [--swim] --frames 240 --capture <png>`

- `before-sheet.jpg`, `after-sheet.jpg`: all of them (the "dry" tile is the
  noon launch with `--rain 0`, which did not stop the rain; ignore it).
- `before-after-<shot>.jpg`: each side by side.

Night, mean linear luminance over the same regions: the sea 0.0091 before,
0.0055 after; the sand beside it 0.0137 both; the sky just over the horizon
0.0028. The sea is now darker than the beach; it is still about twice the
night sky drawn over the horizon, because it mirrors the L2 sky that also
lights the ground (an overcast night "an even grey-blue, never black") and
the dome draws no such glow at night.
