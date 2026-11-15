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

### 1. One page, fifteen tabs in three rows

`docs/mockups/ocean.html`, titled "Seas, Rivers and Marshes" (the owner widened
"ocean" to rivers and marshes; the file keeps the name the task gave it).

| Row | Tabs |
| --- | --- |
| Sea | Open sea; Coast and tide; Coral reef; Kelp forest; Polar sea; The deep and vents; Trench, caves, wrecks |
| Fresh | River, source to mouth; Beavers and dams; Marsh and estuary; Water sim (added 2026-11-15, decision 11) |
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
- Creatures were pixel cards in the first draft (a side view turned along the
  heading, a crawler laid flat). The owner answered Q2 on 2026-11-15: every
  animal alive in the world is a low-poly 3D body, and a caught fish dropped
  by the player is a 2D item sprite (decision 11).

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

(In the mockup they are drawn by the Water sim's rule since 2026-11-15:
decision 12. In the game they still wait, as below.)

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

### 11. The owner's review of 2026-11-15: 3D animals, a water sim, a denser reef

The owner, looking at the river tab's waterfall:
- "there should be a sim where I can place water blocks and see how their
  physics plays out, I had asked to look at minetest source code for water
  implementation"
- "ducks should be 3d, birds in general are 3d, this water fall has zfighting"
- "all fish should also be 3d, unless they are caught, and a player drops them
  as 2d sprites"

And of the reef, the same day: "coral reefs need to be denser, with
vegetation and stuff", with a reference photo ("just look at this scene!").
The photo is not in the repository (its licence is unknown); what it shows,
and what the reef tab now aims at: wide flat table corals on short stalks,
dark teal-green with a hard rim, layered at different heights as the skyline
in the middle distance; dense rounded clumps of branching and finger corals in
cream, lilac, pink, mauve and tan, some knobbly like cauliflower, crowded edge
to edge with no sand between them near the eye; a big anemone, long waving
yellow-green tentacles over a purple foot, with an orange clownfish in it;
boulder and brain corals, sponges and rubble in greys, greens and pinks in
every gap, small yellow soft-coral tufts; a cloud of hundreds of small orange
anthias over the reef and dark damselfish higher up; bright cyan water above,
deep blue in the distance, colour most saturated near the eye.

What changed in the mockup:
- **Animals are 3D in the world** (this answers Q2). Every duck (drake, hen,
  duckling, diver), the heron, kingfisher and gulls, every fish, eel, ray,
  squid, jelly and octopus, every crab, lobster, crayfish, hermit, starfish,
  urchin, clam and mussel, and the seals, otters, beavers, frogs, toads,
  turtle, mire lurker and dragonflies are low-poly meshes
  (`docs/mockups/ocean/creatures3d.js`): flat shaded, one colour per face, the
  colours read off each creature's own sprite. The fish body follows
  `fish.html`'s `buildFish` (M9): a six-sided body in rings with tail, dorsal,
  anal and pectoral fins. A swimmer sways from the tail in the vertex shader;
  a duck paddles, tips up, tucks its head to sleep and flies with its wings;
  a heron strikes with its neck; a ray and a turtle flap; an octopus's arms
  move; a jelly's bell pulses; the fiddler waves its claw. Behaviour is
  unchanged (`duckFamily`, `duckFlock`, the octopus, the clams, the seals):
  only how they are drawn. A school is one `InstancedMesh`.
- **A dropped catch is a 2D item sprite.** The river tab's "Drop a catch"
  drops a caught fish on the bank as its item icon (the game's own,
  `assets/items/fish`), standing on the ground and turned to the eye, beside
  the live 3D trout and perch in the pool. It is the only animal left as a
  sprite in a scene. The bestiary keeps the sprites as portraits; plant cards
  (kelp, reeds, lilies, mangroves) stay cards.
- **The reef is packed**: a jittered grid every 0.62 m, dense within 24 m and
  round the default view, thinner to 46 m, each point something, clustered by
  kind and grown on one another (branching and finger corals on boulders),
  with turf and rubble filling the gaps and the eye's line to the anemone kept
  low. Fourteen kinds (branching, finger, boulder and brain, table, soft
  tufts, tube and barrel sponges, fans, anemones, algae turf, sea grass,
  seaweed, rubble, shells) and the big anemone: 9,568 pieces in 15 instanced
  draws, coloured per instance from the kind's palette; 540 anthias, 40
  damsels and the reef's other fish in instanced schools. The default view
  ("Reef") draws 88 calls and about 0.96 million triangles (SwiftShader, so
  only the counts are meaningful). Two new species in the bestiary, the reef
  anthias and the sooty damsel. At night the reef is dim, lit by its glowing
  life, the sea sparkle and the diver's lamp.
- **The waterfall's z-fighting is gone.** A falling sheet stood on the cell's
  edge, the same plane as the cliff face (the upper column's side), and the
  two fought. Each sheet now stands 0.08 m in front of the edge, toward the
  lower cell, reaching 0.05 m past each corner to meet the next sheet round a
  bend, with a strip of cap over the lip so the water runs onto it; it runs
  4 cm into the pool below so no seam opens. No polygon offset. The flow
  streaks are drawn in the water's own shader (foam advected along the flow
  vector), not as overlay geometry, so they cannot fight the faces under
  them; caps of neighbouring cells never share a plane (a lower neighbour
  gets a falling sheet, a level one no face).
- **A water sim tab** ("Water sim"): a hex patch nine cells from its middle,
  layers 0 to 15, where a click pours a source, digs or places a block, and
  the water moves by Luanti's (Minetest's) liquid rule,
  `ServerMap::transformLiquidsLocal` and `get_max_liquid_level`
  (`src/servermap.cpp` at `a10dc8d`), as M28 ported it to the hex grid:
  sources and flowing levels, range 8, falling water first, renewal at 3 on
  hex with Luanti's 2 and off as a toggle. The rule moved out of
  `water-flow.html` into `docs/mockups/water-flow/liquid.js`, which both pages
  load (M28 settles every preset as before; its from-scratch check stays at
  nought). Play, pause, step, a tick-speed slider, reset, and presets: a pond
  (three sources renew into a pool), a waterfall down cell steps, and a dam
  whose reservoir pours through when "Breach the dam" digs a gap. Its
  how-it-works text is in a modal and names the Luanti source and licence
  (LGPL-2.1-or-later, `water-flow/COPYING.LESSER`).
- **Descriptive text moved into modals** (CLAUDE.md, "Descriptive text goes in
  modals"): the page's about, each place's description, the light tab's
  how-to, the bestiary's cards (a compact grid on screen, the card in a
  modal), the chart's rules and the engine tab's tables and decisions.

All checked headless (Chromium on SwiftShader, no GPU): every tab opens with no
console errors and one WebGL context; the camera stays put when idle; no
horizontal scroll at 390 px; the sim spreads water from a poured source tick
by tick. Screenshots in `docs/screenshots/mockup-ocean/` (`*-before.jpg` and
`*-after.jpg`, `sim-*.jpg`). Frame cost was not measured.

### 12. Fresh water on cells is the Water sim's, on every tab; M28 folded in

The owner, 2026-11-15: "water that flows - make sure the rivers use that
too, so merge these artifacts together so we get it more consistent". Until
then the river, the dam and the marsh drew their water with hand-placed caps
(`makeCaps`: a fixed level per cell, a fixed flow vector, falling sheets),
while the Water sim tab and M28 ran Luanti's rule and drew it with Luanti's
mesher. Now every tab's fresh water that stands or flows on cells is
simulated by the rule (`water-flow/liquid.js`, unchanged) and drawn by the
mesher (`PBDWater.simWater` in `water-flow/water-look.js`), through one
helper (`cellWater` in `ocean.html`):
- **Still water is sources**, as the world's generated water is: the river's
  spring, its plunge pool and the bed under each deep reach, the beaver pond,
  the marsh pools, the sea's cells at the river mouth. Everything else
  follows from the sources and the cells by the rule, settled from scratch
  (every source queued, ticks until nothing is) before the first frame, so a
  tab opens on running water, not an empty bed.
- **The sea stays the wave surface** (`makeSea`): the open sea, the coast and
  its surf are not cells. At the river mouth the sea is also sources at sea
  level in the rule's patch, as M28's "Channel from the sea" has it, so the
  river runs into it; the mesher leaves those cells to the wave surface (a
  `skip` hook in `liquidMesh`) and draws the river's water spreading over it.
  The coast's rock pools and the cave pocket are sea water and keep their
  caps.

**What the rule asks of a river.** A reach of flowing water runs eight cells
from where it is fed (levels 7 to 0) and dries beyond. A step down feeds the
reach below anew: one layer gives the lip's level plus three
(`WATER_DROP_BOOST` 4, less the one the falling cell loses), two layers or
more the full seven. Measured on the river with the old levels (a Node run of
`liquid.js` on the tab's own cells): the hills' riffles, 8 to 10 m apart,
ran; the meanders, one layer every 22 to 28 m, dried 17 m below each step,
68 of 154 river cells dry. So a river on cells steps a layer at most four
cells apart, or two layers at most seven apart. The meanders now step a layer
every 10 m and the whole river stands four layers higher than its caps did
(the spring at 19, the fall from 14 to 10, the last reach at 1, the sea at
0); settled in 81 ticks, no river cell dry, 1,095 sources (the sea's
included), 230 flowing, 51 of them falling. A deep reach is two layers, a
source under the flowing layer, except its lip (a cell beside a lower reach),
which is one layer deep, so the still water of one reach never touches the
flowing layer of the next and nothing renews down the river. The plunge pool
is three deep; the four-layer fall into it is four whole falling blocks.
The game's river pass (task 8.1) has the same constraint: proposed, written
in `water-flow` design 7f.

**The dam.** The stream comes down its valley a layer every 10 m from a
spring at its head, each dam site in the middle of a reach, a flat meadow
either side for the pond to flood. A dam is solid cells of sticks four
layers over the stream's bed, and the colony leaves the stream's middle cell
one layer short: the spillway. The pond is sources filled from just behind
the dam up to one layer over its lowest cell, rising a layer at a time as
the days pass, so a full pond spills over the spillway and the stream below
runs on by the rule; while the dam is going up the pond spills over whatever
is lowest. "Breach the dam" (as in the Water sim) digs the spillway and the
cell beside it to the bed and lets the rule run live: the pond pours through,
and since a source never drains, it keeps pouring. The player's break on day
30 still drains the pond (it is the history's), and the stream runs through
the gap. Day 40: 105 sources, 246 flowing, settled in 25 ticks.

**The marsh.** Its pools are sources in basins: a source's top is its
layer's full height, so the pools stand at 1 m over their beds and the
hummocks and peat a layer higher, at 2 (they were 0.6 m water against 1 m
hummocks). A pool cell beside the channel, the mudflats or the patch's edge
is a hummock, so every pool holds: 393 sources, nothing flows.

**M28 is folded into M25.** The Water sim tab is the one place for the
playable sim. It carries M28's coast patch (the sea, a beach, a plain with a
dry basin, a ledge and a plateau) and every preset (spring, channel, basin,
pair, runaway, recede) beside its own (pond, waterfall, dam, flat), the
renew toggle (3, 2, off), the range slider, speeds of 1, 4 and 16 ticks a
second, step, reset, the pour, dig and place tools, the arrows' toggle and
M28's "Settle from scratch and compare". Each preset settles in the same
ticks to the same counts as on M28 (spring 22 ticks, 580 sources and 380
flowing, 126 falling; channel 7, 580/22 (1); basin 4, 586/12 (0); pair 13,
581/208 (12); runaway 19, 896/38 (19); recede 53, 579/0) and as before here
(pond 5, 7/30; waterfall 27, 1/452 (182); dam 0, 78/0). `water-flow.html`
is now a short page pointing to `ocean.html#sim`, with the tracker link and
the LGPL notice; `water-flow/liquid.js`, `water-look.js` and
`COPYING.LESSER` stay where they are, since M25 loads them.

Checked headless (Chromium on SwiftShader, no GPU, so the look only, never
the frame cost): every tab opens with no console errors and one WebGL
context, nothing moves when idle, night is lit, no horizontal scroll at
390 px; the river has no dry cell (154 river cells), the fall is three
columns of falling cells at layers 10 to 13, and all 159 arrows on flowing
tops point downstream; the dam holds (its gap columns 7 cells of sticks and
the spill) and breaches (0 sticks, 8 cells of water, settled in 26 ticks);
"Drop a catch" still drops. Screenshots `river-sim-*`, `dam-sim-*` (with
`dam-sim-holds-after` and `dam-sim-breach-after`), `marsh-sim-*` and
`sim-presets-*` in `docs/screenshots/mockup-ocean/`: each M28 preset's
`-before` is M28's own page, its `-after` this tab; `merged-into-m25.jpg` in
`docs/screenshots/mockup-water-flow/` is the page M28 now is.

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
- **3D bodies cost more than cards.** Hundreds of fish are one instanced draw
  per species with the sway in the vertex shader, and the reef one draw per
  kind, so the cost is triangles, not draws: about 0.96 million in the reef's
  default view (decision 11). Judged on the owner's desktop, not here.
- **Fitting the deep to 145 m** makes the twilight zone shallower than Earth's.
  The planet is small (150 m summits), so the scale is consistent.
- **A beaver colony can flood a field.** It never floods a cell the player
  touched; a field the player has tilled is touched.

## Questions for the owner (with screenshots)

- **Q1. Tides.** `coast-low-tide-day.jpg` beside `coast-high-tide-day.jpg`.
  Recommend yes, 3 m from the moon.
- **Q2. Creatures as pixel cards.** Answered by the owner, 2026-11-15: animals
  are 3D in the world; a dropped catch is a 2D item sprite (decision 11). Mark
  it answered in the survey.
- **Q3. The deep fitted to the column.** `light-40m.jpg`, `light-90m.jpg`,
  `light-140m.jpg`, `deep-day.jpg`. Recommend fitting.

## Building it (2026-11-15)

The owner, 2026-11-15, on the roadmap's phase 2 ("advanced foliage and flora
(M13), and an underwater overhaul (M25)"): "continue with next phase while i
rewview". That is the request to build. Q1 to Q3 are still unanswered, so
their recommendations are taken as provisional (recommendation taken, ask
only with screenshots) and each is easy to turn back: the zones are two
numbers, tides are one range, the cards are the movers' drawing. Branch
`claude/underwater-v2`. Land flora, grass and trees are another agent's, and
nothing here touches them.

Order: group 4 (light), then group 7 (water life), then group 5 (tides) only
if the shader-against-core sea test still agrees within 1 mm and hulls still
float, then group 6 in a new generator version if time remains. Group 8 waits
for volumetric water, as decision 9 says.

### B1. What was measured first

Captures of the walker standing on the seabed (`--walk --at`, fixed step,
the M4 Max Mac, `--profile fast`), level and looking up 40 degrees, in
`docs/screenshots/underwater-v2/before-*.jpg`:
- **The 140 m shot cannot be taken on the shipped seed.** The deepest
  seabed the generator makes is 125 m (10.5, -88.0); within reach of the
  same daylight, 103 m. The fourth depth is 123 m, the deepest there is, and
  every depth is the eye's depth on a real floor.
- **Before, 90 m is still lit blue** (the floor reads about (0, 1, 13) of
  255 and the water overhead a clear blue), as the proposal said: one
  absorption fades the light so slowly that the twilight zone never comes.
- **The deep's colour is near black at any depth.** `deep_color` is an
  albedo of 0.04 at most, so the water seen across a distance is dark
  navy even at 10 m; that is the open water's look and is left alone.

### B2. Where the two coefficients go

- `downwell_per_m` (new in `water.ron`, 0.40 / 0.065 / 0.046) is the light
  ARRIVING: the seabed's light at its depth (`planet_surface.wgsl`, which
  used the absorption for this) and the murk's light at the eye's depth
  (`murk` in `water.wgsl`).
- `absorption_per_m` (0.32 / 0.035 / 0.012, unchanged) stays the view's path
  through water, in the composite and in the sheet. The design table's 0.25
  for red is not taken: the shallows were tuned on 0.32 (`water-lit`) and
  the split is the change, not a new red.
- The terrain's uniform keeps its lane: `water_absorption.rgb` carries the
  downwelling now (its only reader was the seabed's light), and its name
  says so in a comment, to keep the Rust and WGSL layouts as they are.

### B3. Turbidity before the environment function

`ocean::environment` (6.1) needs the climate normals and the generator's new
places, so it comes with group 6. Until then the turbidity at the eye is read
off what the game already knows: the water class under the eye
(`fauna::water_class`) and the depth. River 0.12; the sea from coast 0.07 in
the shallows to open ocean 0.022 past the shelf, smoothly by depth; eased
over two seconds so crossing a class never steps. It thickens the view's
path toward the murk colour, in the composite and from the sheet. Reef,
kelp, polar and marsh values wait in `water.ron` for 6.1.

### B4. The zones as a core rule

`pbd_core::ocean` holds the light left at a depth (`exp(-downwell * d)`) and
the zone it puts the point in: sunlit while blue keeps over 15%, twilight to
1.7%, midnight below. A test pins 40 m and 90 m for ocean water and the
"red goes first" scenario, and that both lie above `column::BASE_M`.

### B5. Caustics, shafts, snow and spray

- **Caustics**: on up-facing seabed lit by the sun, a moving web projected
  along the sun, in 0.25 m steps, scaled by the light left at the face's
  depth and faded out by `caustics.fade_m`; none at night (it rides the
  sun's own term, so cloud, shade and night take it).
- **Shafts**: from under the water by day, a few samples along the view ray
  of a slow pattern on the surface traced back along the sun, each lit by
  the light left at its depth; strongest looking toward the sun.
- **Marine snow**: specks on three shells round the eye in a grid anchored
  to the body, sinking, from the twilight zone down, lit by what light is
  left and a faint glow of their own (`snow.glow`) so the midnight zone has
  something in it to judge depth by.
- **Spray and streaks**: from `spray_from_mps` (a gale, 17 m/s) up, foam
  streaks laid along the wind sea's heading and spray on the crests, both
  growing with the wind past it.
- All four are drawn, derived and never saved. Their strengths are
  `water.ron` numbers; zero turns each off.

### B6. Water life, first cut

- Species are appended to `home` in `fauna.ron` (the index is the save).
  Each gains `env` (empty: anywhere its water class fits) and a `mover`:
  `School` (today's), `Crawler` (walks the bed: crabs, lobsters, clams) or
  `Floater` (rides the sea: ducks, otters). Old entries default to
  `School` and no environment, so the eight fish are unchanged.
- Environment tags that need 6.1 (reef, kelp, vent, under-ice) are carried
  but read as "open water" until it lands; the spawn rule today is class,
  temperature and, for a crawler, the bed depth.
- **Protected** species refuse the catch with the reason. The rod's bite
  is only by fish (`Class::Fish`); a crawler is taken by hand and a floater
  never.
- Creatures are drawn as the schools are today (small lit meshes) until the
  pixel cards (Q2) are judged.
- **Seven species in the first cut**, appended after the fish: shore crab,
  spiny lobster, giant clam (crawlers), reef turtle (a protected swimmer),
  sea otter, reed dabbler and pond diver (protected floaters). The rest of
  the bestiary's 41 waits for the environments that place them.
- **Their own spawn cap** (`spawn.max_life`, 6) beside the fish's 8, so a
  raft of ducks never takes a school's place and the starter missions'
  "catch x fish" finds fish as before.
- **Poses, not new movers**, carry the behaviours: a clam within 4 m of the
  eye shuts and opens 6 s after it leaves; a dabbler tips up about 30% of
  the day on its own hashed cycle, paddles off within 9 m and sleeps on the
  water at night; a diver goes under about a third of the time by day.
  Startled is the school's existing `spook`, so a frightened crab scuttles
  at twice its pace as a frightened fish swims.
- **Known limit**: a creature is lit as a lit mesh is, not darkened by the
  downwelling at its depth; the crawlers live in the sunlit zone (to 40 m),
  where that shows least.

### B7. What was left, and why (2026-11-15)

- **Tides (group 5) are not built.** The condition was the shader-against-
  core sea test within 1 mm and hulls still floating; both would hold, since
  a tide is one offset in `LocalSea` and `sea.wgsl` alike. What does not
  hold is the shore: the sheet is drawn only over water cells
  (`water.wgsl`'s cap comes from each water cell's record), so at a 1.5 m
  high tide the last water cell's cap would stand as a lip of water above
  the beach beside it, and at low tide the seabed's tint
  (`planet_surface.wgsl`, from the mean sheet radius) would stay on sand the
  sea had left. A tide needs the sheet to follow the shoreline cells, which
  is volumetric water's job (decision 9). Written up here rather than
  shipped with a lip at every coast.
  *Update, `claude/env-water`:* built. The sheet is listed over every cell
  under high water (the ground hides it where the ground stands higher), and
  the ground's tint follows the tide at the camera, so neither the lip nor
  the tinted dry sand is left. The walker's swim still reads the static sea.
- **Places in the generator (group 6) are not built.** The rules in
  decision 4 read the sea's yearly temperature normals and the plate field,
  which the game does not have yet (`climate-and-fish-maps`); a reef placed
  by latitude instead would be a second rule that the environment function
  then has to disagree with. Group 6 starts with 6.1 once the normals exist.
- **Fresh water (group 8)** waits for volumetric water, as planned.
- **The 4.3 storm capture** is owed: no gale was found to film on the seed
  in the time, so only a test capture with the threshold lowered exists.

### B8. Bodies: the mockup's 3D creatures in the game (2026-11-15)

The owner approved M25 and asked to "commence implementation of all env
changes". The first cut drew every creature as a five-sided cone with a
fin, one colour a species (`docs/screenshots/underwater-v2/life-zoomed.jpg`).
Decision 11 says what they are: low-poly bodies, flat shaded, one colour a
face, read off each creature's sprite. How they go in:
- **One kit, ported.** `crates/pbd-app/src/fish/bodies.rs` is
  `ocean/creatures3d.js` in Rust: the face-coloured kit (rings along the
  body, boxes, prisms, ellipsoids, flat polygons, an icosahedron, three.js's
  own vertex orders so a face's hashed colour lands where the mockup's
  does), and the species table (`SPEC`) with the mockup's colours, sRGB
  hex read as linear for the vertex colour. A model is one unit long, head
  along +X, back along +Y, as in the mockup.
- **The house approach.** The herd's animals (`husbandry/rig.rs`) are a
  rig of parts on pivots, posed and built into one vertex-coloured mesh on
  the CPU each frame. The water life does the same: each species' body is
  built once and cached; each frame a school's mesh is every creature's body
  posed (parts turned on their pivots), swayed, scaled to the species'
  `length_m` and turned along its heading, as today's cones were. A school is
  still one entity and one mesh (the fish rule: what there are many of is
  not an entity). No new shader: the sway the mockup does in its vertex
  shader is done on the CPU here, so the material is a plain
  `StandardMaterial` with vertex colours, double sided, warmed behind the
  loading screen like every other.
- **The animation is the mockup's**: a fish sways from the tail (an eel
  along its length), a ray and a turtle flap, a duck paddles its feet, tips
  up (body nose-down and head down) and tucks its head to sleep, an octopus's
  arms wave, a clam's upper shell opens on its hinge and shuts, a tube worm's
  plume stands out of its tube and snaps in, a crab walks sideways. Each is a
  function of the world clock and the creature's own phase (hashed from its
  id), so nothing is stored.
- **Which body.** The table maps each species id to its model:
  the eight fish to the mockup's `game/fish-*` (from fish.html's
  `buildFish`), the crab to M7's shore crab, the reed dabbler to the drake
  and, for every second bird of a flock, the hen. A species with no model
  falls back to the minnow's body in its own colour.
- **A caught fish stays a 2D item sprite**: the hotbar, the pack and a fish
  dropped on the ground are its item icon, as now (decision 11). Nothing
  in the drop path changes.

### B9. The octopus, tube worms and seals (7.2)

Three species appended after the pond diver (append only; the index is the
save), each with its behaviour as a pure rule in `pbd_core::fauna` beside the
clam's and the ducks':
- **Den octopus** (`octopus`), a crawler of the reef and the shelf's rock,
  1 to 40 m, above 6 C, wild only and not taken on a line (a knife, later).
  `Behaviour::Hides`: out, it crawls the bed near its den (the place it
  spawned); when the eye comes within 6 m it inks and jets home at four
  times its pace; home, it hides (drawn half size, flat in the den) for 12 s
  and until the eye is 8 m off, then comes out. Its colour runs from the
  rock's grey-brown when still to flushed red when it moves (decision 6's
  reef row).
- **Plume worm** (`tube_worm`), a cluster round a vent on the deep floor,
  8 to 30 C at the vent, never moves (a crawler at no pace). `Shuts`, as the
  clam: within 5 m the red plume snaps into its white tube, and comes out
  6 s after the eye leaves. Left alone; it gives nothing.
- **Ice seal** (`seal`), a protected floater of polar and kelp water,
  -2 to 14 C. `Behaviour::HaulsOut`: calm, it lies out on the water still
  (on a floe once sea ice exists, B10); within 9 m it slides in, swims off
  under the water and comes back up 20 s later.
Each has a core test of its state machine against the eye's distance. Until
the environment function lands (B10), the octopus and tube worm would spawn
anywhere their class fits; so the spawner reads the environment from the
same step, and a species tagged `Vent` never spawns outside a vent.

### B10. Places: what the game has, and the minimum to build them

Measured on `claude/env-v2` (2026-11-15) for B7's two inputs:
- **Sea temperature normals: missing, but the pieces are there.** No yearly
  normals are shipped (`climate-and-fish-maps` 3.1 and 3.2 are not built).
  What is shipped is `assets/climate/settled-g6-l5.bin`, a settled level-5
  atmosphere at the start of a year, and `examples/fish_ranges.rs` already
  runs a year and keeps each cell's coldest and warmest daily-mean sea
  temperature (`ground_k`). The minimum is that, written down once.
- **The plate field: missing entirely.** Nothing in the game has plates,
  ridges or trenches.

So the minimum, per `climate-and-fish-maps` design 3 but narrower:
- **`examples/sea_normals.rs`** (a measurement instrument) restores
  `settled-g6-l5`, runs one year of 100 days at the shipped settings, and
  writes per atmosphere cell the coldest and warmest daily-mean sea
  temperature and the share of days frozen (at or below -1.8 C):
  `assets/climate/sea-normals-g7.bin` (magic, level, cell count, three f32
  planes). It is a generator INPUT, so unlike the climate layer's asset it
  is never re-baked: it is version 7's, frozen, pinned by the version 7
  digest test, and `include_bytes!`'d by `pbd-core` so the generator, the
  spawner and the map read the same bytes with no file system. A retuned
  atmosphere later makes a new generator version's file, never this one.
  The climate layer's full normals (`climate-and-fish-maps` 3) stay that
  change's; this is the sea's three planes only.
- **`pbd_core::ocean::plates`**: fourteen plates seeded on the sphere
  (jittered Fibonacci points), each turning about its own pole; a direction
  belongs to the nearest seed after a low-frequency warp, so boundaries
  wander. At a boundary the two plates' motions along its normal say what
  it is: parting is a ridge, closing is a trench. A pure function of the
  seed, cheap, tested for coverage (every plate holds ground, both kinds of
  boundary exist under the sea).
- **`pbd_core::ocean::environment(cfg, direction)`**, decision 4's rules on
  this planet's scale (the body is 4.8 km in radius, so decision 4's
  kilometres are cut to metres, recommendation taken):
  - Reef: under 6 m, coldest sea above 23 C, no river channel within 60 m;
  - Kelp: 6 to 40 m on rocky ground (`rockiness` above its median),
    warmest sea 5 to 20 C (upwelling from the live current is left out:
    the generator reads no live weather);
  - Polar: the sea frozen on at least a quarter of the year's days;
  - Vent: deeper than 90 m within 120 m of a ridge;
  - Trench: deeper than 60 m within 150 m of a trench line;
  - Coast: within two cells of the shore; Open: the rest.
  Read by the generator (pieces and the trench), the spawner (the species'
  `env` tags) and the map.
- **Generator version 7** (`TENEBRIS_V7`, a new world's): version 6 with
  `ocean: Some(OceanPlaces)`. Version 6 is pinned by digest before it lands
  and never edited. What 7 changes:
  - **the trench**: the floor deepened toward the column's floor along a
    trench line (to 140 m, inside `column::BASE_M`);
  - **the pieces**, derived like trees and drawn only on a version-7 world:
    the reef packed per decision 11 (table corals on stalks, branching and
    finger clumps, boulder and brain corals, soft tufts, sponges, fans,
    anemones, turf, sea grass, rubble), kelp stacked to the surface with a
    canopy, black smokers with a glow at the mouth, and floes on polar
    water. They are drawn in tiles round the eye, one vertex-coloured mesh a
    tile, from the same kit as the bodies;
  - **the life**: species with `env` tags spawn only where the environment
    function says (the octopus on the reef, the tube worms at a vent, the
    seal on polar water). A species whose tags are all fresh-water places
    (the ducks: marsh, pool, riffle, estuary) keeps to its water classes
    until volumetric water makes those places. On a version 4 to 6 world the
    spawner does not ask the place at all, so its water life is as it was,
    but for the plume worm, which lives only at vents and so never appears
    there.
  - The land, the biomes and the sea mask are version 6's, so the settled
    climate a version-7 world starts from is version 6's state, shipped as
    `settled-g7-*` (the climate reads the sea mask and the land's height,
    neither of which moves).
- **What is not built, and why: reef, ice, basalt and mussel-bed CELLS.**
  Decision 5 makes reef rock, brain coral, sea ice, vent basalt and mussel
  beds cells. The column renderer's material word is four bits and full
  (`planet_terrain.rs`: "`FORGE`, the LAST code the four-bit material word
  has"), so a new cell material needs that word widened in the column pass
  and its shaders, which is the water agents' ground this week. Version 7's
  reef bed is sand and its vent floor stone, which the column already draws,
  and the coral, the chimneys and the floes are pieces. Widening the word is
  written up here as the next step and not taken.

### B11. The gale spray capture (4.3)

Still owed. `examples/gale_probe.rs` (an instrument) runs the settled
climate a new world opens on through its first day and finds the strongest
wind over the sea each hour: gusts of 19.8 m/s at (-27.30, 33.36) at 11:00
and 21.9 m/s at (55.80, -10.92) at 23:00, each one cell for about an hour,
so the sea (which follows the wind with a lag) is never seen at a gale. The
capture harness also cannot hold the eye on the surface over deep water:
`--walk`, `--swim` and `--fly --height` all put the walker on the seabed
there. Filming it needs a harness option that holds the eye above the
sheet, or a world whose weather has run into a real storm; not done.

### B12. What was built and measured (2026-11-15, `claude/env-life`)

- **Bodies (B8)**: every species has its mockup body; `--life lineup` lays
  them side by side. A capture is a fixed step on the M4 Max Mac with
  `--profile fast`: looks, not frame cost, which was NOT measured.
- **The places (B10)**, measured on 400,000 directions of version 7
  (`examples/ocean_places.rs`): of the sea, open 83.1%, kelp 10.9%, coast
  3.6%, polar 1.2%, trench 0.75%, vent 0.31%, reef 0.23%. The deepest floor
  is 140.0 m (version 6's: 125 m), inside the column.
- **The reef's temperature**: decision 4's 23 C gives no reef at all on this
  planet; its warmest coldest-day sea is 21.9 C (`sea_normals`). 20 C is
  taken (recommendation taken, ask only with screenshots); real reefs reach
  down to about 18 C.
- **Pieces**: drawn in 12 m tiles out to 46 m round the eye, planned off the
  main thread and built one tile a frame. Known limits, each written here
  rather than hidden: a tile appears whole when it comes in reach (no dither
  fade yet, against CLAUDE.md's "no pop-in", so the reef's far edge pops,
  softened by the murk); kelp and soft corals stand still (the mockup sways
  them in a vertex shader); pieces sit on the cell under their own middle,
  so one straddling a cell's edge overhangs the step; the big anemone with
  its clownfish and the reef's anthias and damsels are not built (they
  would be species); the vents' smoke and their light on what is near them
  are not built (the glow is an unlit mesh, no lamp).
- **Generator version 7** is a new world's. The digests of versions 6 and 7
  were taken on the Mac; versions 4 and 5's ground digests already fail on
  the Mac (they were taken on the desktop), so version 6 and 7's ground
  digests may fail on the desktop the same way, while the top-block digests
  hold on both (version 4's does). Another agent adding a generator version
  this week would collide on the number 7; whoever merges second renumbers.
- **A walking test found a trench's edge case**: `walking::tests::toggling_to_walk_over_the_sea_lands_in_the_water_not_on_a_shore`
  stands over the planet's deepest water, which on version 7 is a 140 m
  trench; there, leaving flight moves the walker 1.7 m sideways in the
  first tick (under 1 m over the 125 m open floor, as before). Not found
  yet; the test is held on the deepest open sea (126 m and less) with a
  note, and the trench case is open.

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
