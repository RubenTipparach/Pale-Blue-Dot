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
the textures are in and a few frames more, then waits for input. They show what
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
| `1-vab-launch-night.png` | The crew rocket's launch preview at night, a few seconds in: the boosters and core lit, the exhaust lighting the pad. | `?tab=vab&preset=crew&time=night&launch=1&still` |
| `1-vab-too-heavy.png` | The Too heavy preset: TWR 0.55, the one example that fails its checks by design. | `?tab=vab&preset=heavy&time=day&still` |
| `2-layer-editor.png` | The working panel under the stage, on the colony lander's layer 11: layer tabs (decks underlined green), the 61-cell disc in plan with the layer below ghosted and every cell a walker stands in outlined green, symmetry 1/2/3/6, the part palette with the six interior parts, the presets, the stack in section through the core with the walkers marked, the stage table and every check passing. | `?tab=vab&preset=colony&time=day&layer=11&still`, the `#work` panel |
| `3-inside-night.png` | Aboard: first person on the colony lander's lowest deck (L7) at night, beside the ladder shaft, looking across to the cabins and the portholes. Only that deck's lamps light it (the hull and the floors stand between it and every other lamp, and the moon is shut out). Drag to look; nothing moves until you do. | `...&time=night&view=inside&still` |
| `3-inside-day.png` | The same on the second deck (L10) by day, the ladder going down through the floor to the deck below. | `...&time=day&view=inside&stand=1&still` |
| `4-antimatter-lander-night.png` | Tab 4 at night: the Antimatter lander preset (four antimatter cores, a deck with three cabins and an airlock) landed on Cinder's basalt flats, its portholes lit, the landing ground's lamps in their own pools. | `?tab=antimatter&sub=lander&time=night&still` |
| `4-antimatter-course.png` | Tab 4's course: Pale Blue's surface to Cinder's, lift off, burn, flip, brake, set down, 5 min 10 s; the fusion tier's three-craft way dashed beside it. No warp, no star map. | `?tab=antimatter&sub=course&still` |

The part icons and textures (six new interior parts: crew deck, cabin, ladder,
hatch, airlock, hull wall; a fusion ship sprite replacing the starship) are drawn
by `tools/art/ship_tiers_art.py` with the balatro-pixel-art kit;
`python tools/art/ship_tiers_art.py --sheet OUT.png` sheets them beside the kit's
references.
