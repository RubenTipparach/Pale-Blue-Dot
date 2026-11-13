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
builder section with `?still&preset=<id>`. Nothing on the page turns on its own; the rotors stand at fixed angles.

The shots taken before per-piece paint (`airfield-*.jpg` but `airfield-rolled-out.jpg`, `hangar-night.jpg`, `bumble-pad-night.jpg`, `presets.jpg`) show the presets without the little paint they now carry: Sharky's fin is red and Puffin's nacelles orange on the page.

| File | What it shows | Design |
| --- | --- | --- |
| `airfield-night.jpg` | The whole field at night: the three-bay hangar with its doors up and its lamps on, the runway's edge and threshold lights, the blue taxiway lights, the green pad lights round Bumble, the floodlight masts, the tower's beacon, the nav lights on every aircraft. The ground between the pools stays dark. | 4, the page 1; CLAUDE.md "Mockups are lit at night" |
| `airfield-day.jpg` | The same view by day: the hex ground of the game's atlas, the granite apron and taxiway, the runway with its markings and designators, the helipad, the windsock and the tower; Puffin, Dart and the game's Kestrel on the apron. | 4, the page 1 |
| `hangar-night.jpg` | Into the bays at night: Sharky and Gull inside under the bay lamps, the workbench where the builder opens, the light the open doors spill on the apron. | 4, "A hangar is required" |
| `bumble-pad-night.jpg` | Bumble on the helipad at night, ringed by its pad lights. | 4, the page 1 |
| `presets.jpg` | The five presets close up by day, one panel each: Sharky (prop fighter, shark mouth, nose number 106), Bumble (utility helicopter, glass nose, skids, tail rotor), Puffin (tiltrotor VTOL, beside the game's Kestrel), Gull (floatplane), Dart (little jet). The chibi proportions are the owner's question A1. | 4, "Chibi"; 5 |
| `builder-sharky.jpg` | The builder with Sharky loaded: the palette (its pieces in the livery's colours, the paint brush beside the eraser), the brush swatches, the mirrored 13 x 11 grid of 1 m cells with every piece drawn in its colour (teal base; sunny trim on the cowl, the tailplanes and the wing tips; the fin painted red, marked in the cell's corner), the livery panel, the 3D preview, the numbers, the checks and the bill of materials. Retaken for "the grid shows the paint". | 4, "The builder is M4's", "The grid shows the paint", "Numbers from the parts", "Checks" |
| `builder-bumble-paint.jpg` | Bumble given a paint job with the brush, mirror on: the cab's sides and two bands on the tail boom in sunny, the tail rotor, the tailplanes and the rotor's blade tips in red, on its green livery. The grid and the preview show the same colours. | 4, "The grid shows the paint" |
| `airfield-rolled-out.jpg` | That repainted Bumble rolled out on the apron in front of the middle bay, by day: the rolled-out craft wears each piece's paint. | 4, "The grid shows the paint"; the page 2, "Roll out" |
| `fly-bank-day.jpg` | "Fly it": Sharky in a banked turn over the field by day (`?shot=fly&fly=sharky&fscript=bank`), the ailerons deflected on the stick, the chase camera beside and behind it, and the flight instruments: airspeed, height, pitch and bank with the attitude ball, heading, throttle, the surfaces' deflections, the stall speed. A scripted autopilot flies the same stick a player has, for half a second from a set start. | steer-and-sail 2, 6 |
| `fly-approach-night.jpg` | Dart on short final for runway 09 at night (`fscript=approach`): its nav lights, the runway edge lights, the green threshold row and the field's lamps. | steer-and-sail 6; CLAUDE.md "Mockups are lit at night" |
| `builder-surfaces-refused.jpg` | The builder marking the control surfaces (A ailerons, the outer two wing cells each side; E elevator; R rudder; a cyan hinge in the preview), here with Sharky's tailplane erased: "No elevator: it cannot pitch", and "Fly it" refuses it. | steer-and-sail 2 |
| `contact-sheet-art-paints.png` | Every part icon in every livery paint, one row a part (cockpit to the brush's spray can), as `tools/art/hangar_art.py` draws them into `hangar/<part>-paints.png`: the painted panels in each paint's own ramp, all in the kit's palette. | 4, "The grid shows the paint" |
| `contact-sheet-art.png` | The new pixel art from `tools/art/hangar_art.py` beside four of the solitaire collection's reference sprites: the part icons, the livery decals (shark mouth, the three roundels, the digits) and the textures, each tiled 2 x 2 so a seam would show. | ART-DIRECTION, "The gate before a page is published" |
