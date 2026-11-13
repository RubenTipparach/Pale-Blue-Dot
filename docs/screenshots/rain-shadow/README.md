# RAIN SHADOW on and off (`player-settings` 8)

The rain shadow is the rain volume: the rain fog and the animated streaks
behind the near rain (CLAUDE.md, "What the owner means by rain shadow").

Fixed-step captures on the Mac (`--profile fast`), frame 240, each in a
world made fresh for the shot: `--at 12.948 8.135` (map 12.948S 8.135E, the
owner's beach), on foot, eye 2.6 m above sea level and 1.6 m above the
ground, 12:00 on the planet clock (sun 68.2 degrees up), forced rain:
`Weather here: cover 1.00, rain 1.00`.

`pbd-app --world <new> --walk --at 12.948 8.135 --time 12 --yaw 90 --pitch 2 --rain 1 --rain-shadows on|off --frames 240 --capture <png>`

- `on.jpg`: the rain fog greys the sky and the far trees.
- `off.jpg`: no rain volume. The near streaks, the wet ground and the lens
  drops are still drawn. The storm sky behind the fog shows as dark navy:
  the fog was what made a storm sky read grey.
- `on-off.jpg`: side by side.
