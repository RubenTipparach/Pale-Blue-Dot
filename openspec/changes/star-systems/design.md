# Design: Aubade, its neighbours, and Tenebris's worlds

Every decision below is a recommendation taken (ask only with screenshots)
unless it says it is the owner's; the owner can overturn any of them on the
tracker. The decisions table on planets.html (P1 to P9, S1 to S14) carries the
same list.

## 0. The owner's corrections, 2026-11-14

Three messages after the first draft, verbatim:

> pale blue = tenebris, so call it that, sequoia is the sister planet

> also there is no landing on gas giants, some kind of air ship could deploy
> in the atmosphere though, but at a certain point in the atmosphere should
> just be an ocean of hydrogen

> gas giants will generally destroy any spacecraft that reaches a certain
> depth, well before they get to the ocean

And on a screenshot of the orbit view, where Cinder and Rime crossed Amber's
rings:

> make note that the moons should not be intersecting rings. maybe just shrink
> the planet+ring for now, but keep the proportions for that system (gas giant
> + moons is a system as opposed to solar system)

So: the home world is named Tenebris (it always was Tenebris's planet: the
game's generator is `TENEBRIS` v5); Sequoia is its sister in the home system;
the first draft's separate copy of Tenebris in a neighbouring system,
Nocturne, is gone (section 3); gas giants have no landing (section 6); and a
giant with its moons is drawn as a system at its own scale (section 7).

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

The scale is set by Tenebris itself: its 300 m is the home world's 4,800 m
(L11), **sixteen times**. Rule: **sixteen times the Tenebris radius, down to
the rung at or below it, never under 300 m.** It needs no exception (the first
draft, at eight times, had to move Sequoia up a rung by hand). Gravity is
Tenebris's, so Sequoia is still the heavier sister at 1.1 g, though it shares
Tenebris's rung: 1.2 times Tenebris's radius is nearer one rung than two, and
9,600 m (L12) would quadruple its cells.

| Body | Tenebris | 16 x R | Here |
| --- | ---: | ---: | --- |
| Tenebris (the home world) | 300 m | 4,800 m | 4,800 m, L11: the game's planet today, unchanged |
| Sequoia | 360 m | 5,760 m | 4,800 m, L11 |
| Frostbite | 220 m | 3,520 m | 2,400 m, L10 |
| Crag | 190 m | 3,040 m | 2,400 m, L10 |
| Luna | 80 m | 1,280 m | not carried: Lantern (1,200 m, L9) stands in its place (S10) |
| Verdant | 60 m | 960 m | left out (S11) |
| Selene | 50 m | 800 m | 600 m, L8 |
| Quartz | 44 m | 704 m | 600 m, L8 |
| Rime, here Hoar | 40 m | 640 m | 600 m, L8 |
| Sprout | 36 m | 576 m | 300 m, L7 |
| Asteroids A to J | 10 to 22 m | 160 to 352 m | 300 m, L7 each (the floor), in Aubade's belt (S12) |

Alternatives rejected: eight times (the first draft: Tenebris would be
2,400 m, not the home world); keeping each body's Tenebris cell count (almost
every body becomes 300 m and the size order is lost); the nearest rung to the
radius itself (everything collapses onto 300 m).

## 3. Where they live: Aubade, and the neighbours

**Tenebris's bodies are in the home system, Aubade** (the owner's words, S2,
S9). Tenebris's own order round its star is kept, spread to fit beside the
system's own worlds (S13):

| Orbit (km) | Body | From | Moons |
| ---: | --- | --- | --- |
| 120 | Crag, 2,400 m, airless rust | Tenebris | Quartz, 600 m, 32 km |
| 180 | Ochre | Aubade | |
| 300 | **Tenebris**, the home world | the game | Lantern (in Luna's place), 26 km; Selene, 600 m, 44 km |
| 380 | **Sequoia**, Tenebris's sister | Tenebris | Sprout, 300 m, 30 km |
| 450 | Verdance | Aubade | |
| 650 | The belt: Pebble and Tenebris's ten rocks | both | |
| 1,000 | Amber (no landing) | Aubade | Cinder, Rime, Murk |
| 1,600 | Glaucus (no landing) | Aubade | Hollow, Tidewater |
| 2,400 | Frostbite, 2,400 m, airless ice | Tenebris | Hoar (Tenebris's Rime), 600 m, 48 km |

Sequoia rides the next orbit out from Tenebris, as it rode beyond Tenebris in
Tenebris's own system (40 km against 30 there; 380 against 300 here), inside
the living zone (250 to 400 km). Crag goes innermost, inside Ochre; Frostbite
outermost, past Glaucus and the star-lane gate (1,800 km). Tenebris's spacing
in km (Crag 220, the belt 340 to 380, Sequoia 400, Frostbite 550) would crowd
seven orbits into 180 to 650 km, so the order is kept and the spacing is not.

Clashes resolved (recommendations taken, ask only with screenshots):
- **Two Rimes** (S5): Amber's moon keeps Rime, because travel.html's Rime depot
  and the minigames name it; Frostbite's is renamed **Hoar** (hoarfrost), its
  look and numbers unchanged. Its art files moved with it
  (`map_tenrime.png` to `map_hoar.png`, `portrait_tenrime.png` to
  `portrait_hoar.png`, the same bytes).
- **Luna and Selene against Lantern** (S10): Lantern stays (travel.html and
  ship-tiers.html fly to it) and takes Luna's place: the home world's larger
  grey airless moon, and Luna at sixteen times lands on Lantern's own rung.
  Selene joins as a small outer moon.
- **Verdant against Verdance** (S11): Verdant, a bare 60 m moon one letter from
  Verdance's name, is left out; Sprout is Sequoia's one moon.
- **Two belts** (S12): one belt at 650 km; Tenebris's ten rocks are in it beside
  Pebble.
- **The duplicate Tenebris**: removed with its vignette (a homestead in
  Tenebris's textures, with Tenebris's barnyard and stalker). The home world
  keeps the game's look and roster; adding Tenebris's barnyard to it would be
  new game content, a question for later. The species icons stay in the art
  row.

**Nocturne is retired** (S2): it existed only to hold Tenebris's bodies, and
Tallow and Gemel already make the "at least 2 more star systems" the owner
asked for. Its star portrait and the dropped bodies' maps and portraits are
removed from `docs/mockups/planets/` and from `star_systems_art.py`.

| System | Star | Light | Living zone | Bodies | Why go |
| --- | --- | ---: | --- | --- | --- |
| Aubade (home) | G2 V yellow | 1.0 | 250 to 400 km | 18 (above) | home: Tenebris and Sequoia |
| Tallow, 6.3 ly | M2 V red dwarf, flares | 0.03 | 45 to 70 km | Sconce (1,200 m, L9, airless iron, 30 km, locked), Wick (2,400 m, L10, thin cold air, 56 km, locked; twilight-ring life: blackleaf trees, lanternbloom, tallowback, dimwing), Censer (gas giant, 9,600 m cloud tops, no cells, no landing, 160 km), Snuff (600 m, L8, airless sooty ice, moon of Censer) | blackwood, emberglass, a twilight world |
| Gemel, 8.1 ly | K2 V orange plus a white dwarf (Gemel B, 420 km out) | 0.35 | 145 to 230 km | Shale (1,200 m, L9, airless slate, 90 km), Fallow (2,400 m, L10, cold dry Earthlike air, 175 km; steppe grass, stone pines, drover, scythewing, burrowkit; no townsfolk), Lode (300 m, L7, metal asteroid, 240 km) | open land to settle, fleece, platinum |

Airless bodies (Crag, Frostbite, every moon but Tidewater and Murk's haze,
the belt, Sconce, Snuff, Shale, Lode) have no sky, weather or clouds and are
lifeless. "Living zone" is in the game's km, scaled from Aubade's by the square
root of the light.

Names: Aubade, Tallow, Gemel, Sconce, Wick, Censer, Snuff, Shale, Fallow, Lode,
Hoar and every new species name were checked against the mockups; none was
already used.

**Saves.** The home world keeps its generator (`TENEBRIS` v5), its radius
(4,800 m, L11), its orbit and its look. Its name in the mockups is a label: no
save stores "Pale Blue" as a world name, so the rename changes no save and
needs no new generator version. Every body added to Aubade is a new world with
its own generator id at v1, its digest test taken before it ships.

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

No conflict between M2 and M21. The game's systems are drawn small (Tenebris
orbits 300 km from Aubade), so no ship crosses light years by flying, at any
tier. The recommendation (S3) uses M2's mechanism, not a drive: a **star
lane** is a pair of lane gates, one 1,800 km from each star; the passage takes
30 s of play per light year (3:08 to Tallow, 4:03 to Gemel) and drops the ship
in the far star's frame at the speed it entered. The first passage is an
uncrewed seed run carrying the far gate's kit, so nobody is stranded. Gates
need the antimatter collider's power and platinum. World time passes in a
passage as in any other play.

This is the closest the design comes to the line the owner drew ("Let's not
do warp"): the ship never gains a faster-than-light ability; the lane is a
structure in the world. It is recorded as a recommendation and goes to the
owner with the star map's screenshot if they want to look.

Frames: each system is its own f64 frame; a passage rebases the ship into the
destination system's frame at a tick boundary, as M2's frame spheres do
between bodies. A body's system is part of its world id.

## 5. The mockup

planets.html's tabs (one 3D view on the page at a time, the open tab a bare
`#anchor`): Orbit view, Selected body, Surface, `#starmap`, `#tallow`,
`#gemel`, `#tenworlds` ("Tenebris and Sequoia"), `#descent` ("Gas giants"),
the deck, the ladder, lighting and art, decisions. The page has **one WebGL
context**: a single canvas and renderer, made when the first 3D tab opens and
moved into the open tab's view; every view's pointer handlers check the open
tab. Every body is in the deck, the Selected body globe and (giants excepted)
the Surface vignette, pickers grouped by system, and on the ladder.

The Tenebris and Sequoia tab shows each of Tenebris's bodies with what it was
in Tenebris and where it is here; the home world's entry is the game's own
vignette, unchanged. Vignettes are lit by day and at night (lamps, fires,
lanternblooms, glowing mycelium and mushroom spores, molten seams). Nothing
turns on its own; bodies in the orbit views move with world time, under the
Pause and speed buttons; cloud decks drift and lightning flashes in the giant
view because the world moves them. Sequoia and the barren and icy bodies wear
Tenebris's textures and animal atlas; Tallow and Gemel wear the game's own
atlas sheets, tinted. New pixel art is drawn by
`tools/art/star_systems_art.py` with the balatro-pixel-art kit; it imports
`planets_art.py` and writes only new files (re-run on 2026-11-14 into a scratch
folder: every file it writes is byte-identical to the checked-in one).

Verified in headless Chromium on SwiftShader (no GPU in the cloud session; no
frame numbers taken): no console errors on planets.html and travel.html (the
one "failed to load" line is the browser's own favicon request), each 3D tab
draws, one WebGL context across every tab, no horizontal scroll at 390 px.
Screenshots in `docs/screenshots/mockup-planets/`. The GPU screenshot gate of
ART-DIRECTION.md is for the owner's desktop.

## 6. Gas giants: no landing (the owner, 2026-11-14)

Amber, Glaucus and Censer have no surface, no cells and no landing. The
Surface picker leaves them out; the deck and Selected body say "No ground, no
landing"; "Its surface" on a giant opens the Gas giants tab instead. What a
player does there:

- **Dock** at the skimmer platform, which floats at the top of the airship band
  on a hot-hydrogen envelope (travel.html's platform, recast: nothing on it
  changes but what holds it up).
- **Drop an airship**: an aerostat with a hot-hydrogen envelope (hydrogen in
  hydrogen lifts only hot), floating in a band of the cloud decks and scooping
  helium-3 and hydrogen on a line below it (the fusion tier's helium-3 on
  industry.html comes from here). Recommendation S14: the band runs from just
  under the cloud tops down through the water deck, about 1 to 5 bar.
- **The limit**: below the band pressure, heat and wind rise to a depth where
  **every craft is destroyed**, airship or ship, of every tier ("gas giants will
  generally destroy any spacecraft that reaches a certain depth, well before
  they get to the ocean"). One limit per giant, for every tier: no tier is
  built to go deeper.
- **The hydrogen sea**: at the bottom of the visible atmosphere, far below the
  limit, past a long zone where nothing survives, a sea of liquid hydrogen;
  metallic hydrogen under it to the core. It is seen (a dark glint through the
  decks), never reached.

Depths are metres under the cloud tops (the giant's ladder radius), in the
game's small scale; pressures and temperatures are picks for feel, with
log-pressure and temperature interpolated between these points:

| | Amber (19,200 m) | Glaucus (9,600 m) | Censer (9,600 m) |
| --- | --- | --- | --- |
| Haze | +800 m, 0.1 bar, -150 C | methane haze, +600 m, 0.1 bar, -220 C | red haze, +900 m, 0.05 bar, -140 C |
| Cloud tops (top deck) | ammonia ice, 0 m, 0.7 bar, -120 C | methane ice, 0 m, 0.5 bar, -210 C | ammonia, 0 m, 0.7 bar, -110 C |
| Second deck | ammonium hydrosulfide, -350 m, 2 bar | hydrogen sulfide, -300 m, 3 bar | ammonium hydrosulfide, -380 m, 2.2 bar |
| Lightning deck | water, -750 m, 5 bar, +5 C | water and ammonia, -650 m, 8 bar | water, -800 m, 6 bar, +15 C |
| Platform | -120 m | -100 m | -120 m |
| Airship band | -80 to -700 m | -60 to -560 m | -80 to -720 m |
| **The limit** | **-1,500 m, 30 bar, +150 C** | **-1,200 m, 40 bar, +120 C** | **-1,400 m, 35 bar, +160 C** |
| Hydrogen sea | -7,200 m, ~2,000 bar | -5,400 m | -6,400 m |
| Below | metallic hydrogen | metallic hydrogen | metallic hydrogen |

The Gas giants tab draws it: a 3D view of an airship in the decks (the
envelope in the game's cloth, the gondola in its plaster and iron, lit windows
and floods that light the envelope at night, lightning in the deepest deck,
the sea far below), a depth slider that runs from the band down past the
limit, a hull-pressure gauge in the view's HUD with a red limit mark (green in
the band, amber below it, red near the limit), and the profile beside it as a
column and a table. Pushed past the limit, the airship is lost: the view says
so and a Reset puts a new one back in the band.

## 7. A giant and its moons are a system of their own

A giant and its moons are a planetary system, drawn at their own scale in the
orbit view; moons never cross the rings. For a ringed giant (Amber) one scale,
0.157 display units a km, sets the giant's radius (3.0), its rings (1.35 to
2.3 radii, inside the Roche limit, as real rings are) and its moons' orbits
(Cinder 9.4, Rime 14.9, Murk 22.0), so the true proportions of the giant's own
system are kept and every moon orbits clear of the outer ring edge. The page
checks it on load (an error in the console if a moon's orbit, less its drawn
radius, reaches the ring). The solar-system distances stay not to scale, as
before. Unringed Glaucus keeps its enlarged look.

## Verdance wears Tenebris's surface set (owner, 2026-11-14)

The owner: "can you have verdance use my old texture set from tenbris-rust?". Verdance's surface in
the mockup now lays Tenebris's own tiles (the byte-identical copies in `docs/mockups/planets/tenebris/`)
on its columns, untinted: `dirt` at and under the water line, `jungle_grass` over `jungle_dirt_grass`
on the low ground, `grass` over `dirt_grass` from 3 m up, `dirt` below and `rocks` for stone. In the
game this is Verdance's block palette naming Tenebris's surface blocks; Verdance is not built, so no
save is touched.

