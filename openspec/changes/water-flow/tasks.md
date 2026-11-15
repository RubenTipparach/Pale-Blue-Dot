# Tasks

## 1. The mockup

- [x] 1.1 Write-up: proposal, design, spec delta. Verify: `openspec validate --all`.
- [x] 1.2 `docs/mockups/water-flow.html`: Luanti's `transformLiquidsLocal` ported to the hex grid (design 1 to 4), with the LGPL notice. A hex patch with a hill, a ledge, a basin and the sea. Pour, dig and place tools, presets, and Run off by default. Lit at night. Verify: headless screenshots of each preset settled, by day and at night, in `docs/screenshots/mockup-water-flow/`; no page errors.
- [x] 1.3 Settling is a function of sources and solids (design 6). Verify: in the page, "Settle from scratch" matches the incremental state cell for cell after each preset.
- [x] 1.4 On the tracker as M28, and in `docs/mockups/mockup-work.md`. Verify: the tracker entry in the repo.
- [x] 1.6 Sync the water's look with M25's Water sim (design 7a): frothy fall walls and lips on every step down, flow streaks along each cap, Flow arrows on by default, in `water-flow/water-look.js` shared by both pages; M28's descriptive text moved into a modal. Verify: every preset settles in the same ticks to the same counts as before; both pages open with no errors and one WebGL context; before and after screenshots (`sync-*.png`) in `docs/screenshots/mockup-water-flow/`.
- [x] 1.7 The owner's phone notes of 2026-11-15 (design 7b, 7c): the patch's edge cut with walls to a stone base in both pages; a depth-only pass so falls, caps and arrows sort; "Arrows: on / off" toggles; every flowing cap pointing the way its water goes; Tenebris's water tile panned along each cap's flow and down each fall in `water-look.js`. Verify: Playwright on SwiftShader (no GPU): M28 at 390 x 844, M25 #sim, #river and #coast open with no errors and one WebGL context; the front edge has walls; the falls sort from the default and a low side view; the toggle flips the arrows, its pressed state and label; two frames a second apart differ on flowing caps and not on still water; nothing moves in 2 s idle; no horizontal scroll at 390 px; `flowtex-*.png` before and after in `docs/screenshots/mockup-water-flow/`.
- [x] 1.8 Luanti's liquid mesher on the hex grid (design 7e), in `water-look.js` for both pages: per-cell corner levels, tops, sides, bottoms and falling blocks, the flow from the corner slope. The rule unchanged. Verify: every preset settles in the same ticks to the same counts; Playwright on SwiftShader: M28 at 390 x 844 and desktop and M25 #sim with no errors and one WebGL context; from the ledge edge the sheet slopes to walls and the falls stack block by block (t0, t2, t4, t6); lit at night; nothing moves in 2 s idle; `mesher-*.png` in `docs/screenshots/mockup-water-flow/`.
- [ ] 1.5 The owner approves the mockup on the tracker. Verify: the approval is read back and recorded.

## 2. The rule in `pbd-core` (after approval and a request)

- [ ] 2.1 The transform as a pure function of a cell and its eight neighbours, written from design (decision 8). Verify: unit tests for falling, spreading, receding, the renew threshold of three and a pentagon cell, each against the mockup's result for the same case.
- [ ] 2.2 The queue and the tick, bounded as Luanti's. Verify: a test that a spring on a slope settles to the mockup's field.

## 3. Saves, rendering, the game

- [ ] 3.1 Save format with sources as edits and the level for flowing cells; the old format readable. Verify: a save from before opens unchanged.
- [ ] 3.2 The water top reads the flowing level. Verify: the pipeline validates; a capture of a spring on a hill.
- [ ] 3.3 Frame cost on the desktop. Verify: `perf_suite.py`, before and after.
- [ ] 3.4 A flow-arrow overlay (requirement "A flow-arrow overlay"; owner, 2026-11-15: "the game should have this too"), as a debug key and a player setting, off by default. Write-up first; no code before the owner approves the mockup and asks. Verify: a capture of a spring with the overlay on and off.
- [ ] 3.5 The water top pans its texture along the baked flow vector, and a falling face pans down (requirement "Flowing water pans its texture along its flow"; design 7c, Tenebris C's method). Write-up first, as 3.4. Verify: two captures a second apart differ on flowing water and not on still; the pipeline validates.
- [ ] 3.6 The game's water mesher draws every water cell from its corner levels, as Luanti's `drawLiquidNode` (requirement "Flowing water is meshed per cell from its corner levels"; design 7e): sloped tops, walls where water meets air, falling cells as whole blocks, bottoms over air, the flow from the corner slope. Write-up first, as 3.4. Verify: captures of a spring falling over a ledge, at three ticks as the column grows; the pipeline validates.
