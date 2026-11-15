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
| `sync-basin-night-before.png` | `sync-basin-night-after.png` | Three sources make a pool, at night: the arrows run out of the pool's sources into the ring round them, lit by the basin's lamp. |
| | `sync-390-after.png`, `sync-390-modal-after.png` | 390 px wide: no horizontal scroll; the descriptive text in its modal. |
| | `sync-m25-sim-after.png` | M25's Water sim tab on the shared file (its Waterfall preset, Close view): as before, less the 6 cm slivers of froth noted below. |

Checks (Playwright): every M28 preset settles in the same ticks to the same source, flowing and falling counts as
before, and "Settle from scratch" differs by 0 cells (spring 22 ticks, 580 / 380 / 126; channel 7, 580 / 22 / 1;
basin 4, 586 / 12 / 0; pair 13, 581 / 208 / 12; runaway 19, 896 / 38 / 19; recede 31 after the dig, 579 / 0 / 0).
Both pages: no console errors, one WebGL context; camera and every object's rotation unchanged over 2 s idle. Spring:
499 fall sheets, 186 arrows, each along its cap's flow vector, with lower water (by head) within 60 degrees of
where it points.

One fix in the shared builder, which M25's sim takes too: a source is drawn 6 cm below the level-7 water beside it
(0.94 against 1.0 of its layer), so arrows ran into a pool of sources and every pool and falling column grew a 6 cm
sliver of froth. The flow now runs on the head (a source an eighth above level 7) and no fall is drawn for a step
under 7 cm; real steps (an eighth and up) keep their sheets. M25's Waterfall preset: 532 sheets before, 417 after.

## The owner's phone notes (2026-11-15): edge walls, sorting, the arrows' toggle, flow, and a panned texture

openspec/changes/water-flow design 7b and 7c. Headless Chromium on SwiftShader (no GPU in the cloud session): the look,
never the frame rate. Before is the branch at `26829d0`, after is this change, same camera, same settled preset.

| Before | After | Shows |
| --- | --- | --- |
| `flowtex-phone-default-before.png` | `flowtex-phone-default-after.png` | M28 at 390 × 844 (2x), the default view: the outer sea sheet is gone; Tenebris's water tile on the patch's sea. |
| `flowtex-front-edge-before.png` | `flowtex-front-edge-after.png` | "The very front part of the cell doesn't have any geometry": the near edge's walls, hidden under the outer sea before, show down to the base; the sea cells stand as a water wall at the edge. |
| `flowtex-sort-side-before.png`, `flowtex-sort-down-before.png` | `flowtex-sort-side-after.png`, `flowtex-sort-down-after.png` | The spring's falls from two low side views: before, sheets behind sheets blended through each other; after, only the nearest water draws, the tile panning down each fall. |
| `flowtex-phone-spring-default-before.png`, `flowtex-phone-spring-low-before.png`, `flowtex-phone-spring-night-before.png` | the same `-after` | The spring on a phone: default view, a low view across the falls, and 22:00 (each lamp lights the water and foam near it). |
| `flowtex-basin-arrows-before.png` | `flowtex-basin-arrows-after.png` | "we can see 1 arrow in a cell with 4 pads": the basin's pool, 6 arrows before, 18 after (every cap but the pool's still middle). |
| `flowtex-front-arrows-before.png` | `flowtex-front-arrows-after.png` | The spring 9 ticks in, still spreading: the front caps now point on into the dry plain. |
| | `flowtex-move-flow-a.png`, `flowtex-move-flow-b.png` | Two frames a second apart over flowing caps and falls (arrows off): the tile has moved downstream and down the falls; 10.8 % of pixels changed by more than 24 levels. |
| | `flowtex-move-still-a.png`, `flowtex-move-still-b.png` | The same over the still sea: 0.4 % changed (the waves' own shading). |
| `flowtex-m25-sim-before.png` | `flowtex-m25-sim-after.png`, `flowtex-m25-sim-arrowsoff-after.png`, `flowtex-m25-sim-night-after.png` | M25's Water sim (Waterfall, settled): its 1-layer front edge walled with water standing on it; "Arrows: on" and "Arrows: off"; at night. |
| `flowtex-m25-river-before.png` | `flowtex-m25-river-after.png` | M25's river at its waterfall: the fall's froth from the panned tile, the river's caps streaked along the flow. |
| `flowtex-m25-coast-before.png` | `flowtex-m25-coast-after.png` | M25's coast: the sea and its surf unchanged (the tile is off on the sea); the tide pools take the still tile. |

Checks (Playwright): M28 at 390 × 844, M25 #sim, #river and #coast: no console errors, one WebGL context, no horizontal
scroll at 390 px. Arrows: "Arrows: off" hides them and sets `aria-pressed` false, "Arrows: on" brings them back, on both
pages. Flow on caps (settled): spring 230 flowing caps, 229 with an arrow and the one without has nowhere to go; basin
12 of 12, plus 6 of its 7 sources (water leaves them); channel 20 of 20; pair 190 of 190. The arrows and the texture read
the same flow vector. Nothing moves on its own: camera and every object's rotation unchanged over 2 s idle. The liquid
rule is untouched.

## Luanti's liquid mesher (owner, 2026-11-15): sloped edges, walls, and falls block by block

openspec/changes/water-flow design 7e. The owner, on M28's ledge: "same issues as before for the leading edge, also
theres no flow maps on the edge as well and no walls, note that you missed a critical part of the luanti/minetest water
implementation that shows the water falling", then "water should generate blocks vertically as it falls too".
Headless Chromium on SwiftShader (no GPU in the cloud session): the look, never the frame rate. Before is the branch
before the mesher (`438d4f2`'s `water-look.js`), after is the mesher; same camera, same preset, same tick. `phone` is
390 × 844 (2x), `desk` 1400 × 900 (the stage only).

| Before | After | Shows |
| --- | --- | --- |
| `mesher-ledge-default-{phone,desk}-before.png` | `-after` | M28's spring settled, its own view (the owner's): before, each falling column is a flat cap at the lip a cell out from the cliff with one sheet to the plain; after, the sheet slopes to the edge and every falling cell is drawn, a wall of blocks down the cliff. |
| `mesher-ledge-edge-{phone,desk}-before.png` | `-after` | The ledge edge close, from the plain: the falls stack block by block (a thin band at each layer), foaming, the tile running down them, a chevron down each falling block and down each sloping top. |
| `mesher-ledge-edge-night-{phone,desk}-before.png` | `-after` | The same at 22:00: the lamps light the water and the foam near them. |
| `mesher-grow-t{0,2,4,6,8,10}-{phone,desk}-before.png` | `-after` | The spring tick by tick: t2 the sheet's front slopes to a wall; t4 the first falling cell over the cliff; t6, t8, t10 the column one layer longer each tick, block by block, until it reaches the plain. Before, the same ticks show a cap hanging at the lip. |
| `mesher-m25-sim-fall-{phone,desk}-before.png`, `-close-`, `-close-night-` | `-after` | M25's Water sim, Waterfall settled: every step falls as whole cells with sloped tops above them; Close; Close at night. |
| `mesher-m25-sim-fall-t2-…`, `mesher-m25-sim-fall-t6-…` | `-after` | M25's Waterfall 2 and 6 ticks in: the first falls growing a cell a tick. |

The rule (`water-flow/liquid.js`) is unchanged. It already made the falling cells: on the spring, the first falling
cell over the cliff is at tick 4, and the column grows a layer a tick to the plain by tick 10 (design 7e). Checks
(Playwright): every preset of both pages settles in the same ticks to the same counts as before (M28 spring 22 ticks,
580 sources, 380 flowing, 126 falling; channel 7, 580, 22, 1; basin 4, 586, 12, 0; pair 13, 581, 208, 12; runaway
19, 896, 38, 19; recede 53, 579, 0, 0; M25 pond 5, 7, 30, 0; waterfall 27, 1, 452, 182; dam 0, 78, 0, 0; flat 0);
M28 at 390 × 844 and 1400 × 900 and M25 #sim: no page errors (the only console line is the fonts request the harness
blocks), one WebGL context, no horizontal scroll; nothing moves in 2 s idle. On the settled spring, 253 of 254
flowing tops carry a flow vector and an arrow; the one without is the basin's still middle.
