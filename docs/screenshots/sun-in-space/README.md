# sun-in-space: before and after

The `sun-in-space` change (`openspec/changes/sun-in-space`). The owner
(2026-11-14): "why did the sun dissapear? when I go to space". The sun was a
term of the atmosphere dome's shader, and from outside the shell the dome has
fragments only where a ray meets the shell, so a sun above the shell's rim was
not drawn and one across it was cut flat along it.

## How they were taken

- 2026-11-14, the M4 Max Mac (Metal, the GPU), 2880 x 1800, the fast build
  (`cargo build --profile fast -p pbd-app`). "Before" is `d43d018`; "after" is
  the change. Fixed-step stills, `--capture <png> --frames 200`, every shot a
  new world.
- The shot list is `shots.txt` (name, `--at` lat lon, `--height`, `--pitch`,
  `--yaw`, `--time`, extra flags), all `--view column`. The yaw and pitch facing
  the sun at a clock hour were worked out from `pbd_core::daylight::Clock::sun`
  on day 0 (the northern solstice): at 0 N 0 E from 6 km the shell's rim is
  57.8 degrees under the horizontal and the planet's limb 63.6.
- Sheets built with `tools/contact_sheet.py`; the close-ups are full-resolution
  crops round the sun.

## The sheets

| File | What it shows |
| --- | --- |
| `orbit.jpg` | From 6 km, before and after: the sun above the rim (before: gone, the glow's lower edge only; after: whole), on the rim (before: cut flat; after: round), through the air (both whole; after a little warmer, through the air's extinction), half behind the limb, behind the planet (hidden in both). |
| `rim-closeup.jpg` | On the rim, then through the air: before, after, before, after. |
| `limb-closeup.jpg` | Half behind the limb, before and after. Before, the dome added the sun's glow over the planet's face in front of it (a bright patch on the sea and land at the limb); after, the glow stops at the limb, since the sun is behind the planet. |
| `crossing.jpg` | Climbing out through the shell's top (960 m above the sea) with the sun 5.3 degrees under the horizontal: 900 m and 1050 m. Before, the sun vanishes in the 150 m climb; after, it is the same sun at both. |
| `unchanged.jpg` | On the ground at noon and at sunset, and inside a storm cloud: the same before and after. |

## Measured differences, before against after (pixels changed by more than 8 of 255)

| Shot | Whole frame | Centre half (round the sun) |
| --- | ---: | ---: |
| ground noon | 389 (the HUD, rain on the lens) | 0 |
| ground sunset | 529 (a cloud drifted) | 89, in the cloud, not the sun |
| in cloud | 1084 (HUD, cloud) | 0 |
| 900 m, inside the shell | 747 | 0 |
| behind the planet | 0 | 0 |

## Pipelines and errors

- A normal launch (with the loading screen's warm-up), standing on the ground,
  20 s after `READY`: `READY ... 92 pipelines`, **0** `LATE_PIPELINE` lines.
- A capture skips the warm-up, so every pipeline is logged as late there: 18
  before, 19 after (the new one, `SUN_DISC`, beside the dome's). No shader
  errors; the one `ERROR` line in the logs is a missing
  `textures/settlement/flame.png`, before and after.

Frame time was not measured (one quad of about 14 degrees, collapsed from
inside the shell). The perf suite is the desktop's.
