# tenebris-towns: the owner's walk-through notes

Before and after shots for `openspec/changes/tenebris-towns`, design section
12: the eight notes the owner sent on 2026-09-29. The owner's own screenshots
are in `docs/handoff/2026-09-29-owner-notes/`.

The shots are `docs/mockups/towns.html` in headless Chromium, with no GPU
(SwiftShader), at 900 x 600. They are taken at 11:00, except the night
window shot at 22:30. `tools/mockup_towns_notes_shots.sh <dir> [page]`
takes every view. "Before" is the page before the fixes; "after" is the
page as committed. The numbers are from `tools/mockup_towns_checks.js` and
are in the design.

| note | files | what it shows |
| --- | --- | --- |
| 1. Water inside the boats | `1-boats-before/after.jpg` | The harbour's moored boats from above. Before, the water sheet ran across each hull's inside. After, each hull masks the water inside its waterline, and its planks show dry. |
| 2. Holes in the stairs | `2-steps-before/after.jpg` | The harbour's up-street steps. Before, a step's top faced down and left a stripe through to the water under the map. After, no hole. |
| 3. Windows | `3-windows-day-before/after.jpg`, `3-windows-night-before/after.jpg` | The village's fieldstone house from the lane. Before, open holes by day and glowing panes by night. After, see-through glass, day and night, with no glow. |
| 4. Chimney bottoms | `4-chimney-before/after.jpg` | Looking up at the top floor's ceiling. Before, the chimney's bottom flickered on the soffit. After, clean. |
| 5. Doors into furniture | `5-door-before/after.jpg` | Inside the great smial. Before, the round door swung in over the table. After, it opens outward into the garden. |
| 6. Jungle bridge posts | `6-bridge-before/after.jpg` | A rope bridge meeting a platform. Before, its rails ended 17 cm short of the platform's corners. After, they end on the corners. |
| 7. Stairwell light | `7-flight-before/after.jpg`, `7-newel-before/after.jpg` | A straight flight and a newel stair in the village. After, each has a candle sconce burning day and night: the flight's is on its right wall. |
| 8. The igloo | `8-igloo-before/after.jpg` | Before, slab walls with a vault laid on top, open ends, and a jagged cut in the dome. After, one arched tunnel of snow blocks, joined to the dome and closed at the mouth. |
