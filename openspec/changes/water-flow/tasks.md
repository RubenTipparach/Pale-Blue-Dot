# Tasks

## 1. The mockup

- [x] 1.1 Write-up: proposal, design, spec delta. Verify: `openspec validate --all`.
- [ ] 1.2 `docs/mockups/water-flow.html`: Luanti's `transformLiquidsLocal` ported to the hex grid (design 1 to 4), with the LGPL notice. A hex patch with a hill, a ledge, a basin and the sea. Pour, dig and place tools, presets, and Run off by default. Lit at night. Verify: headless screenshots of each preset settled, by day and at night, in `docs/screenshots/mockup-water-flow/`; no page errors.
- [ ] 1.3 Settling is a function of sources and solids (design 6). Verify: in the page, "Settle from scratch" matches the incremental state cell for cell after each preset.
- [ ] 1.4 On the tracker as M26, and in `docs/mockups/mockup-work.md`. Verify: the tracker entry in the repo.
- [ ] 1.5 The owner approves the mockup on the tracker. Verify: the approval is read back and recorded.

## 2. The rule in `pbd-core` (after approval and a request)

- [ ] 2.1 The transform as a pure function of a cell and its eight neighbours, written from design (decision 8). Verify: unit tests for falling, spreading, receding, the renew threshold of three and a pentagon cell, each against the mockup's result for the same case.
- [ ] 2.2 The queue and the tick, bounded as Luanti's. Verify: a test that a spring on a slope settles to the mockup's field.

## 3. Saves, rendering, the game

- [ ] 3.1 Save format with sources as edits and the level for flowing cells; the old format readable. Verify: a save from before opens unchanged.
- [ ] 3.2 The water top reads the flowing level. Verify: the pipeline validates; a capture of a spring on a hill.
- [ ] 3.3 Frame cost on the desktop. Verify: `perf_suite.py`, before and after.
