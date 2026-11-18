## Why

The owner, 2026-11-17, after the bot-played session
(`docs/benchmarks/2026-11-17-play-budget/`) found the sun's shadow pass
7.3 ms of a 16.6 ms GPU frame (up to 13 ms looking across open land): "oof,
reduce to 2? 1024 in size, and is there some way to blur them? also add
shadow quality slider".

Per cascade on the pasture view (`SHADOW_SPANS`, the desktop): the pass is
bound by vertices, not fill. Four cascades of 2048 ran 4.1, 4.4, 6.1 and
12.0 million vertex invocations; the far one is redrawn every fourth frame.

## What changes

- **SHADOW QUALITY**, a player setting on PICTURE (`player-settings`
  version 10): LOW, MEDIUM, HIGH, ULTRA.

  | Level | Cascades | Map side | Soft least radius | 3 x 3 spacing |
  | --- | ---: | ---: | ---: | ---: |
  | LOW (new default) | 2 | 1024 | 1.5 texels | 1.35 |
  | MEDIUM | 3 | 1024 | 1.0 | 1.15 |
  | HIGH | 4 | 1024 | 0.75 | 1.0 |
  | ULTRA (as before) | 4 | 2048 | 0.5 | 1.0 |

  An older settings file reads LOW. A capture or benchmark runs ULTRA, the
  shadows as they were, so no perf comparison moves. `--shadows` sets it.
- **The blur** is the soft filter's least radius and the 3 x 3's spacing,
  in texels: wider at the lower levels, so a coarser map reads soft rather
  than blocky.
- **Fewer cascades split the same reach** (900 m on foot). The farthest is
  redrawn every fourth frame at every level, the finest every frame, a
  middle one every other: drawing two must not redraw the far reach each
  frame (it did in the first try, 12 M vertices a frame).
- The map stays one 2048 array. A lower level draws into its 1024 corner
  (viewport) and the receivers read that corner, so every material that
  binds the map keeps a live texture when the level changes.

## Measured

The desktop, `fast` build, the pasture start on foot, 9:00, mean over about
30 s after warm-up:

| Level | Shadow pass | Terrain pass | GPU total |
| --- | ---: | ---: | ---: |
| LOW | 1.76 ms | 2.45 ms | 5.16 ms |
| MEDIUM | 2.06 | 2.19 | 5.13 |
| ULTRA | 3.78 | 1.84 | 6.41 |

The terrain pass grows at LOW by the wider soft filter's taps. Screenshots:
`docs/screenshots/shadow-quality/low.png` and `ultra.png`, the pasture.

## Next

The finest cascade, a box a few metres across, still draws 4 M vertices: its
cull keeps nearly everything. Culling casters to what each cascade can shade
is the next cut, and keeps the picture.
