# A height fog with RAIN SHADOW off (`weather-rain` 7)

The owner, 2026-11-16: "when on the planet, if we have rain shadow turned
off can we just do normal distance fog?", then "or height fog?". RAIN SHADOW
is the whole rain volume (CLAUDE.md, "What the owner means by rain shadow").
The design is `openspec/changes/weather-rain/design.md`, "Height fog with
RAIN SHADOW off".

Fixed-step captures on the Mac (Apple M4 Max, Metal, `--profile fast`),
frame 240, 2880 x 1800 scaled to 1440 x 900, each in a fresh world, forced
rain, with the rain splashes from `claude/env-v2`. Each `-sheet.jpg` is
three shots of one view: RAIN SHADOW off as it was (`PBD_HEIGHT_FOG=0`, the
fog's lane zero: today's picture), RAIN SHADOW off with the height fog, and
RAIN SHADOW on (the rain volume) for comparison.

`[PBD_HEIGHT_FOG=0] pbd-app --world <new> <where> --rain 1 --rain-shadows <off|on> --frames 240 --capture <png>`

| Shots | Where (`<where>`) | Weather here |
| --- | --- | --- |
| `lane-12-*` | `--walk --at 29.69673 1.19287 --time 12 --yaw 179.4 --pitch 0` (Holbrook's lane) | cover 1.00, rain 1.00 |
| `beach-12-*` | `--walk --at 12.948 8.135 --time 12 --yaw 90 --pitch 2` (the owner's beach) | cover 1.00, rain 1.00 |
| `beach-22-*` | `--walk --at 12.948 8.135 --time 22 --yaw 0 --pitch 2`, at night | cover 1.00, rain 0.77 |
| `valley-90-*` | `--fly --at 29.69673 1.19287 --height 160 --time 12 --yaw 90 --pitch -12`: over the valley toward Tahal | cover 1.00, rain 1.00 |
| `valley-0-*` | the same, `--yaw 0`: a hill and the low ground beside it | cover 1.00, rain 1.00 |

What they show:

- **Before**, a storm with RAIN SHADOW off has no murk at all: blue sky
  through the holes in the deck and the far hills sharp. That is the look
  the owner asked to replace.
- **The height fog** greys the storm out much as the volume does. On the
  beach shot the sky at the horizon reads (103, 112, 122) against the
  volume's (113, 122, 130), and the far sea (74, 69, 57) against (75, 70,
  59), out of 255; before, (11, 49, 76) and (67, 56, 31). So flipping the
  setting in a storm does not jump.
- **From 160 m up**, the murk lies below: the valley and the low ground are
  lost in it, the near slope under the eye shows, and the cloud deck above
  stays visible through thinner fog where the volume hides it.
- **At night** the fog is as dark as the night: it takes the rain's light
  at that hour (`rain_light`), as the volume does.
- What the cheap fog does not draw: the volume's streaks in the distance
  and the lamps' glow in the rain.

A first cut with a 120 m falloff left the horizon a third as murky as the
volume (on a 4.8 km body a level ray climbs 400 m in its 2 km); the shipped
`rain_height_fog_falloff_m` is 250 m.
