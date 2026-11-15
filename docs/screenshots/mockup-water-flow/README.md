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
