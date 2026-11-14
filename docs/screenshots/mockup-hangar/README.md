# Hangar and Chibi Aircraft (M24): screenshots

The mockup is `docs/mockups/hangar.html`; its write-up is
`openspec/changes/vehicle-yards` (design decisions 1, 4 and 5).

**Rendered without a GPU.** These were taken in a Claude Code cloud container,
which has no GPU, with headless Chromium on SwiftShader (software rendering),
1280 x 800 at device scale 2, served by `python3 -m http.server` from
`docs/mockups`. They show what the page looks like, never how fast it runs. On
the owner's desktop the page is screenshotted on the GPU (CLAUDE.md, "Render on
the GPU when there is one").

The shots taken before the country round the field (decision 2a) show the field's edge falling away to a flat plain; the `fly-*`, `airship-flying-day` and `town-*` shots have the hills, the coast and the towns.

**The tabs (2026-11-13).** The page is now in four tabs, so only one 3D view is
ever on the page (the owner: "make sure to apply the "tabs" idea, so we always
only ever see 1 3d renderer on page at a time"): `#airfield` (the airfield and
Fly it), `#planes` (the plane builder), `#airship` (the airship builder) and
`#how` (how the numbers follow, the parts, the rules, saves and decisions). The
`tab-<name>.jpg` shots are each tab opened from its `#anchor` with `?still`, at
1280 x 800 and **device scale 1**, again SwiftShader shots from a cloud
container with no GPU: they show the layout, never the speed. Every other shot
below predates the tabs and shows the old one-long-page layout.

| File | What it shows |
| --- | --- |
| `tab-airfield.jpg` | `#airfield`: the night airfield filling the window under the tab bar, its view, time and Fly it / Expert controls at the top right, the info box and hotbar at the bottom. |
| `tab-planes.jpg` | `#planes`: the plane builder in three columns, all in one screen: examples, parts, mirror, brush and Roll out / Fly it / Clear on the left; the 13 x 11 grid and the livery in the middle; the 3D preview, the numbers and the checks on the right (the numbers and checks scroll inside their column). |
| `tab-airship.jpg` | `#airship`: the airship builder the same way: parts and actions, the 18 x 7 grid of 2 m cells with livery and gas, the preview and its numbers. |
| `tab-how.jpg` | `#how`: the reading tab, no 3D view: the table of how the numbers follow from the parts, then the parts (planes and airships), the rules, saves and the decisions. |

The `?shot=` and `?still&preset=` links below still work: `?shot` opens the
airfield tab, and a builder shot now needs its anchor (`?still&preset=bumble#planes`,
`?still&apreset=albatross#airship`; the old `?tab=airship` still opens the
airship tab).

Each shot is the page with `?shot=<view>&time=<day|night>` (the stage fills the
window and holds one picture; the rotors are stopped at fixed angles), or the
builder section with `?still&preset=<id>`. Nothing on the page turns on its own; the rotors stand at fixed angles.

The shots taken before per-piece paint (`airfield-*.jpg` but `airfield-rolled-out.jpg`, `hangar-night.jpg`, `bumble-pad-night.jpg`, `presets.jpg`) show the presets without the little paint they now carry: Sharky's fin is red and Puffin's nacelles orange on the page. Every shot but the `fly-*`, `airship-*` and `town-*` ones also predates decision 2c's bigger wings (a wing cell is 3 m&sup2;; the shapes did not change) and Puffin's new shape: its wing three cells deep and a second fuel tank in its nose, which balances it on that wing.

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
| `fly-bank-day.jpg` | "Fly it": Sharky in a banked turn over the field by day (`?shot=fly&fly=sharky&fscript=bank`), the ailerons deflected on the stick, the chase camera beside and behind it, the country running on past the field, and the flight instruments: airspeed, height, pitch and bank with the attitude ball, heading, throttle, the surfaces' deflections, the stall speed. A scripted autopilot flies the same stick a player has, for half a second from a set start. Retaken with the country round the field. | steer-and-sail 2, 6 |
| `fly-approach-night.jpg` | Dart on short final for runway 09 at night (`fscript=approach`): its nav lights, the runway edge lights, the green threshold row and the field's lamps, the dark country beyond. Retaken with the country round the field. | steer-and-sail 6; decision 2a; CLAUDE.md "Mockups are lit at night" |
| `builder-surfaces-refused.jpg` | The builder marking the control surfaces (A ailerons, the outer two wing cells each side; E elevator; R rudder; a cyan hinge in the preview), here with Sharky's tailplane erased: "No elevator: it cannot pitch", and "Fly it" refuses it. | steer-and-sail 2 |
| `airship-moored-night.jpg` | The airships at night (`?shot=mast&time=night`): Albatross riding at the big mast, nose in the cup, its gondola windows lit, the mast's red obstruction light; the airship shed (the taller bay) with its lamps and lit windows; Puff at the small mast by the shed. Nothing moves: a moored airship would swing with the wind, and only does when flown. | steer-and-sail 3b |
| `airship-flying-day.jpg` | Albatross flying over the field by day (`?shot=fly&fly=albatross`), crabbing in the wind, the rudder and elevators deflected, the props turning, and its instruments: height, free lift, ballast, gas, airspeed, ground speed, heading against track. Retaken with the country round the field. | steer-and-sail 3b |
| `airship-builder.jpg` | The Airship tab: 2 m cells, the airship parts (gas cell, gondola, engine pod, fin, ballast tank, mooring cone) drawn in their paint, R and E on the fins, the gas choice, the numbers (gas, gross lift, weight, free lift, ballast, pressure height, where it floats), the checks, and the preview with the hinges marked. | steer-and-sail 3b |
| `contact-sheet-art-paints.png` | Every part icon in every livery paint, one row a part (cockpit to the brush's spray can), as `tools/art/hangar_art.py` draws them into `hangar/<part>-paints.png`: the painted panels in each paint's own ramp, all in the kit's palette. | 4, "The grid shows the paint" |
| `contact-sheet-art.png` | The new pixel art from `tools/art/hangar_art.py` beside four of the solitaire collection's reference sprites: the part icons, the livery decals (shark mouth, the three roundels, the digits) and the textures, each tiled 2 x 2 so a seam would show. | ART-DIRECTION, "The gate before a page is published" |
| `fly-takeoff-day.jpg` | Sharky just off runway 09 by day (`fscript=takeoff`): flown by the simulation itself with Shift held 3.5 s from the threshold, the assists on (it rotated by itself at flying speed), the country beyond the field: hex hills in whole 1 m layers rising past the field's edge. | decision 2a, "Room to fly", "Forgiving flight" |
| `fly-hills-day.jpg` | Sharky turning left over the hills 500 m east of the field (`fscript=hills`), A held 2.2 s, the bank held at the assist's 25&deg; for its speed. Past 200 m the ground is drawn in coarser hex columns (10 cells across, then 80), their faces the atlas's own tiles at the near cells' scale; the coast and the sea to the north. | decision 2a, "Room to fly" |
| `town-wendle-night.jpg` | Wendle at night (`?shot=wendle&time=night`), 2.3 km east of the field, about 70 s at Sharky's cruise: stone and plank houses under slate along two streets, every window lit, a lamp post every 26 m and one in the square, each lighting what is near it. | decision 2a, "Room to fly"; CLAUDE.md "Mockups are lit at night" |
| `town-saltby-night.jpg` | Saltby at night (`?shot=saltby`), on the coast 2 km west-north-west: the same streets and lamps, and the quay out into the sea with its light. | decision 2a, "Room to fly"; CLAUDE.md "Mockups are lit at night" |
| `flight-test.txt` | The output of the player-like Playwright test `tools/mockups/hangar_flight_test.js`: (since 2026-11-13 also section 8, the flight effects) each preset's stall speed and take-off roll, a 60 s circuit in Sharky on W, Shift, A and D only, and the crash checks (the hangar wall, a dive into the hills, a house in Wendle, a parked craft, an airship into the shed), each with the count of steps where any point of the craft was inside a solid or under the ground (all 0). | decision 2a, 2c |

**Flight effects (2026-11-13).** The owner, on Sharky low over the hills: "some air flow lines when flying and wing tip lines would help sell the flight, it feels pretty good so far". The flight model is unchanged; the effects only read it. Airflow lines are short streaks fixed in the air, round the airframe and round the camera, so they stream past along the relative wind (against the velocity, not the nose), more and longer with airspeed and none below about 0.85 of the stall speed (10 m/s for rotorcraft). A trail leaves each wingtip (and each tailplane tip where the tailplane is clearly narrower than the wing), found from the built parts' widest points, so a design from the builder gets its own; it is faint in cruise and grows bright and wide with the load factor and the wing's angle of attack, fading over 1.2 s. A prop leaves a faint swirl with power on; a helicopter or Puffin in a low hover blows dust off the ground; an airship has none. They are lit like everything else (the sky's light, and every lamp near them, the craft's own nav lights included), drawn in four flat steps of strength, and the **Effects** button above the view (or the box in **Controls**) turns them off. Shots at 1280 x 720, **device scale 1**, `?shot=fly&fly=<id>&fscript=<cruise|turn|lowturn|hover>&time=<day|night>`, SwiftShader in a cloud container with no GPU: they show the look, never the speed.

| File | What it shows | Design |
| --- | --- | --- |
| `flow-cruise-day.png` | Sharky straight and level at 39 m/s by day (`fscript=cruise`): a few airflow lines streaming past the airframe and the camera, faint trails off the wingtips. | the owner, 2026-11-13 |
| `flow-turn-day.png` | Sharky in a hard banked turn over the hills (`fscript=turn`: rolled in on A, then S held 1.4 s at full power, a player's keys): bright, wide wingtip trails curving round behind it, airflow lines along the turn. | the owner, 2026-11-13 |
| `flow-turn-night.png` | The same pull at night, low over the field (`fscript=lowturn`): the trails and lines lit by the apron's floods and lamps and the plane's own red and green nav lights, the runway lights and the hangar beyond. | the owner, 2026-11-13; CLAUDE.md "Mockups are lit at night" |
| `flow-hover-night.png` | Bumble lifted into a low hover off its pad at night (`fscript=hover`, Shift held 2.4 s): rotor downwash blowing dust off the ground, lit by the pad lights. | the owner, 2026-11-13 |
