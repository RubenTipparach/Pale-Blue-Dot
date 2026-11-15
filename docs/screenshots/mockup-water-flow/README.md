# Mockup M28: water that flows (`docs/mockups/water-flow.html`)

openspec/changes/water-flow. Headless Chromium on SwiftShader (no GPU in the cloud session), 1280 × 800 at 2x: the look,
never the frame rate. Each preset was settled through the page's `window.__water` hook and checked against "Settle
from scratch" (design 6): **0 cells differ in all six**.

| Shot | Preset | Shows | Settled |
| --- | --- | --- | --- |
| `spring.png` | Spring on the ledge | One source on the ledge's edge: it spreads, pours off the cliff in falling sheets (pale), spreads again on the plain and falls into the basin. | 22 ticks; 380 flowing, 126 falling |
| `channel.png` | Channel from the sea | A channel one layer deep from the beach to the dry basin: the sea runs in and the basin floods, a level lower each cell. | 7 ticks; 22 flowing |
| `basin.png` | Three sources make a pool | Three sources round the basin's middle cell: it and the three between renew (7 sources); the outer ring stays flowing. | 4 ticks; 7 new sources |
| `pair.png` | Two sources stay a pair | Two sources side by side with the hex threshold of three: the sheet spreads to range and stops; no source forms. | 13 ticks; 2 sources, 208 flowing |
| `runaway.png` | Luanti's threshold on hex | The same pair with Luanti's threshold of two: 317 new sources flood the plain to the patch's edge. | 19 ticks |
| `recede.png` | Take the spring away | The spring settled, then dug out: everything recedes to the generated patch. | 31 ticks after the dig; 0 flowing |
| `spring-night.png`, `channel-night.png`, `basin-night.png` | | The same at 22:00: each lamp lights the water and ground round it, dark between. | |

## Synced with M25's Water sim (2026-11-15)

The owner: "sync m28 with these changes" (M25's frothy falls, flow streaks and flow arrows). M28's water is now drawn by
`docs/mockups/water-flow/water-look.js`, which M25 loads too (design 7a). Same renderer and resolution as above
(SwiftShader, no GPU: the look, not the frame rate), 1280 × 800 at 1x, the `.view` element only. Each pair is the same
preset, settled, from the same camera.

| Before | After | Shows |
| --- | --- | --- |
| `sync-spring-before.png` | `sync-spring-after.png` | The spring off the ledge: the flat pale sides become foaming fall walls down the cliff; arrows on every flowing cap point downstream. |
| `sync-spring-night-before.png` | `sync-spring-night-after.png` | The same at 22:00: the seven lamps light the water and the foam near them, dark between; the arrows are dimmed at night. |
| `sync-channel-before.png` | `sync-channel-after.png` | The channel from the sea: the arrows run from the beach into the basin. |
| `sync-basin-night-before.png` | `sync-basin-night-after.png` | Three sources make a pool, at night: the outer ring's arrows point out of the pool of sources. |
| | `sync-390-after.png`, `sync-390-modal-after.png` | 390 px wide: no horizontal scroll; the descriptive text in its modal. |
| | `sync-m25-sim-after.png` | M25's Water sim tab on the shared file, unchanged in look (its Waterfall preset, Close view). |

Checks (Playwright): every M28 preset settles in the same ticks to the same source, flowing and falling counts as
before, and "Settle from scratch" differs by 0 cells (spring 22 ticks, 580 / 380 / 126; channel 7, 580 / 22 / 1;
basin 4, 586 / 12 / 0; pair 13, 581 / 208 / 12; runaway 19, 896 / 38 / 19; recede 31 after the dig, 579 / 0 / 0).
Both pages: no console errors, one WebGL context; camera and every object's rotation unchanged over 2 s idle. Spring:
506 fall sheets, 205 arrows, each along its cap's flow vector, with lower water within 60 degrees of where it points.
