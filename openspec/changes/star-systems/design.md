# Design: Aubade, its neighbours, and Tenebris's worlds

Every decision below is a recommendation taken (ask only with screenshots)
unless it says otherwise; the owner can overturn any of them on the tracker.
The decisions table on planets.html (S1 to S8) carries the same list.

## 1. The Tenebris survey

Read from the owner's Tenebris repository, cloned read-only at commit
`ef98651`: the Rust port (`tenebris-rs/crates/tenebris-core/src/planet.rs`,
`planet_gen.rs`, `world.rs`, `fauna.rs`, `blocks.rs`, `orbit.rs`;
`tenebris-client/src/renderer.rs`, `animals.rs`; `assets/config/atmosphere.yaml`;
`SEQUOIA_PLAN.md`, `SPACE_ERA_PLAN.md`) and the TypeScript original
(`src/config/SolarSystemConfig.ts`, `PLAN_SEQUOIA.md`).

The Rust port defines twenty bodies (`BodyId`), parked for physics and on
circular rails for display (`orbit.rs`):

| Body | Tenebris radius, gravity | Its level and cell in Tenebris | What it is |
| --- | --- | --- | --- |
| Tenebris | 300 m, 1.0 g | L7, 2.833 m (the gold standard) | Earthlike main planet: Fields (yellow-green DryGrass pasture), Desert (sand, red CragRock outcrops), Jungle, Swamp, Mountains, Tundra; blue sea. Life: broadleaf trees (4-block trunk, two-layer crown), tundra pines, cactus; critter, stalker (night, hostile), cat, sheep, chicken, cow, bird, frog, lizard, penguin; fish, large fish, eel, ray, turtle. Orbit 30 km, 1 year. |
| Luna | 80 m, 0.4 g | L7, 0.76 m | Barren grey moon of Tenebris, 3 km out. |
| Selene | 50 m, 0.25 g | L6, 0.94 m | Barren moon of Tenebris, 5 km out. |
| Sequoia | 360 m, 1.1 g | L7, 3.40 m | The green sister: an archipelago (raised sea, more rivers, a lower island gate), thick dusty-brown air (red-heavy scattering), green water (#2a6655), small polar caps, snow only high, brown cloud decks. Biomes as built: redwood groves on tan sod ("Sequoia doesnt have green grass"), a fungal forest of pale mycelium with giant mushrooms, a dense alien jungle on tan sod, a dry wasteland where nothing grows, tundra, slate mountains. Trees as built (`world.rs`): redwoods 10 to 13 blocks, the lower two thirds bare, a rust-red crown; giant mushrooms, a stem and three layers of rose cap. Fauna as built (`fauna.rs`, a roster with no overlap onto Tenebris's): sparkit and shellkin (mascot bipeds), crane (wader), raptor (high ground), the sea serpent (in Sequoia's water only). Planned, not built (`SEQUOIA_PLAN.md`): gloomhopper, sporeling, barkstrider, fernwhistle, canopyclaw, mirefoot, duskstalker, mossback, silthog, slatehorn; redwoods 12 to 24 blocks with a two-block trunk, glow-vines, buttress roots, banyans. Orbit 40 km, 1.5 years. |
| Verdant | 60 m, 0.3 g | L6, 1.13 m | Barren moon of Sequoia (moon rock, despite the name). |
| Sprout | 36 m, 0.15 g | L6, 0.68 m | Barren moon of Sequoia, the smallest. |
| Crag | 190 m, 0.6 g | L7, 1.79 m | Barren rusty-red planet (moon.png tinted 1.30, 0.55, 0.35). Orbit 22 km. |
| Quartz | 44 m, 0.2 g | L6, 0.83 m | Barren moon of Crag. |
| Frostbite | 220 m, 0.8 g | L7, 2.08 m | Icy outer planet: snow over glacial ice (ice.png tinted 0.55, 0.80, 1.25). Orbit 55 km. |
| Rime | 40 m, 0.15 g | L6, 0.76 m | Icy moon of Frostbite. |
| Asteroids A to J | 10 to 22 m, 0.05 g | L4 to L5 | Dim carbonaceous rocks (rocks.png tinted 0.70, 0.68, 0.65), 34 to 38 km out. |

Levels are Tenebris's own rule (`planet::subdivisions_for_radius`,
`ceil(log2(1.05 R))` held to 3..7). Only the main planet held 2.833 m; the rest
ran from 0.68 m to 3.40 m, the bug CLAUDE.md's "Hex size is fixed" rule exists
to prevent. Life (`fauna::body_has_fauna`) is only on Tenebris and Sequoia;
barren and icy bodies have no atmosphere, which matches CLAUDE.md's "airless
bodies are lifeless". The TypeScript original had Earth (100 m), its Moon
(50 m), Sequoia (200 m, "2x Earth's radius", reddish air, greenish water,
mushroom trees) and a Sun; the Rust port supersedes it.

Textures: all real PNGs (16 x 16, the animal atlas 768 x 32); none was a Git
LFS stub, so no stand-ins were drawn. Copied byte-identical with a README.

## 2. Onto the ladder

Rule: **eight times the Tenebris radius, down to the rung at or below it,
never under 300 m**, with one exception: Sequoia goes one rung up, because
"the bigger sister" is its identity (360 m against 300 in Rust; twice Earth in
TypeScript). Tenebris itself lands on 2,400 m (L10); every other body keeps
its order. Gravity is Tenebris's.

| Body | Tenebris | 8 x R | Here |
| --- | ---: | ---: | --- |
| Sequoia | 360 m | 2,880 m | 4,800 m, L11 (one rung up) |
| Tenebris | 300 m | 2,400 m | 2,400 m, L10 |
| Frostbite | 220 m | 1,760 m | 1,200 m, L9 |
| Crag | 190 m | 1,520 m | 1,200 m, L9 |
| Luna | 80 m | 640 m | 600 m, L8 |
| Verdant | 60 m | 480 m | 300 m, L7 |
| Selene | 50 m | 400 m | 300 m, L7 |
| Quartz | 44 m | 352 m | 300 m, L7 |
| Rime | 40 m | 320 m | 300 m, L7 |
| Sprout | 36 m | 288 m | 300 m, L7 (the floor) |
| Asteroids A to J | 10 to 22 m | 80 to 176 m | 300 m, L7 each (the floor) |

Alternatives rejected: keeping each body's Tenebris cell count (almost every
body becomes 300 m and the size order is lost); the nearest rung to the radius
itself (everything collapses onto 300 m too).

Orbits keep Tenebris's order and ratios, ten times further out in km (Crag
220, Tenebris 300, the belt 340 to 380, Sequoia 400, Frostbite 550); moon
distances scale by eight like the radii; years and moon periods are
Tenebris's `orbit_years` times the 100-day year; spins are its `spin_days`
times the 48-minute day.

## 3. Where they live, and the new systems

Tenebris's worlds stay together as one family round one star, **Nocturne**
(S2), rather than being spread through Aubade, so Sequoia keeps its moons and
its sister. Tenebris's "Sun" never had a name.

| System | Star | Light | Living zone | Bodies | Why go |
| --- | --- | ---: | --- | --- | --- |
| Aubade (home) | G2 V yellow | 1.0 | 250 to 400 km | the twelve of M1 | home |
| Nocturne, 4.6 ly | G6 V warm yellow | 0.85 | 230 to 370 km | 11 (above) | Sequoia's redwoods and bipeds; the gold-standard world |
| Tallow, 6.3 ly | M2 V red dwarf, flares | 0.03 | 45 to 70 km | Sconce (1,200 m, L9, airless iron, 30 km, locked), Wick (2,400 m, L10, thin cold air, 56 km, locked; twilight-ring life: blackleaf trees, lanternbloom, tallowback, dimwing), Censer (gas giant, 9,600 m cloud tops, no cells, 160 km), Snuff (600 m, L8, airless sooty ice, moon of Censer) | blackwood, emberglass, a twilight world |
| Gemel, 8.1 ly | K2 V orange plus a white dwarf (Gemel B, 420 km out) | 0.35 | 145 to 230 km | Shale (1,200 m, L9, airless slate, 90 km), Fallow (2,400 m, L10, cold dry Earthlike air, 175 km; steppe grass, stone pines, drover, scythewing, burrowkit; no townsfolk), Lode (300 m, L7, metal asteroid, 240 km) | open land to settle, fleece, platinum |

Airless bodies (Crag, Frostbite, every moon, the belt, Sconce, Snuff, Shale,
Lode) have no sky, weather or clouds and are lifeless. "Living zone" is in the
game's km, scaled from Aubade's by the square root of the light.

Names: Aubade, Nocturne, Tallow, Gemel, Sconce, Wick, Censer, Snuff, Shale,
Fallow, Lode and every new species name were checked against the mockups;
none was already used. Two collisions are inherited, not new: Tenebris's Rime
against Amber's Rime (S5: keep Tenebris's; rename Amber's when M1 is next
revised) and Tenebris's Verdant beside Aubade's Verdance (different names,
kept).

## 4. Getting there: star lanes

What the other mockups establish:
- `travel.html` (M2): ships fly Newtonian at up to 4 km/s with a transit drive;
  **lanes** are gate pairs that carry a ship to the partner gate at 25 km/s
  "with nothing simulated in between", built late as a goal; Glaucus is "the
  far end of the lanes".
- `ship-tiers.html` (M21): chemical, nuclear, fusion and antimatter tiers; the
  page never mentions warp or other stars. Its change records the owner's
  2026-11-12 words: antimatter "starships ... can travel between stars using
  warp drives", **withdrawn the same day: "Let's not do warp."**
- CLAUDE.md: Newtonian ships, no patched conics, f64 system coordinates with an
  explicit frame.

No conflict between M2 and M21. The game's systems are drawn small (Pale Blue
orbits 300 km from Aubade), so no ship crosses light years by flying, at any
tier. The recommendation (S3) uses M2's mechanism, not a drive: a **star
lane** is a pair of lane gates, one 1,800 km from each star; the passage takes
30 s of play per light year (2:18 to Nocturne, 3:08 to Tallow, 4:03 to Gemel)
and drops the ship in the far star's frame at the speed it entered. The first
passage is an uncrewed seed run carrying the far gate's kit, so nobody is
stranded. Gates need the antimatter collider's power and platinum. World time
passes in a passage as in any other play.

This is the closest the design comes to the line the owner drew ("Let's not
do warp"): the ship never gains a faster-than-light ability; the lane is a
structure in the world. It is recorded as a recommendation and goes to the
owner with the star map's screenshot if they want to look.

Frames: each system is its own f64 frame; a passage rebases the ship into the
destination system's frame at a tick boundary, as M2's frame spheres do
between bodies. A body's system is part of its world id.

## 5. The mockup

planets.html gains five tabs (one 3D view on the page at a time, the open tab a
bare `#anchor`): `#starmap`, `#nocturne`, `#tallow`, `#gemel`, `#tenworlds`.
The page now has **one WebGL context**: a single canvas and renderer, made
when the first 3D tab opens and moved into the open tab's view; every view's
pointer handlers check the open tab. Every new body is also in the deck, the
Selected body globe and the Surface vignette (pickers grouped by system), and
on the ladder.

Vignettes are lit by day and at night (lamps, fires, torches, lanternblooms,
glowing mycelium and mushroom spores, molten seams). Nothing turns on its own;
the bodies in the orbit views move with world time, under the existing Pause
and speed buttons. The ported worlds wear Tenebris's textures and Tenebris's
animal atlas; Tallow and Gemel wear the game's own atlas sheets, tinted. New
pixel art (maps, portraits, species, resources, box textures) is drawn by
`tools/art/star_systems_art.py` with the balatro-pixel-art kit; it imports
`planets_art.py` and writes only new files.

Verified in headless Chromium on SwiftShader (no GPU in the cloud session; no
frame numbers taken): no console errors on planets, travel and minigames; each
new tab draws; one WebGL context across every tab; no horizontal scroll at
390 px. Screenshots in `docs/screenshots/mockup-planets/`. The GPU screenshot
gate of ART-DIRECTION.md is for the owner's desktop.
