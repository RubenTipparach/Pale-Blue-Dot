# Design: seas, rivers and marshes (ocean-environments)

## Context

The proposal quotes the owner's four messages. This design fixes what the
mockup shows, the rules each place comes from, and how each would enter the
engine, so the page, the generator and the spawner can never disagree.

What the game gives us, measured on `main`:
- **The cell.** 2.833 m flat to flat and 1 m a layer on every body (CLAUDE.md,
  "Hex size is fixed"). Seabeds terrace, reefs and ice are cells.
- **The column.** From `column::BASE_M` = -145 m up, 504 layers. A saved edit
  is indexed from `BASE_M`, which never moves.
- **The ocean floor** is 60 to 145 m down (`planet_terrain.rs`, the
  `ocean floor` assertion).
- **The sea.** One function in `pbd_core::sea` that the shader draws and hulls
  float on: fixed wavenumbers per planet, amplitudes from the wind by
  Pierson-Moskowitz with a lag, shoaling to 0.6 of the depth
  (`openspec/specs/planet/water`). Foam on crests and slopes
  (`water.ron` `foam_crest_*`, `foam_slope_*`).
- **Under the sheet.** The composite's murk is `deep_color` times the open
  water's light times `exp(-absorption_per_m * eye_depth)`, with
  `absorption_per_m` 0.32 / 0.035 / 0.012 (`water.wgsl` `murk`, `water-lit`).
  Snell's window and distortion are built (`water-composite`).
- **Fauna.** Eight species in schools of boids, spawned where the water class
  (river, shallows to 6 m, shelf to 40 m, deep) and the live temperature fit
  the species' window, never below -1.8 C (`fauna.ron`,
  `openspec/specs/world/fauna`). The roster is append-only.
- **Climate.** The atmosphere carries ocean currents and the sea's heat
  (`atmosphere/ocean.rs`); `climate-and-fish-maps` plans yearly normals,
  including sea ice frozen all year or in winter.
- **Rivers.** A ridged noise field carves channels below 40 m altitude down to
  3 m under the sea (`planet_gen.rs` `river_*`); their water is the sea's.
  `water-flow` ported the flow term, zero for every cell.
- **Water above the sea** does not exist: `volumetric-water` phase 1 makes
  water a material the renderer and walker see; phases 2 and 3 settle water at
  any height.
- **Authorship.** `world-persistence` designs a journal where every line
  names its author (`@p`, `@w<process>:<record>`, `@c`), world processes that
  propose edits, and a check that refuses any proposal touching a cell the
  player touched.
- **M7 and M9.** The animals mockup draws the shore crab, gull, heron, bog
  toad, mire lurker, firefly and glowmoth, and their drops. The fish lab adds
  bloatfin (reef), lanternjaw (deep angler), glassback (an ice moon's),
  emberback (vents) and sunplate (ponds). This page extends them and redraws
  none.

## Goals / Non-Goals

**Goals:**
- Each water place reads as itself at a glance, by day and at night, from the
  surface and from under it.
- Each place comes from planet data the game has or would derive, by one
  function the generator, the spawner and the map share.
- Life in every place, moving: what the player meets, takes and leaves alone.
- Nothing proposed resets a save or needs a deeper column.

**Non-Goals:**
- Game code. This change is the write-up and the mockup.
- Frame cost. Not measured here (a cloud container, SwiftShader).
- Water physics above the sea: that is `volumetric-water`'s, and this design
  waits for it.

## Decisions

### 1. One page, fourteen tabs in three rows

`docs/mockups/ocean.html`, titled "Seas, Rivers and Marshes" (the owner widened
"ocean" to rivers and marshes; the file keeps the name the task gave it).

| Row | Tabs |
| --- | --- |
| Sea | Open sea; Coast and tide; Coral reef; Kelp forest; Polar sea; The deep and vents; Trench, caves, wrecks |
| Fresh | River, source to mouth; Beavers and dams; Marsh and estuary |
| Guide | How underwater looks; Water life; Where they occur; In the engine |

The page follows `docs/mockups/ART-DIRECTION.md` and CLAUDE.md's mockup rules:
- **One 3D view at a time.** One WebGL renderer for the whole page, made when
  the first 3D tab opens and moved into whichever 3D tab is open; each place's
  scene is built the first time its tab opens; a 2D tab stops the loop. The
  open tab is the link's bare `#anchor`.
- **Lit at night by its own lights.** Every lamp lights what is within its
  range wherever the camera is (the lamps of `drydock.html`'s lit model):
  lanterns, the lighthouse, a mill, a cabin, a stilt hut, the diver's lamp,
  vent glow, lanternjaw lures, fireflies, glowmoths, sea sparkle. The moon
  stays dim.
- **Nothing spins.** The camera stays where the viewer leaves it; the
  lighthouse has a fixed light. What moves is alive (creatures) or moved by the
  world (waves, kelp in the surge, smoke from a vent, marine snow sinking).
- **The game's art first**, copied byte-identical into `ocean/game/` with a
  README; M7's and M9's portraits copied into `ocean/m7/` and `ocean/m9/`;
  everything else drawn by `tools/art/ocean_art.py` with the
  balatro-pixel-art kit.
- **The sea shader is drydock's** (M23, after its 2026-11-13 fix): waves fade
  out of the geometry between 3 and 6 wavelengths from the eye, normals and
  foam per pixel, foam in 0.5 m pixels, lamps lighting the foam. Whitecaps
  break in patches, never along a whole swell crest.

### 2. Light under water: one coefficient for the view, one for the day

Today one absorption does both jobs: it colours the view through water and it
dims the murk with the eye's depth. At its value for clear water, blue light
at the column's floor is still 19% of the surface's, so the deep reads as a
lit blue. Two coefficients split the jobs:

| Term | What it is | Ocean | Where |
| --- | --- | --- | --- |
| `absorption_per_m` | the view's path through water, per colour | 0.25 / 0.035 / 0.012 | exists (`water.ron`) |
| `downwell_per_m` | daylight fading with depth, per colour | 0.40 / 0.065 / 0.046 | new (`water.ron`) |
| `turbidity` | scatter per metre toward the water's own colour | 0.022 | new, per place (`ocean.ron`) |

Lamps under water are absorbed along their own path, so a diver's lamp is warm
near and blue-green far, and a red plume shows red only close up.

Turbidity per place (sight distance, about `3 / (turbidity + K_blue / 2)`):
reef 0.016 (over 100 m), ocean 0.022, polar 0.03, kelp 0.05, coast 0.07 (about
30 m), river 0.12 (15 m), marsh 0.35 (5 m).

New terms, all derived and none saved:
- **Caustics** on wet ground under water by day, in 0.25 m steps to match the
  nearest-sampled textures, fading with depth.
- **God rays**: shafts from the surface seen from under it by day, fading with
  the eye's depth.
- **Marine snow**: particles sinking round the eye in the twilight and
  midnight zones, seen in lamplight.
- **Spray and streaked foam** off crests from a gale up (`spray_from_mps`).

### 3. The deep fitted to the column

The column ends at -145 m and cannot grow downward without moving `BASE_M`,
which would break every saved edit. So the zones are defined by the light left,
not by real-ocean metres, and fall inside the column:

| Zone | Blue light left | Depth (ocean water) |
| --- | --- | --- |
| Sunlit | over 15% | 0 to 40 m |
| Twilight | 1.7 to 15% | 40 to 90 m |
| Midnight | under 1.7% | 90 m to the floor |

Recommendation; one of the owner's three questions (Q3, with the light tab's
screenshots at 40, 90 and 140 m).

### 4. Where each place occurs

One function, `ocean::environment(seed, version, direction, normals)`, in
`pbd-core`, read by the generator (what the cells are), the spawner (what lives
there) and the map (the chart layer). The rules, with their data:

| Place | Rule | Read from |
| --- | --- | --- |
| Open sea | deeper than the shelf (40 m), away from coasts and ice | height |
| Coast and tide | within two cells of the shoreline | height, the beach biome |
| Coral reef | shallows (under 6 m) where the coldest month's sea is above 23 C, no river mouth within 20 km | height, sea temperature normals, drainage |
| Kelp forest | rocky shelf, 6 to 40 m, at 5 to 20 C, strongest where the current diverges (upwelling) | height, rock noise, sea temperature, the atmosphere's ocean current |
| Polar sea | sea frozen all year or in winter | climate normals' sea-ice layer |
| Vents | the deepest sea along the mid-ocean ridges of the plate field | height, ridge field |
| Trench | where plates meet under the sea; the basin carved to the column floor | height, plate field |
| River | where gathered rain passes a threshold, down the steepest descent to the sea | height, rain normals |
| Beaver stream | a river cell with flow under a threshold in temperate forest (fields and redwood biomes, 40 to 60 degrees), trees within 20 m | drainage, biome, latitude |
| Marsh and estuary | flat land within a few metres of the sea that the rain keeps wet; estuaries at river mouths; mangroves where the sea is above 20 C | height, slope, rain, drainage |

The chart tab draws these rules on a sample Tenebris-like planet made in the page (Tenebris is the home planet, in the Aubade system; Sequoia, its sister planet with green seas under a brown sky, would run the same rules on its own data). It is a
picture of the rules, not of the shipped seed; the in-game layer would be
computed by the function above.

### 5. What each place is made of

Cells (dug, mined, saved as edits like any ground):
- **reef rock and brain coral** blocks on a sand bed (the atlas's ocean sheet:
  ripple sand, coral rubble, coral and starfish tiles; brain coral a new
  material);
- **sea ice** floes (cells floating on the sea, one to four together), icebergs
  (stacked cells to 12 above and 30 below) and the ice shelf (a wall eight up and
  twenty down), from the frozen sheet;
- **vent ground**: basalt, mussel beds and bacterial mats round each vent;
- **marsh**: moss hummocks, peat, pools one layer down; mudflats;
- **dam and lodge sticks** (beaver-made, see 8).

Pieces (standing on cells, derived like trees and ground clutter): staghorn,
table coral, sea fans, soft coral, anemones, sponges, kelp (stacked cards to
the surface and a canopy), seagrass, reeds and cattails, lily pads, mangroves,
black smoker chimneys and tube worm clusters.

### 6. The places, scene by scene

| Tab | Shows | Life (moving) | Night lights |
| --- | --- | --- | --- |
| Open sea | a hex-cell sloop hove to; sea state calm to storm (Hs 0.15 to 2.9 m), whitecaps, spray, rain; Snell's window from below | gliders leaping, silverfin, lantern jellies, deepback, a serpent deep down, sea sparkle | the sloop's two lanterns, sea sparkle |
| Coast and tide | beach, rock headland with pools, sea cave arch, lighthouse; tide slider over 3 m; pools keep their water above a falling sea | shore crabs sideways (M7), hermits, starfish, urchins, anemones, shrimp in pools, gulls, minnows | the lighthouse's fixed light, the hut's lantern, a driftwood fire, sea sparkle in the surf |
| Coral reef | coral cells and pieces on white sand 1 to 6 m down | reef fish, darters in anemones, beakfish grazing coral, bloatfin (M9), turtle, ray, clams that shut when you come within 4 m, an octopus that changes colour, inks and hides within 6 m | diver's lamp, sea sparkle |
| Kelp forest | fronds from boulders to a surface canopy, swaying; an urchin barren | otters rafting on their backs, kelpfin, silverfin, urchins, abalone, lobsters, crabs, a seal | diver's lamp |
| Polar sea | pack ice floes bobbing, an iceberg, an ice shelf; under the ice | seals that slide in within 9 m, krill swarms, silverfin, deepback | the expedition camp, the ship's lanterns |
| The deep and vents | black smokers on basalt, smoke rising, mats, mussel beds, tube worms that retract within 3.5 m | vent crabs, glowing vent shrimp, emberback (M9), lanternjaw lures (M9), glow squid, jellies, marine snow | vent glow, lures, the submersible's lamp |
| Trench, caves, wrecks | a shelf breaking into a stepped cliff to the floor; a cave with a dry air pocket; a wreck of hex cells with a chest | eels in the wreck, an octopus, lobsters, a ray, deepback, a serpent in the trench | a crystal glow in the pocket, the diver's lamp |
| River | spring, riffles, a four-layer fall, plunge pool, rapid, meanders, gravel bars, mouth; flow streaks downstream, white water on every fall | redrun leaping the fall, trout holding in the pool, perch, minnows, crayfish, river mussels, a river otter, a kingfisher diving, a heron (M7), diving ducks, dragonflies | the mill's lantern, glowmoths (M7), fireflies (M7) |
| Beavers and dams | a valley stream; days 0 to 40: felling, the dam cell by cell, the pond, the lodge; the journal | beavers hauling branches and gnawing, a duck family, a diving duck, trout, frogs on lilies, a heron, dragonflies | the cabin's lantern, a lantern post on the pond, fireflies |
| Marsh and estuary | hummocks and pools, peat, reeds, lilies, mangroves on mudflats, the brackish channel; dawn mist | dabbler flocks that lift and land on Startle, a duck family, a diving duck, a heron, bog toads and a mire lurker (M7), frogs, eels, minnows, fiddler crabs | the stilt hut's lantern, fireflies, glowmoths, a faint marsh-gas glow |

### 7. Water life

The bestiary lists 61 entries: the eight species in the game today, seven of
M7's animals, five of M9's fish, and 41 new entries (38 species plus the
anemone, the dabbler hen and the duckling as their own cards). Each card gives
where it lives (place, depth, water temperature), how it behaves, what it
gives (M7's drop icons: meat, shell, feather, egg, glow, hide) and how it is
taken.

In the engine:
- New species are **appended** to `fauna.ron`'s roster (a caught fish is saved
  as its index).
- Each species gains `env:` tags beside its water classes: reef, kelp, vent,
  under-ice, pool, riffle, marsh, estuary, mudflat. The spawn rule becomes class
  and environment and temperature.
- Beside the school, two movers: a **bed crawler** (walks the cells' tops
  between waypoints, crabs sideways) and a **surface floater** (rides the sea
  function: otters, ducks, seals hauled out).
- Behaviours shown on the page, each a small state machine on the CPU,
  deterministic like the schools: clam open and shut, octopus out, inking and
  hiding, tube worm plume out and in, seal out and in, duck tipping up, duck
  family line, flock lift and land, kingfisher dive, redrun leap.
- **Ducks** (the owner's "And ducks"): the reed dabbler (drake and hen), its
  ducklings and the pond diver. They tip up to feed, lead ducklings in a line
  in each other's wakes, lift as a loose V when startled and land again, and
  sleep on the water at night with their heads tucked. They give feathers and
  eggs (M7's icons), gathered, not shot.
- **Protected**: reef turtles, sea otters, seals and ducks are not caught;
  the action is refused, as a town's livestock is.
- Creatures are pixel cards: a side view turned along the heading and toward
  the eye, a crawler laid flat. Recommendation; the owner's Q2, with
  screenshots.

### 8. Beavers and dams: the world changing itself

A beaver colony is a stored record (`rec colony-<n> colony {members, site}`)
and a world process `beaver` with a period of an in-game hour
(`world-persistence` decision 7). Each period it may propose:
- **fell**: a tree within 20 m of its site becomes a stump (a journal entry; the
  tree was derived, the stump is the edit);
- **dam**: a sticks cell, layer by layer from the stream bed up, across the
  stream at its site, up to four layers;
- **lodge**: sticks cells in the pond once the dam holds water.

Every proposal is checked against the cells the player has touched and refused
if it would change one (`world-persistence` decision 6). So:
- **A jetty first.** If the player built on the stream at the site before the
  colony came, the dam is refused there and the colony moves to its next site
  upstream (its record changes, authored by the process).
- **A broken dam.** If the player breaks the dam, those cells are now the
  player's. The colony does not rebuild on them; it moves to its next site.
  The pond drains.
- **The pond is not saved.** It is what volumetric water does behind saved
  cells: drain a dam and the water goes; rebuild it and it fills.

The mockup's day slider runs this from day 0 to day 40, with buttons for the
jetty and the break, and the journal beside it shows the entries with their
authors.

### 9. Rivers and marsh pools wait for volumetric water

A river that steps down from a spring, a marsh pool 0.6 m above the sea and a
beaver pond all need water above sea level. They are built after
`volumetric-water` phases 2 and 3 and a drainage river pass in a new generator
version. The river tab is that target: steps of one layer are riffles, two or
three a rapid, four or more a fall; the flow field (`water-flow`'s, zero today)
then carries a vector downstream on every cap.

### 10. Tides

A tide is an offset added to the one sea function, so the drawn sea, the
floated hull and the walker agree, as they do today. Range 3 m on
Tenebris from its moon, smaller on a moonless body, two highs a day. Derived from
the clock, never saved. Rock pools hold water above a falling sea once water
can stand above the sea (decision 9). Recommendation; the owner's Q1, with
the coast tab's low and high tide screenshots.

## In the engine, part by part

| Part | Today | New | Config |
| --- | --- | --- | --- |
| Waves by the wind, shoaling | built | swell from distant storms | `water.ron` `sea.swell_*` (exists) |
| Foam | crest and slope foam | spray and streaked foam in a gale | `water.ron` `spray_from_mps` |
| Seeing under water | murk by one absorption, Snell's window | downwelling by depth, turbidity per place | `water.ron` `downwell_per_m`; `ocean.ron` `turbidity` |
| Caustics, shafts, snow | none | all three | `water.ron` `caustics`, `shafts`, `snow` |
| Tides | none | sea function offset | `ocean.ron` `tide_range_m` per body |
| Depth zones | none | by light left | `ocean.ron` `zones_m` |
| Reef, kelp, vents, trench, ice | none | environment classes, cells and pieces, new generator version | `ocean.ron` rules |
| Water life | eight species | appended species, `env:` tags, crawler and floater movers | `fauna.ron` |
| Rivers above the sea, ponds, pools | impossible | volumetric water and a river pass | `column.ron` river pass |
| Beaver dams | none | colony record, `beaver` process, journal entries | `fauna.ron` colony rules |

## Saves

- A new generator version for every change to what the ground is made of;
  each old world keeps its version, pinned by its digest test, and opening it
  never rebuilds it.
- Habitat, environment classes, tides, ponds and everything a spawner reads
  are derived and never saved.
- A colony, its dams, lodges and stumps are journal entries with authors.
- A pearl bed worked or a stock fished down, once those exist, are records.
- The column does not change.

## Risks / Trade-offs

- **Caustics and snow cost frame time.** The caustic term is a few sines per
  wet fragment; snow is a few hundred points. Measured on the owner's desktop
  before shipping (`tools/perf_suite.py`), not here.
- **Cards seen edge on.** A side card turned toward the eye about its heading
  is thin only when the eye is straight ahead of or behind the creature.
- **Fitting the deep to 145 m** makes the twilight zone shallower than Earth's.
  The planet is small (150 m summits), so the scale is consistent.
- **A beaver colony can flood a field.** It never floods a cell the player
  touched; a field the player has tilled is touched.

## Questions for the owner (with screenshots)

- **Q1. Tides.** `coast-low-tide-day.jpg` beside `coast-high-tide-day.jpg`.
  Recommend yes, 3 m from the moon.
- **Q2. Creatures as pixel cards.** `reef-life-day.jpg`, `marsh-ducks-day.jpg`,
  `sea-life-day.jpg`. Recommend cards for anything smaller than a walker.
- **Q3. The deep fitted to the column.** `light-40m.jpg`, `light-90m.jpg`,
  `light-140m.jpg`, `deep-day.jpg`. Recommend fitting.

## Recommendations taken (ask only with screenshots)

Each of these is recorded as "recommendation taken (ask only with
screenshots)", not the owner's words:
- D4. New places are a new generator version for new worlds.
- D5. Reef rock, brain coral, ice and dam sticks are cells; living coral, kelp,
  reeds and lilies are pieces.
- D6. Rivers, marsh pools and beaver ponds wait for volumetric water.
- D7. A beaver dam yields to the player; a broken dam moves the colony on.
- D8. Spawning reads environment tags beside the water classes.
- D9. Turtles, otters, seals and ducks are not caught; eggs and feathers are
  gathered.
- D10. The lighthouse has a fixed light.
- D11. Turbidity per place, from reef (over 100 m of sight) to marsh (5 m).
- The page's title is "Seas, Rivers and Marshes"; the file stays
  `ocean.html`.
