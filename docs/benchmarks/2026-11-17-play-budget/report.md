# Where the frame goes in real play (2026-11-17)

`openspec/changes/play-frame-budget`. The showcase bot played the phase-1
branches on one new world through the player's own inputs
(`python tools/play_budget.py run`): the trunk and fishing to the end,
farming, herding and cooking until they stalled (the seed search and the
hen, the bot's known weak spots). 1055 s of play after warm-up. Fishing is
398 s cast from the sea shore by the pasture start.

The owner's desktop, RTX 3070, the `fast` build, windowed, player-settings
defaults, a clear morning (8:30). The raw frame logs (18 MB) are not checked
in; `summary.json` holds every number below.

## The whole session

| | mean ms |
| --- | ---: |
| Frame (p50 17.4, p99 42.6; 50.9 fps) | 19.6 |
| GPU | 16.6 |
| Main world CPU (the game's systems) | 13.5 |
| Render world CPU (its own work) | 6.8 |
| Render world waiting on the swapchain | 9.4 |

The GPU is the longest of the three in 69% of frames, the main world in
31%, the render world's own work in none.

## GPU by pass, mean over the session

| Pass | ms |
| --- | ---: |
| `planet_terrain` | 7.98 |
| `planet_shadow` | 7.25 |
| `water_compose` | 0.44 |
| `planet_cull` | 0.23 |
| everything else | under 0.6 together |

Terrain and its shadow cascades are 92% of the GPU's time. Clouds, rain and
water cost almost nothing on a clear morning.

## By activity

| Activity | fps | GPU | main CPU | shadow | terrain |
| --- | ---: | ---: | ---: | ---: | ---: |
| Build a crafting bench (trunk 5) | 37 | 25.6 | 13.9 | 13.4 | 11.1 |
| Fishing at the sea (fishing 2, 398 s) | 49 | 19.2 | 12.8 | 9.4 | 7.8 |
| Tame a hen (herding 3) | 59 | 11.3 | 13.9 | 3.5 | 6.8 |
| Grill the catch (cooking 1) | 59 | 7.7 | 12.0 | 4.4 | 2.3 |

The GPU moves from 8 to 26 ms with where the camera looks: across open land
toward the horizon the shadow and terrain passes double. The main world
stays at 12-15 ms whatever the activity, so once the GPU is under it, it is
the floor (about 60-70 fps).

## What follows

1. The shadow pass: keep casters out of cascades whose receivers they
   cannot shade (the plan of `main-perf-2026-11`). Same picture.
2. The terrain pass at a distance.
3. The main world's 13 ms: a breakdown by system is the next instrument.

Effects (clouds, rain, water) are not where this budget goes in clear
weather; a storm sitting is needed before any of them is dialled back.
