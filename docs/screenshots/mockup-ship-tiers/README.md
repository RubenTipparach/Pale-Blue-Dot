# M21 Rockets and Landers: screenshots

The mockup is `docs/mockups/ship-tiers.html` (change `openspec/changes/ship-tiers`,
design sections 3 and 6). The owner, 2026-11-12: "Let's not do warp. Make me some
more rocket building presets. Allow larger rockets with more leg room to walk
around in. (3-4 radius? Blocks)". The page was "Rockets to Starships"; warp,
the starship and the star map are gone, and the assembly grid reaches four rings
out with decks inside.

**Rendered without a GPU.** These were taken in a Claude Code cloud container
with Playwright's Chromium on its software renderer (SwiftShader), at 1280 x 800
CSS pixels and device scale 2, with the page's `?still` flag, which renders until
the textures are in and a few frames more, then waits for input. The flight
shots use the page's screenshot flags: `launch=1` presses Launch, `hold=KeyW`
holds a key from lift-off, `freeze=<s>` stops the loop at that flight time, and
`cyaw`, `cpitch`, `cdist` place the camera. The flight steps 1/20 s or less a
frame, so a frozen shot is at the stated flight time, not wall-clock time. They show what
the page looks like, never how smoothly it runs; no frame time was measured. On
the owner's desktop the same views render on the GPU (CLAUDE.md, "Render on the
GPU when there is one"). Served with `python3 -m http.server 8764` from
`docs/mockups`.

| Shot | What it shows | Page URL flags |
| --- | --- | --- |
| `1-vab-day.png` | The Colony lander (ring 4, 752 cells, 25.5 m across) on the enlarged pad by day: a ring-3 disc of chemical engines and 96 solid booster cells, the nuclear lander stage on six legs, three decks walled in hull with portholes, the pod on the roof. The gantry stands off the lowest hatch with a gangway to it; the game's Kestrel is parked on the apron. | `?tab=vab&preset=colony&time=day&still` |
| `1-vab-night.png` | The same at night: pad rim lamps, floodlights and gantry lamps throw their own pools, the portholes glow with the decks' lamps, the field outside the pools is near-black, the moon dim. | `?tab=vab&preset=colony&time=night&still` |
| `1-vab-cutaway-night.png` | Cut-away at night: the walls on the camera's half are hidden, so the three decks, their cabins, the ladder up the core and each deck's lamps show. | `...&time=night&view=cut&still` |
| `1-vab-crew-day.png` | The Crew rocket, two decks (ring 2): chemical cluster and boosters, a decoupler ring, a nuclear upper stage, two decks and the pod; the gangway runs to its hatch. | `?tab=vab&preset=crew&time=day&still` |
| `1-vab-launch-night.png` | The crew rocket's flight at night, 2.4 s in, stick centred: the dampeners hold it upright, the boosters and core lit, the exhaust lighting the pad. | `?tab=vab&preset=crew&time=night&shot=1&launch=1&still`, taken 2.4 s into the flight |
| `1-vab-too-heavy.png` | The Too heavy preset: TWR 0.55, the one example that fails its checks by design. | `?tab=vab&preset=heavy&time=day&still` |
| `2-layer-editor.png` | The working panel under the stage, on the colony lander's layer 11: layer tabs (decks underlined green), the 61-cell disc in plan with the layer below ghosted and every cell a walker stands in outlined green, symmetry 1/2/3/6, the part palette with the six interior parts, the presets, the stack in section through the core with the walkers marked, the stage table and every check passing. | `?tab=vab&preset=colony&time=day&layer=11&still`, the `#work` panel |
| `3-inside-night.png` | Aboard: first person on the colony lander's lowest deck (L7) at night, beside the ladder shaft, looking across to the cabins and the portholes. Only that deck's lamps light it (the hull and the floors stand between it and every other lamp, and the moon is shut out). Drag to look; nothing moves until you do. | `...&time=night&view=inside&still` |
| `3-inside-day.png` | The same on the second deck (L10) by day, the ladder going down through the floor to the deck below. | `...&time=day&view=inside&stand=1&still` |
| `5-turn-rates-colony.png` | The assembly building's stage table for the Colony lander with the new Turn column: pitch (and yaw) turn rate at full gimbal, 10 deg/s2 for the lifter stage, 3.6 for the lander, with roll from engines off the axis. | `?tab=vab&preset=colony&still`, the stage table |
| `5-turn-rates-hop.png` | The same for the Small hop: 529 deg/s2 from one chemical engine on a 7 t, 3 m stack, and no roll (one engine on the axis; fins roll it only in air). | `?tab=vab&preset=hop&still`, the stage table |
| `6-gimbal-turn-day.png` | The flown launch preview by day: the crew rocket 1.9 s after lift-off with W held. The chemical nozzles are swivelled to their gimbal limit and their plumes follow, the boosters' fixed. The HUD gives the pitch, yaw and roll rates, the gimbal angles, throttle, altitude and speed; the keys are on screen. | `?tab=vab&preset=crew&time=day&launch=1&hold=KeyW&freeze=1.9&cyaw=1.45&cpitch=0.12&cdist=1.25&still` |
| `6-gimbal-turn-night.png` | The same on the ring-4 Colony lander at night, also 1.9 s with W held: tilted 17 degrees against the crew rocket's 35, pitching at 20 deg/s against its 40, as the moments of inertia say. | `?tab=vab&preset=colony&time=night&shot=1&launch=1&hold=KeyW&freeze=1.9&cyaw=1.45&cpitch=0.12&cdist=1.15&still` |
| `6-gimbal-hop-day.png` | The Small hop 0.45 s after lift-off with W held: already pitched over, at 170 deg/s. | `?tab=vab&preset=hop&time=day&launch=1&hold=KeyW&freeze=0.45&cyaw=1.45&cpitch=0.12&still` |
| `4-antimatter-lander-night.png` | Tab 4 at night: the Antimatter lander preset (four antimatter cores, a deck with three cabins and an airlock) landed on Cinder's basalt flats, its portholes lit, the landing ground's lamps in their own pools. | `?tab=antimatter&sub=lander&time=night&still` |
| `4-antimatter-course.png` | Tab 4's course: Pale Blue's surface to Cinder's, lift off, burn, flip, brake, set down, 5 min 10 s; the fusion tier's three-craft way dashed beside it. No warp, no star map. | `?tab=antimatter&sub=course&still` |

The part icons and textures (six new interior parts: crew deck, cabin, ladder,
hatch, airlock, hull wall; a fusion ship sprite replacing the starship) are drawn
by `tools/art/ship_tiers_art.py` with the balatro-pixel-art kit;
`python tools/art/ship_tiers_art.py --sheet OUT.png` sheets them beside the kit's
references.
