# Hangar and Chibi Aircraft (M24): screenshots

The mockup is `docs/mockups/hangar.html`; its write-up is
`openspec/changes/vehicle-yards` (design decisions 1, 4 and 5).

**Rendered without a GPU.** These were taken in a Claude Code cloud container,
which has no GPU, with headless Chromium on SwiftShader (software rendering),
1280 x 800 at device scale 2, served by `python3 -m http.server` from
`docs/mockups`. They show what the page looks like, never how fast it runs. On
the owner's desktop the page is screenshotted on the GPU (CLAUDE.md, "Render on
the GPU when there is one").

Each shot is the page with `?shot=<view>&time=<day|night>` (the stage fills the
window and holds one picture; the rotors are stopped at fixed angles), or the
builder section with `?still&preset=<id>`.

| File | What it shows | Design |
| --- | --- | --- |
| `airfield-night.jpg` | The whole field at night: the three-bay hangar with its doors up and its lamps on, the runway's edge and threshold lights, the blue taxiway lights, the green pad lights round Bumble, the floodlight masts, the tower's beacon, the nav lights on every aircraft. The ground between the pools stays dark. | 4, the page 1; CLAUDE.md "Mockups are lit at night" |
| `airfield-day.jpg` | The same view by day: the hex ground of the game's atlas, the granite apron and taxiway, the runway with its markings and designators, the helipad, the windsock and the tower; Puffin, Dart and the game's Kestrel on the apron. | 4, the page 1 |
| `hangar-night.jpg` | Into the bays at night: Sharky and Gull inside under the bay lamps, the workbench where the builder opens, the light the open doors spill on the apron. | 4, "A hangar is required" |
| `bumble-pad-night.jpg` | Bumble on the helipad at night, ringed by its pad lights. | 4, the page 1 |
| `presets.jpg` | The five presets close up by day, one panel each: Sharky (prop fighter, shark mouth, nose number 106), Bumble (utility helicopter, glass nose, skids, tail rotor), Puffin (tiltrotor VTOL, beside the game's Kestrel), Gull (floatplane), Dart (little jet). The chibi proportions are the owner's question A1. | 4, "Chibi"; 5 |
| `builder-sharky.jpg` | The builder with Sharky loaded: the palette, the mirrored 13 x 11 grid of 1 m cells, the livery panel, the live 3D preview, the numbers that follow from the parts, the checks and the bill of materials. | 4, "The builder is M4's", "Numbers from the parts", "Checks" |
| `contact-sheet-art.png` | The new pixel art from `tools/art/hangar_art.py` beside four of the solitaire collection's reference sprites: the part icons, the livery decals (shark mouth, the three roundels, the digits) and the textures, each tiled 2 x 2 so a seam would show. | ART-DIRECTION, "The gate before a page is published" |
