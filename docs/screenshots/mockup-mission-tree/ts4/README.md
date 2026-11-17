# Mission tree TS.4: M33's look in the game

Captures for `openspec/changes/starter-missions` TS.4.
- **Game:** in-game captures on 2026-11-17, in a cloud container on lavapipe
  with no GPU, at 1440 x 900, from the fast build. Each is a new
  memory-only world with 45 missions' worth of progress (`--missions 45`).
- **Mockup:** M33 rendered in Chromium (software).

| File | Shows | Launch line |
| --- | --- | --- |
| `game-x1.png`, `x1-beside-m33.jpg` | The native card (x1) at twice the gaps, each mission's icon beside its title, the side panel open on Go fishing | `pbd-app --capture game-x1.png --menu missions --missions 45 --tree-select go-fishing --frames 120` |
| `game-fit.png`, `fit-beside-m33.jpg` | Fit (F): the whole tree, each card its mission's icon filling its frame | `... --tree-select fit ...` |
| `game-hover.png`, `hover-beside-m33.jpg` | Zoomed out (x0.42), the full card shown beside the hovered Go fishing tile | `... --tree-zoom 4 --tree-select go-fishing --tree-hover go-fishing ...` (`--tree-hover` is a capture instrument: a headless run has no pointer) |

Frame cost was not measured.
