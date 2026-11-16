# REFLECTIONS: OFF, LOW, MEDIUM, HIGH (`reflection-quality`)

The owner, 2026-11-16: "add SSR quality to settings as well off, low, med,
high (low not having that raycast thing where it finds the geometry facing
groundward)". What each level runs is in
`openspec/changes/reflection-quality/design.md`.

Fixed-step captures on the Mac (Apple M4 Max, Metal, `--profile fast`),
frame 240, 2880 x 1800 scaled to 1440 x 900, each in a world made fresh for
the shot, forced rain, RAIN SHADOW on (a capture's own default). The rain
splashes from `claude/env-v2` are in. A `-sheet.jpg` puts the four levels
side by side: OFF top left, LOW top right, MEDIUM bottom left, HIGH bottom
right.

`pbd-app --world <new> --walk <where> --time <10|22> --rain 1 --reflections <off|low|medium|high> --frames 240 --capture <png>`

| Shots | Where (`<where>`) | What to look at |
| --- | --- | --- |
| `lane-1000-*.jpg` | Holbrook's lane, `--at 29.69673 1.19287 --yaw 179.4 --pitch -4`, 10:00 | The flooded ground at the two houses' feet: HIGH and MEDIUM show the walls in it; LOW the screen trace's softer walls; OFF the sky's grey sheen |
| `lane-2200-*.jpg` | the same, 22:00, the windows lit | Lamp and window light on the wet lane at every level; no level goes black |
| `beach-1000-*.jpg` | the owner's beach, `--at 12.948 8.135 --yaw 90 --pitch -6`, 10:00 | A wet ledge under an overhanging bank: HIGH and MEDIUM find the bank's underside (brown) with the voxel walk, LOW cannot (the screen never saw it) and shows the dark wall the screen did see, OFF shows the sky |
| `beach-2200-*.jpg` | the same, 22:00 | The night: the sea and the ledge dim, never black |
| `menu-picture.jpg` | `--menu settings --settings-tab picture --reflections medium` | The PICTURE tab: Reflections (a four-step slider reading OFF, LOW, MEDIUM, HIGH) and Rain shadow, moved here from WEATHER |

What the captures show:

- **HIGH** is today's picture: the same code path, the same uniform values.
- **MEDIUM** is hard to tell from HIGH at eye height: most of what a puddle
  reflects is within 24 m.
- **LOW** loses what only the walk can find, the faces turned to the
  ground (the bank's underside on the beach), and keeps what was on screen.
- **OFF** reflects the sky alone: the wet ground and the water read as a
  grey sheen by day and dim at night, and there is no black.

Differences between captures that are not the level (the lens drops, the
splashes, the tufts' sway) come from each shot being a fresh world.

## No late pipelines

Real-time walks (the loading screen, `READY`, then play), logged with
`--frame-log`, on the fast build:

| Run | `LATE_PIPELINE` lines |
| --- | ---: |
| start at OFF, LOW, MEDIUM, HIGH (`--reflections <level> --walk-distance 120`) | 0, 0, 0, 0 |
| start at HIGH, `PBD_CYCLE_REFLECTIONS=45`: 737 switches through every level over a 458 m walk | 0 |
| start at OFF, `PBD_CYCLE_REFLECTIONS=20`: 22 switches through every level | 0 |

`PBD_CYCLE_REFLECTIONS=<frames>` steps the player's setting to the next
level every that many frames after `READY`, as the menu's slider does.
