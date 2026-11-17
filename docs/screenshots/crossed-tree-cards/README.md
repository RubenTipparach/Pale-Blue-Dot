# Crossed tree cards: captures

For `openspec/changes/crossed-tree-cards`. Captured 2026-11-17 in a Claude
Code cloud container with **no GPU**: rendered on lavapipe (Mesa's software
Vulkan) under Xvfb, fixed-step `--capture` runs of the `fast` build at
1440x900, each on a new world deleted afterwards, saved as JPEG. They show
what things look like, never how smoothly they run. **Frame cost was not
measured** (no GPU here; the owner's `perf_suite.py` sitting measures it).

- **before**: this branch's start, `66780be5`, with that commit's `assets/`.
- **after**: the branch: far trees as two fixed crossed cards and a crown
  cap, the mid ring as four fixed cards and the cap, the ninth (top-down)
  bake view.

Each binary ran from a folder holding a copy of its own `assets/` (the
shipped-build layout, `paths::shipped_root`), so before and after read their
own shaders:

```text
xvfb-run -a -s "-screen 0 1440x900x24" ./pbd-app <flags> --world <new> --capture <png>
```

| file | flags | what it shows |
| --- | --- | --- |
| `before-high.jpg`, `after-high.jpg` | `--fly --spawn jungle --height 150 --pitch -55 --time 10 --weather-at 1800` | The owner's view, high over forest looking down. Before: every far tree is a card turned to the eye, so under the camera the forest is rings of streaks pointing at the view centre. After: the trees stand as fixed X's with a crown cap, and read as crowns from above. |
| `high-crop.jpg` | the same, crop at 2x | Before (left) and after (right) under the camera, where the streaks were worst. |
| `before-fields-30m.jpg`, `after-fields-30m.jpg` | `--fly --spawn fields --height 30 --yaw 300 --pitch -25 --time 10 --weather-at 1800` | Oblique from 30 m: near models in front, mid and far cards behind. After, the far oaks are fuller crowns (the X's two views and the cap) and the birch trunks stand under them; before, a single card each, with the bark lines laid over the crown. |
| `fields-30m-crop.jpg` | the same, crop at 2x | Before above, after below. |
| `before-fields-14m.jpg`, `after-fields-14m.jpg` | `--fly --spawn fields --height 14 --yaw 300 --pitch -4 --time 10 --weather-at 1800` | Just over the canopy, looking level across the valley to a forest on the hillside beyond: the far ring from the side. Those trees stand above the eye, so their caps are not drawn and what changes is the fixed X in place of the turned card: fuller crowns, less bark drawn over them. |
| `before-jungle-eye.jpg`, `after-jungle-eye.jpg`, `before-fields-eye.jpg`, `after-fields-eye.jpg` | `--walk --spawn jungle --yaw 270 --pitch 2 ...`, `--walk --spawn fields --yaw 300 --pitch 10 ...` (same time and weather) | On foot. The near models fill these frames, so the far ring is mostly hidden behind them; they show that nothing near the player changed. |

## The cap

Within the owner's "2 - 4 billboards", a small horizontal crown cap was
judged acceptable: it is fixed like the cards, and without it a forest seen
from straight above is a field of plus signs. **Recommendation taken (ask
only with screenshots)**: `after-high.jpg` and `fields-30m-crop.jpg` are the
screenshots to overrule it on. The cap is drawn only while the eye is above
it, so from the ground it is not there.

## What it draws

- Vertices invoked per tree in the impostor draw: **30, unchanged** (cards
  0-3, then the cap in the billboard's old slot).
- Triangles rasterised: a far tree **6, was 2** (two cards and the cap); a
  mid tree **10, was 8** (four cards and the cap).
- The tree kit: 6,078 KiB before, 6,672 KiB after (each bake has a ninth
  view), as logged at startup (`tree kit: ...`).

## Logs

No `ERROR` in any capture. `LATE_PIPELINE` lines: the same set before and
after in every pair (30 to 33 a run: Bevy mesh, UI, tonemapping, screenshot
and rain pipelines, and the GPU hex world's, which a fixed-step capture does
not prewarm). The change adds no pipeline.
