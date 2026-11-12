# Cranes and Printers (M22), screenshots

The mockup `docs/mockups/construction.html`, answering the owner's note on Forge and Fabricator
(M3): "Would like to see what more advanced fabricators look like for building large items, cranes
and 3d printers for assembling buildings". The write-up is `openspec/changes/vehicle-yards`
(design decisions 1, 2 and 5).

**Rendered without a GPU.** These were taken in a Claude Code cloud container, which has no GPU:
headless Chromium (Playwright) on SwiftShader, at 1280 x 800 and a device scale of 2, served by
`python3 -m http.server` from `docs/mockups`. They show what the page looks like, never how fast it
runs. On the owner's desktop the same shots are taken on the GPU (CLAUDE.md, "Render on the GPU").

| File | URL (relative to `docs/mockups/`) | What it shows |
| --- | --- | --- |
| `yard-day.jpg` | `construction.html?shot=all&time=day` | The whole yard by day: the printer mid-print (pass 6 of 8), the tower crane on its four stone cells with a wall panel on the hook, the assembly bay, the laydown, the site office, all on the game's hex ground. |
| `yard-night.jpg` | `construction.html?shot=all&time=night` | The same at night (the default). Floodlights, lanterns, the bay's and the office's windows, the print head and the crane's red lights each light what is near them; the apron between the pools and the hills beyond stay dark. |
| `printer-night.jpg` | `construction.html?shot=printer&time=night` | The hex printer close up at night: the teal portal gantry on its rails, the head tracing a wall line of the second storey, the fresh layer darker (wet) than the ones below, the door and window gaps, the bay's lit interior behind. |
| `crane-night.jpg` | `construction.html?shot=crane&time=night` | The tower crane side on at night: lattice mast and jib, counter-jib and counterweights, cat head and pendants, red obstruction lights at the top, the jib tip and the counter-jib's end, the trolley and hoist line. |
| `crane-top-night.jpg` | `construction.html?shot=top&time=night` | The crane's top: the cab with its lit windows, the slewing unit, the cat head lit by its red light. |
| `controls-refused.jpg` | `construction.html?load=engine&trolley=30` | The stage with its side panel: the print slider and readout, the crane's cab controls, and the load chart refusing the 6 t engine past 20 m (the chart allows 4.0 t at the 30 m asked for). |
| `assembly-bay.jpg` | `construction.html` (section 2) | The assembly bay's large recipes (mass, footprint in cells, time, power, inputs, how far the tower crane may take each) and its queue. |
| `pass-plan.jpg` | `construction.html` (section 3) | One pass from above (floor and storey 2, layer 1): the walls on cell edges, the six floor cells and the open stairwell, the head's order and its travel moves, the time and printstone it takes. |

`?shot=<view>` fills the window with the stage and hides its controls; `?time=day|dusk|night`,
`?print=0..800`, `?load=`, `?slew=`, `?trolley=`, `?hoist=`, `?yaw=`, `?pitch=`, `?dist=` and
`?pr=` (pixel ratio) make a shot repeatable.
