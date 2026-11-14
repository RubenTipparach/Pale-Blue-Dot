# Proposal: water that flows (Luanti's liquid rule on the hex grid)

## Why

The owner, 2026-11-14: "do we have flow algorithms for water right now? I
only see water at a single elevation, can you bring in code from Luanti
(formerly Minetest) water implementation, and adapt it for our hexgrid in a
water flow demo mockup?"

**Finding: there is no flow today.** Water is one static `Material::Water`
cell.
- The generator fills every cell from the ground up to sea level with it
  (`pbd-core/src/terrain.rs`, `voxel.layer <= self.sea_level`). That includes
  the river channels, whose water is the sea's (`ocean-environments` design).
- Nothing ever moves a water cell. `Column::water_surface` says so in its
  comment: "Nothing fills a pool yet, so today every water run in a generated
  column runs from the ground to sea level".
- So all water stands at one elevation. A dug hole beside the sea stays dry,
  and water cannot be poured anywhere.

## What changes

- **The rule.** Luanti's liquid transform (`ServerMap::transformLiquidsLocal`)
  is ported to the hex grid. A cell is a source or a flowing cell with a
  level from 7 down to 0. Flowing water falls first, spreads up to 7 cells
  from a source, and recedes when its source goes. Two adjacent sources
  (three on the hex grid, design 2) renew a source between them.
- **A water-flow mockup**, `docs/mockups/water-flow.html`:
  - a hex patch with a hill, a ledge, a basin and the sea;
  - the ported rule running at Luanti's one tick a second, off until Run;
  - tools to pour a source, dig and place stone;
  - presets that show a spring, a waterfall, a channel dug from the sea, a
    basin that renews, and the run-away the hex grid causes with Luanti's
    own threshold.
- **The plan for the game** (design 5 to 7), not built: which cells are
  saved, how old worlds stay untouched, and the queue and tick.

## Licence

Luanti's engine is LGPL-2.1-or-later (`src/servermap.cpp`, SPDX header). The
mockup's port is a derivative of that file. It says so at the top, and keeps
the copyright line and the licence name. The pinned commit is
`a10dc8dbff3b0af8295ae55df44f65b1d9ee92c7` (2026-11-09).

For the game, the recommendation taken (ask only with screenshots) is design
decision 8: the Rust implementation is written from this change's rule
description and the mockup's behaviour as the reference, not translated line
by line from the C++. That keeps the game's licence the owner's call. The
mockup stays the LGPL port.

## Impact

- New: `docs/mockups/water-flow.html` (mockup M28), on the tracker.
- No game code. No change to generation, so no generator version, and no
  save format change until task 3.
