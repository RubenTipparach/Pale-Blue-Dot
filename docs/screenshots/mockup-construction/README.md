# Cranes and Printers (M22), screenshots

The mockup `docs/mockups/construction.html`. It answers the owner's note on Forge and Fabricator
(M3), "Would like to see what more advanced fabricators look like for building large items, cranes
and 3d printers for assembling buildings", and their review of this page's first version: "you
should have medeival cranes/scaffolds for building older style houses too, those metal cranes are
for building new homes, and 3d printer is kind of for smaller homes and general assembly of like
planes and parts and stuff." The write-up is `openspec/changes/vehicle-yards` (design decision 2).

**Rendered without a GPU.** Taken in a Claude Code cloud container with no GPU: headless Chromium
(Playwright) on SwiftShader, 1280 x 800 at a device scale of 2, served by `python3 -m http.server`
from `docs/mockups`. They show the look, never the frame rate. The panel and section shots use
`?pr=1` (the 3D canvas at 1x, the page at 2x) because a 2x canvas beside the side panel timed out
on the software renderer. On the owner's desktop the same shots are taken on the GPU.

| File | URL (relative to `docs/mockups/`) | What it shows |
| --- | --- | --- |
| `yard-day.jpg` | `construction.html?shot=all&time=day` | The whole yard by day, west to east: the old site on its dirt plot off the apron, the printed site with the assembly bay and parts bed behind it, the new site under the tower crane, the laydown and the office. |
| `yard-night.jpg` | `construction.html?shot=all&time=night` | The same at night (the default): the old site lit only by its brazier and lanterns, floodlights and work lamps on the printed and new sites, the bay's and office's windows, the crane's red lights; dark ground between the pools. |
| `old-night.jpg` | `construction.html?shot=old&time=night` | The old way: a half-timbered house of the towns' kit (stone ground storey, half-timbered upper storey, trusses up) inside a timber scaffold (poles at cell corners, plank decks on layer lines, ladders, rope lashings), the treadwheel crane behind holding the ridge beam over the trusses, the brazier, timber and ashlar waiting. |
| `treadwheel-night.jpg` | `construction.html?shot=wheel&time=night` | The treadwheel crane from behind the house: post on crossed sills with braces, the luffing jib, the rope from the wheel's axle, the walk-in wheel with its walker. |
| `printed-night.jpg` | `construction.html?shot=printed&time=night` | The hex printer printing a small home (three cells, one storey) in its third pass, the fresh layer darker; the parts bed and the bay behind. |
| `parts-bed-night.jpg` | `construction.html?shot=parts&time=night` | The parts printer bed: a small gantry over a flat bed printing a wing panel and a propeller's second blade; a finished panel and propeller on a cart at the bay door. |
| `new-night.jpg` | `construction.html?shot=neu&time=night` | The new way: a three-storey house of precast panels on cell edges, lit work windows, the tower crane lowering a panel into the gap in the third storey. |
| `crane-night.jpg` | `construction.html?shot=crane&time=night` | The steel tower crane side on at night: lattice mast and jib, counter-jib, cat head and pendants, red obstruction lights. |
| `controls-treadwheel.jpg` | `construction.html?crane=wheel&load=ashlar&trolley=6.5&pr=1` | The side panel on the treadwheel: its chart (6 t m, 1.5 t hook) refusing the 1.2 t ashlar block past 5.0 m, with Walk the wheel. |
| `controls-tower.jpg` | `construction.html?crane=tower&load=engine&trolley=30&pr=1` | The side panel on the tower crane: its chart (120 t m, 8 t hook) refusing the 6 t engine past 20 m. |
| `assembly-bay.jpg` | `construction.html?pr=1` (section 2) | The bay's large recipes, with printed wing panels and propellers among the inputs, and its queue. |
| `pass-plan.jpg` | `construction.html?pr=1` (section 3) | One pass of the small home from above (layer 2): walls on cell edges, the door and window gaps, the head's order and travel. |

Nothing moves until the viewer moves it (CLAUDE.md, "Nothing in a mockup spins on its own"): the
camera turns only when dragged; Play runs the printers and Walk the wheel the treadwheel, both off by
default. `?shot=<view>` fills the window with the stage; `?time=`, `?print=0..400`, `?crane=wheel|tower`,
`?load=`, `?slew=`, `?trolley=`, `?hoist=`, `?yaw=`, `?pitch=`, `?dist=` and `?pr=` make a shot repeatable.
