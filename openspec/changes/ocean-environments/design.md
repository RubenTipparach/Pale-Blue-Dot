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

### 13. Rivers and marsh pools in the game: generator version 8 (built, 2026-11-15)

Version 8 is version 7 (the sea's places) with rivers that run down to the
sea reach by reach and marsh pools held by rims, for new worlds only;
versions 4 to 7 keep their digests, and version 8's is pinned beside them.
`pbd_core::rivers`.

- **The stage is a field, from one drainage map.** A column is a pure
  function of its direction, so a river's level cannot come from a walk done
  when a tier is built. It comes from one map of the whole body, built once
  per world (0.1 s) from version 7's ground with no channels in it, read by
  every tier and thread alike: a latitude-longitude lattice of 0.5 degrees
  (42 m at the equator) drained by the priority flood. From every sea cell
  outward, in rising order of the level water would spill at, each land cell
  drains into the neighbour it was reached from. So every land cell drains
  to the sea by construction, down a path whose spill never rises, a hollow
  crossed at its spill. A cell is river where 40 cells or more drain through
  it, below 90 m of spill (springs are the river's heads in the hills).
- **Reaches and the water's top.** A reach runs from a lattice cell to the
  next downstream, wandering up to 7 m either side along a smooth field; its
  water is 6.4 m wide where the valley's floor is under it. Its water's top is the cell's spill less a metre; at any point it
  is that, or a metre under the ground the river cuts there where that is
  lower, so a river never stands over its banks and no levee is raised: down
  a slope the river steps with the ground. The bed is two layers under the
  water (two layers of sources, design 12's deep reach), in the valley
  described below.
- **Flowing water, settled.** The column reads the river's own top one cell,
  two cells, up to eight cells up the river: below a step of one layer it
  carries a flowing sheet one layer over the sources at seven less a level a
  cell; at a step of two or more, a column of falling blocks in front of the
  step, its foot flowing over the pool. That is what `water-flow`'s rule
  settles to from the sources, written as a pure function so every tier
  makes the same, and nothing is queued (decision 5): it moves when a
  player edits beside it.
- **Marsh pools** are version 5's swamp water in its lowest layer, now only
  where the ground 2.9 m round it in six directions stands as high (the
  rim), so every pool is held.
- **Everything that asked for a river asks one test** (`planet_gen::in_river`):
  the city sites' river villages, the sea's places at a river mouth, the
  fish's river class (a version 8 river is a river at any height), the coast
  walk and the capture's locator.
- **The climate** for version 8 is version 7's settled state (as 7's is
  6's): the climate's cells are far coarser than a river. Recommendation
  taken (ask only with screenshots), kept on the lead's fourth pass. The
  valleys change a few biome cells the weather reads, which moved the
  default start's cover from 0.69 to 0.67 (no rain).
- **Fourth pass.** A river is a stream at its spring and widens 1.5 m each
  time its catchment doubles, to 18 m across; under a fall of three layers
  or more the bed is two layers deeper for two cells (a plunge pool); its
  last reach is at the sea's level and runs into it. Only sources are
  generated; the tier's build settles them by the rule (`water-flow` E15).
  Beyond the tier the river is drawn on far cells as the sea is, a still
  sheet at its surface over every cell whose centre is within half its width
  of the line (`spare.w`), dithered against the tier's own water across the
  tier's last 25 m.
- **Version 7's noise channels stay** as the sea's inlets, so off the new
  rivers' valleys a version 8 world is version 7's ground: the city survey
  reads version 7's ground (`TerrainConfig::site_ground`), so the list the
  owner approved for a new world is unchanged and Holbrook is where it was,
  and a new world starts where it always did (the start's search reads the
  same ground). A town's ground stands in for a river in its footprint.
- **The valley.** The bed is flat for 1.5 m either side of the line and
  rises evenly to the ground 8.2 m out, so a channel is never a slot a far
  tier's walls cannot close (the seam report holds at none of 4,000 edges).
- **Held in its valley** (`water-flow` E16, fifth pass): a 3 m levee round
  the valley stands a layer over the reach's water and its fall, and the
  ground under a river, its banks and 15 m past them opens no cave above four
  metres under the sea's level, so no river water drains into a cave and
  runs on underground. Tested: no flowing water outside a river's banks on
  four tiers over falls.
- **Measured.** 8,200 of the lattice's 259,200 cells are river; 2.7 % of
  land samples are within reach of one. World preparation went from 3.4 to
  3.8 s.
- **What it is not.** The river's water is drawn only in the column tier (as
  every pool is); beyond it the carved channel shows its bed. Most steps are
  one or two layers, since the ground is gentle at a cell's scale: tall
  waterfalls are rare. M25's plunge pools are not generated.

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

### B8. The second pass on `claude/env-water` (2026-11-15)

Tides are built (5.1). Rivers and marsh pools (8.1, 8.3) are built in the
third pass as generator version 8 (decision 13). The beaver colony (8.2) is
not.

### B13. Second pass (2026-11-15): no pop-in, the reef's life, smoke, seals, the chart

- **No pop-in** (CLAUDE.md priority 1). A tile is a root carrying the
  field's `Faded` (`cities-in-the-world` 4a), so its meshes go through the
  terrain's screen-door mask (`detail-fade`): wholly masked when it lands
  and in over 0.8 s, out as its middle goes from 34 to 46 m off, and
  dropped only once wholly masked at 52 m. The unlit glow and smoke cannot
  take the mask; they show while the tile is under half faded.
- **The reef's life** (decision 11): the big anemone (220 tentacles) on one
  reef tile in three with two clownfish circling it; a cloud of forty
  anthias over one packed reef tile in two; up to three skyline table corals
  a tile and twice the scattered ones. The fish are ambient, drawn afresh
  each frame within 30 m (not species, never caught).
- **Sway**: soft corals, anemones, grass, turf and kelp are their own mesh,
  bent each frame within 30 m by a wave up their height (7% of it, at most
  0.7 m), on the CPU as the bodies are.
- **Smoke**: each smoker's plume, 36 puffs rising 11 m, widening and
  swirling, hot at the mouth, unlit.
- **Seals on floes**: one floe in nine carries a seal lying out; within 9 m
  of the eye it is gone (slid in). They are decoration of the floe, not the
  `seal` species, whose floaters still lie on the water.
- **The chart**: `pbd_core::map::Texel` carries the place (`chart_at`, the
  environment without the coast), the map's cache is renamed (`-p1`) so an
  older one is rebuilt, and the map's registry gains a "Sea places" layer.
- **The trench handoff (B12)**: looked into, not found. The ship stands
  still when parked over the trench (its velocity is 1e-8 m/s), so the
  1.7 m comes from the walker's own first step there, not from the ship.
  The test stays on open sea.

### B14. Textures on the reef, and M9's fish in the water (2026-11-15, `claude/env-reef`)

The owner, looking at the reef in the game: "reef needs some textures". Then,
of the fish: "also please use proper fish models", refined to "use the fish
species mockup to create proper fish": M9, *Fish Lab and Aquariums*
(`docs/mockups/fish.html`, https://claude.ai/artifact/Ay54k8LEf4sdteFpZkeSsG),
whose rule is that every fish is drawn from a species and a seed.

**Finding.** The reef's pieces (B10, B13) and the creatures (B8) are flat,
vertex-coloured faces with no texture at all: beside the terrain's 32 px
pixel-art tiles they read as smooth plastic (`reef.jpg`, `kelp.jpg`,
`vent.jpg`). The fish are the ocean mockup's six-sided darts, not M9's
fish: no gill line, no scales, no pattern but a few coloured faces, and a
species' shape only by its height and tail.

**What is built** (the plan; the result is recorded under it once built):

1. **One atlas for the water life**, `assets/textures/water_life/atlas.png`
   (512 x 512, sRGB, nearest-point sampled, no mipmaps, as M9's own skins
   are), drawn by a committed generator, `tools/water_life_atlas.py`
   (PIL and numpy; Sweaft-64 palette, Bayer-4 dither, no outlines in the
   world), with a manifest `atlas.ron` beside it naming every rect and
   every fish variant's genome, and `PROVENANCE.md`.
2. **The reef's tiles**, 32 x 32 each, greyscale near white so the pieces'
   palettes stay (the texture multiplies the vertex colour, which keeps
   the kind's palette, the per-face shades and the depth dimming): polyps
   (branching and finger corals), brain grooves (boulders), plate rings
   (table corals), pores (sponges), lattice (fans), tentacle stripes
   (anemones, soft corals), blade veins (grass, turf, weed, kelp), grit
   (rubble, shells, mussels, mats), basalt with sulfide crust (chimneys)
   and ice. The world's art rule holds: no outline, the contrast kept near
   each tile's mean (mean luminance about 0.85, darkest marks about 0.6),
   so a tile reads as surface, not as a sticker.
3. **UVs on the pieces**: each part a piece is built from (one `Kit::add`)
   is box-projected by its face's normal into its kind's tile, at one
   scale for the part (its longest side across the tile), so a triangle
   never crosses a tile edge and nothing needs a repeat. The texture rides
   on the one `LifeMaterial` the pieces already share; there is no new
   material and no new draw call. Every mesh drawn with it carries `UV_0`
   (pieces, swaying pieces, ambient life, schools, the loading screen's
   warm mesh), so there is still one pipeline and the prewarm still covers
   it. Sway moves positions only; the screen-door fade is the field's,
   untouched.
4. **M9's fish**, ported from `fish.html`'s `genome`, `paintSkin` and
   `buildFish` rather than invented again:
   - the body: M9's profile (`PROFILES`: fusiform, compressed, disc, eel,
     flat) with its `profH` and the ray's diamond `rayW`, twelve rings of
     six sides (twenty-two for an eel), a slightly flatter belly, the skin's
     UV the length and the angle round the body as M9's;
   - the tail fin by M9's tails (forked, deep fork, lunate, square,
     rounded), the dorsal fins soft, spiny or tall, the anal fin, an eel's
     ribbon fins, the serpent's crest, the pectoral fins and the ray's whip
     tail, all thin double-sided surfaces with M9's fin-ray tile;
   - M9's block eye (a pale block and its pupil);
   - the skin: M9's `paintSkin` in Python, 64 x 32 (countershading in four
     dithered Sweaft tones, then the species' stripes, bars, spots or
     mottling and the dashed lateral line), and its 8 x 8 fin tile;
   - the seed: each species has four variants in the atlas, each from M9's
     `genome(species, seed)` with seeds 1 to 4 (its length factor, depth,
     width, fin size, pattern density, hue and lightness, pattern seed), and
     a creature wears variant `id % 4`. M9's rare morphs (golden, pale,
     dark, one in 64) are not baked.
   The game's own animation is kept: the tail sway and the eel's wave of
   B8, and a new flap for the ray, whose wings are M9's one flat body
   bent up and down by the distance from its spine (M9's `swim` for the
   flat plan).

**M9 to the game's ids.** The eight fish the game has are M9's first eight
under the same ids: `minnow`, `silverfin`, `perch`, `ray`, `eel`, `reef`,
`deepback`, `serpent`, each with M9's numbers (profile, tail, fins, eye,
pattern, colours, variation). M9's five new species (`bloatfin`,
`sunplate`, `lanternjaw`, `glassback`, `emberback`) are not in the game's
roster and are not built. The reef's two ambient fish, which are not
species, are given M9 definitions in the same form: `darter` (compressed,
rounded tail, white bars on orange, the clownfish over the big anemone)
and `anthias` (compressed, lunate, orange-pink with a gold lateral). The
2D item sprite a caught fish becomes is unchanged; it is the species'
icon, which M9 already draws beside its fish.

**The other creatures** (crabs, lobsters, clams, turtles, otters, seals,
ducks, the octopus, plume worms) keep their B8 bodies, flat-coloured, with
their UVs on a white texel of the atlas; they are not rebuilt in this pass.

**M9 is the library (owner, 2026-11-15).** "add more fish from the reef if
needed, just keep that mockup updated since that represents our "Fish"
library", "and coral and all underwater objects to the fish and aquariums doc
too", "fish and aquariums doc should have at least 1 of everything". So M9
gains the darter and the anthias as species entries (roster, sheet, lab,
tank), icons in its manner (`tools/art/fish_art.py`), and a "Reef & sea
floor" tab that draws one of everything the game's sea has (every reef
piece, kelp, the smoker, mat, mussel bed, the floe, and every water
creature's body) from the game's own meshes, written out by the app test
`export_water_life_models` into `fish/water_life_models.js`, in the game's
own atlas. The page's descriptive text moves into info modals as it is
republished (owner, 2026-11-14).

**Found while capturing: version 8 lost the reef.** On a 1-degree global
scan of `ocean::places::environment`, version 7 has 55 reef samples and
version 8 has 3; none is left within 10 degrees of (13.096, 94.896), where
`reef.jpg` was taken, though kelp, vents and polar sea are unchanged. This
is upstream of this branch (generator version 8, design 13) and is not fixed
here; it wants its own look, since a saved version-8 world keeps what it
has.

**Cost.** A fish body is 221 to 288 triangles (a 12 x 6 body, its fins and
the block eyes; the ray 288), an eel 416 and the serpent 452 (22 rings and
the ribbons), pinned under 500 by a test. Schools are rebuilt on the CPU
each frame as before, now also writing a UV per vertex. The frame cost is
not measured on this branch.

### B15. Generator version 9: the reefs back; the other creatures modelled (2026-11-15, `claude/env-reef`)

**Why version 8 lost the reef.** A reef is kept clear of rivers
(`reef_clear_m`, 60 m). Version 7 asked for a noise channel's *mouth*: the
channel above its threshold where the ground would be land without the
carve. Version 8 swapped that for `planet_gen::in_river`, which is true at
*any* point of a noise channel, and the channels run through the open sea as
well as into the land, so nearly every shallow warm sea was "near a river":
on a one-degree globe scan version 7 has 55 reef samples and version 8 has 3.

**Version 9** is version 8 with `OceanPlaces::V9`: version 7's numbers and
`reef_mouths_only`, which clears a reef only near a channel's mouth (version
7's test, the channel read on the ground without version 8's rivers) or a
version 8 river's own reach. Version 8 is not edited; a world that names it
keeps its 3 reefs. The reef is pieces, not ground, so version 9's altitudes,
biomes and top blocks are version 8's digest for digest (both pinned);
`GENERATOR_VERSION` is 9, and the settled climate g9 is g8's, as g8 was g7's.
Tested: version 9's reef share on the scan is 55, version 7's exactly
(`version_9_has_version_7s_reefs_again`). `reef-v9.jpg` is the old
`reef.jpg` view, reef and all, on a new world.

**The other creatures.** The owner: proper models "instead of placeholders".
The crab, lobster, clam, turtle, otter, seal, the two ducks and the diver,
the octopus and the plume worm are modelled again in M9's manner
(`fish/bodies/creatures.rs`): skinned tubes and tapered limbs instead of
boxes, each wearing a 64 x 32 skin drawn by M9's own `paintSkin` on the
creature's colours and a pattern (`tools/water_life_atlas.py`,
`CREATURE_SKINS`: 19 skins, the crab spotted, its legs barred, the
lobster mottled with a barred tail, the clam's fluted shell and spotted
blue mantle, the turtle's mottled shell and spotted skin, the drake's
vermiculated grey and green head, the hen mottled, the diver spotted, the
octopus mottled, the worm's ringed tube and banded plume). Every part and
pivot keeps its posing name, so the ducks' heads, wings and feet, the
turtle's flippers, the octopus's arms, the clam's lid and the worm's plume
move as before. In the fish's budget: 150 (worm) to 470 (lobster)
triangles, pinned under 500 with every UV in a tile by a test. M9's "Reef &
sea floor" tab draws the same meshes.

### B16. Kelp that reads as kelp (owner, 2026-11-15: "kelp needs fixing")

The owner's reference is the "Kelp forest" frame of the phase-2 montage
(`docs/screenshots/step-videos/phase-2-tech-montage-sheet.jpg` on
`claude/env-v2`, captured at `--walk --at 71.308 -8.196 --time 12`): the
kelp stands as huge flat slabs and ribbons, much wider than fronds, with
hard edges, little shading and a patchy mix of dark and lime greens. Blue
blobs lie on the floor at their feet.

**Finding.**
- **The shape is M9's tank kelp, stretched.** `places::kelp_stand` builds
  M9's 5 m aquarium model (`fish/water_life_models.js`, "kelp"): three
  stipes of flat strips 0.24 m wide, each 0.7 m segment turned 0.6 rad
  from the last, and four flat 1.6 m blades at the top. Stretched up a
  6 to 40 m water column it has no fronds along its length at all. Seen
  from inside the forest, the twisted strips read as slabs and ribbons.
- **The colours cycle by triangle.** A tint is chosen from four colours
  (rust, orange, lime, dark green) every six triangles, so neighbouring
  segments jump between dark and lime: the patchy mix.
- **The blue blobs are coral boulders.** A kelp stand puts a `Boulder`
  piece at its foot half the time, and `Boulder` is the reef's brain-coral
  kind, coloured from the coral palette (blue-greys, teal, pink).
- The light is not at fault. The kelp shares the reef pieces' material, is
  lit by the field and dimmed by depth as they are, and sways (B13).

**The approved look** is M25's kelp forest (`docs/mockups/ocean.html`,
"Kelp forest"; its frond is `tools/art/ocean_art.py` `kelp_card`): a
slender stipe with narrow blades alternating up it, a gas float at each
blade's base, in giant kelp's golden olive (M25's `kelp` ramp), with one
blade in three in the green of its `kelpg` ramp, and a canopy spreading at
the surface.

**Change** (drawing only; where kelp stands is unchanged, so no generator
version):
- **The stand**: two or three stipes, each two crossed strips 0.07 m wide,
  so a stipe reads from every side. Narrow tapered blades (0.85 m long,
  0.12 m at the widest) every 0.6 m up it, spiralling round the stipe and
  angled up about 40 degrees. A small gas float (two crossed diamonds,
  0.08 m) sits at each blade's base. Near the surface, four longer blades
  (1.3 m) lie out flat as the canopy.
- **The colours**: one per part, not cycled per triangle. Stipe `#725244`;
  blades `#ab8465`, with one in three `#05582d`; floats `#c56d25` (M25's
  frond; the green is the deep step of its `kelpg` ramp: the first after
  shot, `kelp-after-first.jpg`, had the brighter `#559642`, which still
  stood out lime in the water). The kelp's `veins` tile still gives the blades their grain.
  Depth dimming and the field light are unchanged.
- **The rocks at its foot** keep their place and shape, but take the
  rock's palette (`#434137`, `#725244`, `#a09f91`), not the coral's.
- **Verified** by captures at the owner's spot, before and after, and a
  test that a stand's blades are narrow (no part wider than 0.15 m but
  the canopy) and that its colours are only the frond's.

### B18. Sea life that vanishes up close, and floes on the beach (owner, 2026-11-16)

The owner, on a cold coast with sea ice: "what the heck are these fish
doing" and "they disappear when I get close to them". Their frame: white hex
floes tipped at odd angles and stacked between sandy, snow-topped terraces,
and a dark rounded shape (a seal) between them.

**Reproduced** at the polar shore of B13, `--walk --at 86.965 -17.827 --time
12 --pitch -10` (`docs/screenshots/sea-life-fixes/`). The walker stands on
sand 0.6 m above the sea, and floes lie all over that sand.

**What was found, each from the code and the capture:**
- **Floes are put on dry land.** `places::plan` puts a floe on every 2.8 m
  lattice point the environment function calls `UnderIce`, and that
  function is coarse (a 3 m lattice and a zone noise): it calls the beach
  and the shore terraces under-ice too. `build_tile` then lays every floe at
  the mean sea radius with 0.6 m of ice below it, whatever the ground there.
  On a beach the terrain's hex columns stand up through the slab, so the
  floes read as tipped and stacked; at low tide (a third of the 3 m range
  below the mean) even the ones over shallow water sit in the sand. The
  slabs themselves are flat: each is built square to its own up.
- **A seal on a floe vanishes at 9 m.** `places::ambient` skips a floe's
  seal in one frame once the eye is within nine metres ("slid in"). There
  is no slide: it is there, then it is not.
- **A swimming seal drops 1.2 m in one frame.** A seal of the schools is
  `HaulsOut`: within its 9 m `shy_m` its pose becomes `Under`, and
  `draw::build` draws it 1.2 m lower at once, under the surface; from above
  the water that is a vanish. It does not swim away either: the startle only
  raises its speed cap, so it keeps paddling round the goal it had.
- **A diving duck vanishes.** A `Dives` duck is `Under` for a third of every
  14 s cycle, and `draw::build` skips a creature that is `Under`: gone in
  one frame, back in one frame.
- **The reef's ambient life pops at 30 m.** A tile draws its anthias cloud,
  clownfish and floe seals only while its middle is within `NEAR_M` (30 m),
  but the tile itself is whole to 34 m and fades out from 34 to 46 m. The
  life appears and goes in one frame at 30 m, in the whole part of the tile.
  The sway snaps to rest at the same line.
- **Schools appear and go in one frame.** A school is spawned 20 to 60 m out
  and dropped at 90 m with no fade: a duck or a seal on the surface 20 m off
  appears whole in one frame.
- Not the cause: frustum culling (Bevy 0.18 recomputes a mesh's bounds when
  its asset changes, and every school mesh is rebuilt each frame), the
  camera's near plane (creatures are drawn at 2 m in the lineup), and the
  fishing line (only a splash scatters fish, and they swim, not jump).

**Change** (drawing and the creatures' state; no generator version, since
floes are pieces drawn from the plan, not cells, and nothing is saved):
- **Floes float or are not there.** A floe is laid only where the water
  under its middle and its six corners is deep enough for its 0.6 m of ice
  at low water (the mean less a third of the tide's range); elsewhere,
  beaches and shore terraces included, there is none. The ones left still
  ride the tide.
- **A seal on a floe slides off.** When the eye first comes within 9 m, the
  seal slides forward off its floe's edge and into the water over 1.2 s,
  then swims away from the eye under the surface, visibly, and stays off
  the floe until the eye has been gone 20 s; then it climbs back out over
  1.2 s. Each step is a moment in time, keyed to when it started.
- **A seal of the schools dives and swims off.** Each creature keeps an
  `under` share, 0 on the surface to 1 dived, eased toward 1 over 1.2 s
  while startled and back over 2 s after; it is drawn that share of 1.2 m
  down. A startled floater (seal, otter, duck) also turns its school's goal
  away from the eye, 12 m on in water that fits it, so it is seen leaving.
- **A diving duck dives.** Its `Under` window ramps: it sinks nose-first
  over its first 0.6 s and rises over its last 0.6 s, drawn under the water
  in between, never skipped.
- **Ambient life is drawn while its tile is drawn**, and so fades with the
  tile's screen-door mask from 34 to 46 m. The sway eases out from 26 to
  30 m rather than snapping.
- **Schools fade in and out.** A new school fades in over 0.8 s through the
  same screen-door mask as the sea's places; a school past 90 m fades out
  over 0.8 s and only then is dropped.

**Verified** by: core tests that a startled seal's `under` share moves no
more than its rate allows in a step and that a startled floater's goal is
away from the eye; app tests that no floe is laid over ground above its
draft at low water, that a floe seal is drawn on every frame of its slide,
and that a diver's submersion never jumps; captures before and after at the
polar shore and the `--life` approach sequence at 30, 20, 12, 8, 5, 3 and
2 m; READY with no new `LATE_PIPELINE`.

**Built** (2026-11-16, `claude/sea-life-fixes`): as above. The first
flee only aimed straight away and pulled at the goal's weak 0.45, and the
seal stayed about 5 m off while it dived; it now tries bearings off the
straight line and nearer reaches for water that fits, and pulls with a
lure's weight, and swims from 5.6 m to 17 m in 6 s
(`docs/screenshots/sea-life-fixes/seal-approach-trace.txt`). Known limit: a
seal of the schools rides the water's surface and knows nothing of the
floes, so in a dense floe field it can lie half inside one; hauling it out
onto a floe needs the floes in the water the schools read, and is not done
here. The `--life` approach ends at 5 m at the polar shore: no water nearer
is deep enough for a seal.

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

### B17. Under a river's water (owner, 2026-11-16)

The owner: "problems with submerging in rivers"
(`docs/screenshots/rain-artifacts/owner-river-43.webp`). Down in a river
channel the river's sheet hangs over the eye as a thin translucent layer,
and the view under it reads as dry air: no fog, no tint, no light.

**Reproduced** (`docs/screenshots/river-fixes/before-under.jpg`,
`before-up.jpg`): a version 10 world, a river 60 m from the spawn
(`--fly --at 28.2500 0.7500 --height 0.8`: the ground at 63 m, the river's
water's surface, as the column reports it at the eye, at 65.8 m, the eye at
64.6 m).

**What decides "under water" today, measured in the code:**
- The eye's water is the column's: `publish_eye_water` asks the column at
  the eye (`Column::water_surface`), which knows river and flowing water,
  and `submersion` turns it into the tri-state `fx.y`. In the river it is
  1, fully under: that is why the river's sheet is drawn at all from below
  (`water.wgsl` discards the back of the game's own water unless `fx.y` is
  over 0.75), and why audio already muffles there.
- The composite that draws the underwater view (`compose` in `water.wgsl`)
  then measures everything against the SEA's sphere, `planet_center.w`:
  how deep the eye is (`cam_gap`), which pixels are under water (the
  waterline mask `pixel_alt`), how far a sky ray travels to the surface,
  and the shafts' and the snow's depth. A river stands over the sea, so
  every pixel is "above the water", the mask is nought and the frame is
  passed through untouched. The sea worked only because its surface IS
  that sphere.
- `submersion` also adds the sea's swell at the camera to the surface,
  which a river does not have.
- The near shower (`weather.rs`) stops only under the sea's level.

**The fix: the eye's own surface.** The water view carries the surface of
the water the eye is in (`sea_frame.y`, a spare lane: nought means the sea
and the sphere as before). Every underwater term in `compose` and in
`under_water_light` reads that surface instead of the sea's sphere when
it is set: depth fog and colour by how far the view travels in the water,
the turbidity, the sky seen through the surface, the shafts and the
snow by depth under it, the distortion. The sheet's own from-below path
already measures the murk from the sheet it draws, so the surface from
below is the sea's (Snell's window, the murk past it). The swell is added
only to the sea's surface; a river's is where the column puts it. The near
shower stops when the eye's water says it is under.

Everything else that asks "is the eye under" already reads `fx.y` or
`EyeWaterState` (the clouds, the rain, the lens, the emerge drips, audio),
so it follows.

**Swimming** reads the column already (`walking::Sea::state`, `water-flow`
E10): a walker in a river floats, swims and climbs out as in the sea. A
test pins it.

**Limit.** The mask is a sphere at the river's surface round the planet's
centre, as the sea's is. A pixel of ground lower than the river's surface
but seen past the river's end (down a fall, in the air) counts as wet. The
river is narrow and its reach ends in a fall; the eye under it sees the
fall's own sheet first. Written down, not fixed.

**Second pass, the waterline (owner, 2026-11-16).** "underwater ground tint
is happening before the water line", and "the land right above the water
right there is also getting that subtle twisty effect from being
underwater". Two faults, measured in the code:
- The composite's waterline mask, `1 - smoothstep(-band, band, pixel_alt)`,
  reached `partial_band_m` (0.8 m) ABOVE the surface, so ground just over
  the water was fogged, tinted and wobbled. With the eye's own surface now
  a river's, every eye at a river's waterline (the straddle) showed it on
  the banks. The mask now fades in over the 0.1 m UNDER the surface
  (`WATERLINE_EDGE_M`) and nothing above the surface is ever wet; what lies
  under a crest over the mean sea is behind the sheet, which draws its own.
- The terrain's own underwater tint, darkening and caustics
  (`planet_surface.wgsl`) measured every column's water against the sea's
  tided sheet unless the column's water stood under the sea's level: a
  river's bed was never tinted, and a river's edge near the sea took the
  tide's line rather than its own. A column's water over the sea's level is
  now its own surface, and that surface is the water's own height, a
  flowing top included: the record carries how far the top layer's water
  stands under the layer's top in eighths (`WATER_SHORT_SHIFT`), so the
  tint stops within 6 cm of the surface, where it stopped up to a metre
  over a thin film. Rain rings follow (they stop where the tint starts).

Captures: `before-skim.jpg` and `after-skim.jpg` (the eye at a river's
surface), `after-bank.jpg` (from over the bank).

### B18. The river's water from inside, third pass (owner, 2026-11-16: "rivers not validated")

The owner, in the river at 28.250 0.750 (65.6 m, 2.6 m over the bed, by
day under cloud): "I see rain shader at the bottom; I dont see water surface
above the player; that gradient looks like an incorrect implementation of
being underwater?", and on the bright strip: "you must be submerged at just
the right height, if not 1 block try 2 blocks".

**Reproduced** at that spot, and in the sea at about one and two layers down
(`docs/screenshots/river-fixes/third-pass/`), with a capture-only camera
path (`PBD_EYE_PATH`, `desktop/eye_path.rs`) that moves the eye through the
surface over 720 frames at 60 fps with its height against the water burnt
into a corner. Measured with a temporary log of the tri-state:

1. **No surface overhead.** The straddle band, 0.8 m, holds the sea's swell.
   A river has none, yet its eye was "straddling" for the 0.8 m under its
   surface, and the game's own water discards a face seen from below by a
   straddling eye. The owner's eye, 0.2 m under, saw no surface.
2. **The surface the eye was measured against was not the one drawn.** A
   flowing top is meshed through its corners (Luanti's corner rule): up to a
   layer's share over the cell's own level where a neighbour is full. The
   eye's water was the cell's level, so for the centimetres between the two
   the eye was "dry" under a drawn top: the face was discarded and nothing
   stood overhead.
3. **The flat gradient, and the bright strip.** With the eye under, the
   composite still masked wetness by a sphere at the eye's own surface. A
   river stands at different heights cell to cell, so a wall under a
   neighbour's higher water was dry and bright; and at the sea, one layer
   down, the bank between the drawn sheet (0.5 m under sea level, plus the
   tide) and the bank's top was seen past the sheet's edge, dry: the owner's
   bright strip at the waterline (`before-sea-1-layer.jpg`).
4. **Rain rings on the bed**: not reproduced at the spot after the first
   pass (the terrain's rings stop where its own tint starts, and the bed's
   column now carries its water); the videos show none under the surface.

**Fixes.**
- Inland water has no straddle band (`INLAND_BAND_M` = 0): the eye is under
  a river's water as soon as it is below the drawn surface.
- The eye's surface is the DRAWN top (`water_flow::drawn_top_m`, the mean of
  the corner levels `cell_pieces` meshes), so the decision and the picture
  agree to the centimetre.
- A straddling eye draws the face it is under from below.
- With the eye under, everything not hidden by a sheet is seen through
  water: the composite's mask is gone for an eye under, as it should be (the
  sheet draws what is above the surface from the scene before the pass).
- An eye a hand over the surface looking down: the sheet nearer than the
  near plane is clipped; the composite fogs what lies under it along the ray
  from the surface (`NEAR_SHEET_M`).

The underwater look is the sea's own model throughout (the same murk,
absorption, shafts and the terrain's downwelling darkening); what differed
was where it was applied.

**Validation**: videos crossing the surface at the owner's spot, a river
bank, the sea's shore and where waters meet, before and after, with every
frame within 0.3 m of the surface stepped and listed (the report's issue
list); numeric junction checks in `water_junction_tests`.

**Where waters meet** (B18, the junction check). The check runs on 5 mouths, 3 tributaries and 3 falls from the generator's rivers, on a real tier. It samples every metre along a line from upstream to 40 m past the junction.
- **Mouths: fixed.** Before the fix, every mouth drew two surfaces in 22 to 37 cells: the sea's sheet under the river's own water, wherever the bed is below sea level. The sheet is now marked dry wherever water stands over the sea's layer. All 11 lines read no double surface and no flip in the eye's decision.
- **Steps are kept.** The steps along a line are the river's own: riffles under a metre, and falls of 4 to 11 m at the mouths' cut. They are not seams.
- **Not built.** Lakes are not generated. Marsh pools do not join rivers. No fall drops into the sea. Poured water meeting a river and the tide against a reach have no generated instance to test on; the tide's sheet is the one the eye now follows.

**Second review of the videos (2026-11-16).**
- **A dark sky over the water.** The sky darkened with the eye up to half a metre over the sea. The tri-state took the column's own sea top as the surface, which stands half a metre over the drawn sheet, and added the tide and the swell to it. So an eye over the drawn, tided sheet read as under, and the whole sky went to murk. The fix: one function, `eye_surface_radius`, now gives the surface that the tri-state, the composite and the capture label all judge the eye against. It is an inland water's drawn top, or the sea's sheet lifted by the tide and never by the waves. A pool in a cave below the sea's level is inland water too.
- **The capture label** now uses that same surface and the same eye rule (`eye_water_at`), for this frame's eye.
- **The river reads darker than the sea.** I measured frames at the same depth (`videos/same-depth-sea-vs-river.jpg`). The open water is the same in both: (6, 29, 43) against (6, 27, 37) at 1 m down. What differs is the bed. The sea's is sand on an open shore. The river's is dirt, as the owner asked, in a channel whose walls shade it. A capture with the water composite switched off shows the same gap in the terrain's own shading, so the water model is the same in both. Matching the brightness would mean a lighter bed (sand or gravel) for river beds. That is the owner's call, so it is not made here.
- **The riffle.** At a step, the eye a few centimetres under a thin cell's drawn top can see out through that cell's side face, over the lower reach beyond. That frame is lit as water seen from inside, which is what it is. It shows only on the capture path, which follows the water's height as it jumps up and down at each step.

### B19. The waterline from just under, the river's ceiling, the speckle (owner, 2026-11-16)

The owner, standing in shallow sea at `52.053 -118.367` (the eye at -0.4 m):
"the water line shades above it", with a dark blue band over the bank and
sand above the water from the waterline up, the left bank's base dark teal
"as if under water", and bright bands on the far side. Under a river at
`56.322 -117.795`: "getting better, but still lots of work to do": a dark
navy ceiling with blotches, a band of speckled noise at the far end of the
channel, and sand rims at the channel's top edge that look like they float.

**Reproduced** with the capture path (`PBD_EYE_PATH`) at both spots, the
eye held at +0.15, +0.05, -0.03, -1 and -2 m against the water at four
headings, at the default, low and high tide
(`docs/screenshots/waterline-3/`). Measured in the code:

1. **The dark band is the surface seen from just under it.** With the eye a
   few centimetres under the sea's tided surface (the owner's -0.4 m is
   within a few centimetres of the -0.42 m sheet; the swell takes the eye
   across it), every ray that rises less than about 40 degrees meets the
   surface outside Snell's window. The sheet's from-below path paints all of
   it the flat murk colour (`murk`), so the band of rays just over the
   horizon, exactly where the bank above the water would be, is a flat dark
   navy band starting at the waterline, and the bank under it is (rightly)
   seen through water: the "dark teal base".
2. **The bright bands are a hole in the surface.** The sheet nearer the eye
   than the near plane (0.2 m) is clipped. An eye 3 cm under sees every ray
   rising more than about 9 degrees cross the surface inside that distance,
   so the top of the frame looks through the hole: the sky nearly clear (the
   composite fogs a sky ray by its 3 cm of water) and the trees as dark
   silhouettes, over the navy band.
3. **The sea's sheet flips per fragment.** The sea's `below` test compares
   the eye's radius with each fragment's, waves included. With the eye
   within the swell of the surface, a far crest over the eye's height is
   drawn as if seen from below (murk) while the eye is above the water: a
   dark band at the far waterline that comes and goes with the waves.
4. **The ceiling: navy with blotches.** Outside Snell's window the surface
   from below is total internal reflection: it mirrors what is under the
   water, the bed and the walls. The game paints it the murk colour, which
   in a shallow river with a lit bed and lit walls is far darker than
   everything round it. The blotches are Snell's window cut by the waves'
   normals into bright patches. The sea shows the same model; it reads
   better there only because the open sea's fog is the same colour.
5. **The floating rims.** The channel's walls are lit to their tops, and
   immediately over them is the flat murk of the ceiling: a lit wall that
   ends against a dark sky reads as a slab floating, where in the water it
   would meet its own mirror image at the surface.
6. **The speckle.** Two terms point-sample a pattern finer than their
   sampling, so the far end of the channel turns to per-pixel noise:
   - the shafts march 10 samples, evenly spread over up to 40 m and offset
     by a per-pixel random jitter, through a bright pattern about 4.5 m
     across: 4 m between samples, so every pixel lands on a different part
     of the pattern;
   - the caustic web on the bed is snapped to 0.25 m and has no footprint
     filter; past a few metres a pixel spans several of its blocks.

**Fixes.**
- **The surface from below mirrors the water** (fixes 1, 4, 5). Outside
  Snell's window the sheet shows the underwater scene reflected in it,
  traced in the screen history the sea already uses for its reflections
  (`sea_ssr`, `lighting-model` L8), with the extra water the reflected light
  crosses on its way. Where the trace finds nothing, what the eye sees in
  the mirrored ray's direction 25 m out, fogged for the leg from the eye
  up to the surface, if what the eye sees there stands within those 25 m
  (past them, the sky over a river's lower reach was mirrored as a sliver);
  otherwise the murk as before. The waves tilt the mirror a quarter as much
  as the window (at their full tilt the grazing mirror swirled), and past
  6 to 20 m from the eye the mirror gives way to the murk (the trace's hits
  and misses alternated pixel to pixel there, a grain). Only a face that
  faces up mirrors: the side face of a step or a fall stays the murk (the
  first cut mirrored them too, a bright slab at the channel's far end).
  Inside the window the sky, as before; the window's edge is the same
  Fresnel ramp. The walls now meet their mirror at the surface, and the
  ceiling is as bright as the water under it.
- **No hole near the eye** (fixes 2). The composite stands in for the
  clipped sheet: a ray from an eye under the surface that crosses it within
  `NEAR_SHEET_M` is shaded as the sheet would shade it, the mirror outside
  the window and the view through it inside. Its mirror is the eye's own
  view in the mirrored direction, not a trace: a trace from a point a few
  centimetres from the eye starts behind the near plane, and one frame of
  a crossing came back as a grid of misses.
- **The sea's side is the eye's** (fixes 3). A sea fragment more than a few
  metres away is seen from below only when the eye is under its own
  surface (`eye_water_surface`); within those metres the wave under the eye
  still decides, so a crest passing over the eye is seen from below.
- **No per-pixel noise** (fixes 6). The shafts' samples take one offset
  for every ray instead of a random one per pixel, so neighbouring pixels
  sample neighbouring points; they are packed toward the eye (quadratic
  spacing), and each fades to the pattern's mean where its stretch of the
  ray outgrows the pattern, so the slices never show. The caustic web fades
  to its mean where a pixel's footprint outgrows its 0.25 m blocks. Fading
  every sample but the first (tried) took the shafts out altogether, a
  dropped effect; this keeps them.

**Not changed.** The river bed's material and colour (the owner's call, task
12.5's side-by-side). The look of the sea seen from above.

**The capture items from B18.**
- **Mouth f804-808** (the eye "above" for 5 frames): the capture path
  teleported the eye 40 m, and for the frames the tier took to serve the new
  cell the eye's water was `Unknown`, which reads the sea's height field.
  That is the capture tool; a walker cannot move 40 m in a frame. The
  validation videos of this pass never teleport.
- **The tributary riffle f512**: the capture path followed the water's
  height up a step, so the eye was a few centimetres under a thin cell's
  top and saw out through that cell's own side face. The picture was right
  for where the eye was; the path was the problem. This pass's paths are
  fixed in height and place through each crossing (owner, 2026-11-16: "if
  you smooth the up and down test for water line, then that might be a
  better test").

**Validation.** Before and after MP4s at 1080p60 at the owner's two spots,
a sea shore and a river mouth: one steady descent and one steady ascent at
0.1 m/s through ±0.4 m of the surface, the place fixed, the eye's height
burnt in. On Drive as `waterline3-<place>-<before|after>-1080p60.mp4`;
contact sheets and the frame-by-frame list in
`docs/screenshots/waterline-3/README.md`.

### B20. Kelp that does not hammer the frame (owner, 2026-11-16)

The owner, looking across shallow sea from just above the surface toward
jungle islands: "kelp forest is HAMMERING PERF", "FIX THEM, they dont need
that MANY POLYGONS". A dense field of stands to the edge of the view, each a
dark stipe with a star of pale blades lying out on the surface.

**Measurement instruments** (code whose only job is a number this section
needs; no behaviour changes with them unset):
- `PBD_PASS_OFF=kelp` lays no kelp stand in a sea-places tile (its rocks
  stay), so a held view can be priced with kelp and without on the same
  exe, in the same sitting.
- A `SEA_PLACES` log line, once a second: the tiles drawn, the triangles in
  them, the kelp's share, the swaying vertices rewritten that frame, and
  the main thread's mean time in the sea places' system.

**Finding** (the `SEA_PLACES` line at the B16 spot, `--walk --at 71.308
-8.196`, the Mac; the bed 9 m down):
- **625,011 kelp triangles** are drawn round the eye, 91% of the sea's
  places (686,211), in 71 tiles out to the 46 m fade.
- **1,875,033 vertices are rewritten on the CPU every frame** for the
  sway: every tile within 30 m rebuilds its whole swaying mesh and Bevy
  uploads all of it again (position, normal, UV and colour, about 90 MB a
  frame).
- **A stand is about 575 triangles in 12 m of water** (420 at this spot,
  1,935 at 40 m): B16 drew M25's frond as geometry, two or three crossed
  stipes a joint every metre, a 5-triangle blade and two crossed floats
  every 0.6 m up each, and a canopy of four 5-triangle blades a stipe. And
  the stands stand every 2 m at 70%, 0.175 a square metre.
- **The approved mockup is a couple of polygons a plant** (the owner,
  2026-11-16: "kelp forest in the mockup was like just a couple polygons
  from what I remember"). M25's kelp forest (`docs/mockups/ocean.html`,
  `sceneKelp`) draws each stand as two crossed cards of its frond texture
  (`card-kelp.png`, 16 x 64, cut out) for every 4 m of water, stretched to
  the surface, and one card of the same frond lying flat on the surface
  1.4 m off to one side: **14 triangles in 12 m of water**. Its stands sit
  on 58% of the 2.833 m hex cells, **0.083 a square metre**.
- **The game had drifted from the mockup's look.** B16 rebuilt the frond as
  slender geometry and gave the canopy four long blades a stipe, which is
  the "star of pale blades lying out on the surface" the owner saw; M25's
  canopy is one frond card lying on the water.
- Not the cause: shadows (Bevy's are off; the sun's cascades draw the
  planet and the towns only), reflection or refraction passes (none draw
  the sea's places), or a prepass (none).
- The kelp's tiles are drawn with `NoFrustumCulling`, so the half behind
  the eye was drawn too.

**Change** (drawing only; where kelp stands is a derived piece, never
saved, so no generator version):
- **The stand is M25's**: two crossed cards a 4 m of water and one card
  flat on the surface 1.4 m off, `4 * ceil(depth / 4) + 2` triangles, in
  M25's own card (`assets/textures/water_life/kelp_card.png`, a byte copy
  of `card-kelp.png`), cut out at half alpha, both faces, nearest
  sampling. The vertex colour carries the depth's light as before; the
  card carries the frond's colours.
- **M25's density**: a 3 m lattice at 75%, 0.083 a square metre. The rocks
  at a stand's foot stay, one stand in two.
- The kelp is its own mesh in its own material (the field lights it as it
  lights the reef; its pipeline is warmed on the loading screen by a
  hidden card, as the water life's is), and it is frustum culled.
- The sway is unchanged (the same wave, to the same 30 m), on the far
  fewer vertices.
- No distance LOD: at 14 triangles a stand there is nothing left to cut
  with distance, and none to cross-fade.

**Measured** (`docs/benchmarks/2026-11-16-kelp/`, the Mac, release, a held
view on the bed in the forest, before, after and before without kelp
interleaved, two sittings): kelp triangles round the eye 625,011 to 8,974;
vertices rewritten a frame 1,875,033 to 26,922 (about 90 MB to 1.3 MB of
upload); the sea places' system on the main thread 7 ms to 0.16 ms a frame.
Frame time was not measured cleanly: other sessions' games and builds ran on
the Mac through both sittings, and the repeats spread 19 to 104 ms at p50.
The sitting is to be repeated on a quiet machine. Stills before and after,
from 1.6 m over the water and on the bed, are in
`docs/screenshots/kelp-perf/`. Known, and not changed here: the stands are
cut to the mean sea, so at low tide the canopy card lies above the water, as
B16's canopy did.

### B21. No mirror on the water's underside; the split frame (owner, 2026-11-16)

Two owner reports from the playtest build:

- **The split frame.** "some weirdness happening here when I'm partially
  submerged camera", in shallow sea at `39.566 14.326`, the eye at -1.4 m,
  looking level with the eye at or just under the surface. The top of the
  frame is a flat dark navy, a hard-edged trapezoid of bright teal sea floor
  floats in the middle as if the floor were drawn upside down over the
  waterline, a band of horizontal streaks and smears runs across the middle,
  and the lower half is the real sea floor.
- **The mirror from below.** "do we really need SSR from bottom of the
  water?", "I dont think it looks that good", under the sea at
  `40.455 14.582` (-3.4 m) looking up and ahead in rain. The underside of
  the surface is a dark mirror of the scene under the water, with rain
  rings, fading into a speckled band toward the horizon.

**The owner's call:** remove the reflection on the underside of the water.
From below, the surface shows Snell's window (the sky, refracted, straight
up) and outside it the water's own murk colour, the rain rings and the
waves only a gentle shading on it. No mirrored scene and no SSR, at any
REFLECTIONS level.

**Cause, both reports: B19's mirror.** B19 made the sheet seen from below
mirror the water outside Snell's window (`under_mirror` in `water.wgsl`),
called from two places:
1. the sheet's from-below path in `fragment` (the `if (below)` branch):
   a screen-space trace of the reflected ray through last frame's colour
   (`sea_ssr` over `ssr_history`), and where it missed, the history's texel
   in the mirrored direction 25 m out, faded to the murk from 6 to 20 m;
2. the composite's stand-in for the sheet the near plane clips (`compose`,
   `NEAR_SHEET_M`), the second form only.

With the eye just under a shallow sea, every ray rising less than about
40 degrees meets the surface outside the window, so nearly the whole upper
half of the frame is the mirror:
- **the trapezoid** is the sea floor in front of the eye, found by the
  mirrored ray in the history and drawn over the waterline, upside down and
  hard-edged where the trace's hits end;
- **the streaks** are the trace under the waves: the normal is bent a
  quarter of the waves' tilt, which is still enough to swing a grazing
  reflected ray across rows of the history, and hits alternate with misses;
- **the flat navy** is past `MIRROR_FADE_M` (20 m), where the mirror gives
  way to the murk.

The rain report's speckled band is the same trace's hits and misses at the
6 to 20 m fade, the rings bending the mirrored ray.

**The fix.**
- **`under_mirror` is gone**, with its two constants and the trace's
  `sea_ssr_t`. The sheet from below is Snell's window inside (unchanged: the
  scene over the water, refracted, through the water between) and outside
  it the murk, `murk` at the eye's depth, which is also what the composite
  fogs the far water to, so the ceiling meets the far water with no seam.
- **The waves and the rings shade the ceiling gently** (`under_ceiling`): a
  facet tilted toward the eye faces nearer the window and takes a little
  more of its light, one tilted away a little less, at most 12% of the murk
  either way. Only a face that faces up; a step's or a fall's side face
  stays the plain murk, as in B19.
- **The composite's stand-in** for the clipped sheet is the murk outside
  the window and the view through it inside.
- **No setting turns it back on.** The REFLECTIONS levels (`reflection-quality`)
  keep governing the reflections seen from above; nothing under the water
  reads the screen history, at any level.

**A third report, the same day: the dry beach tinted on a jump.** "dry
ground had the weird water blur pass that we fixed earlier. This only
happens when you jump above water", "you can see the subtle sand tinted blue
green": standing or jumping on a dry sand beach beside shallow water, the
whole beach in front takes a faint underwater tint and wobble.

*Cause.* The composite's waterline is `eye_water_surface()` in `water.wgsl`,
which reads `sea_frame.y` and falls back to the sea's MEAN sphere
(`planet_center.w`) when it is nought. `sea_frame.y` was
`eye_surface_radius(..).unwrap_or(0.0)` (`planet_water.rs`), nought for an
eye in air. An eye standing on a beach under the high tide's reach (ground
plus eye height under the sea's level plus `tide_high_m`, 2 m) is "over the
sea's water" (`eye_water_at`'s second arm), so the composite measured from
the tided surface; a jump lifts the eye out of that reach into air, and the
composite measured from the mean sphere instead. At low tide the mean lies a
metre or so over the tided water, so the dry sand between the two, a whole
beach's strip, was taken for under water for the frames of the jump: tinted
by the travel under the mean and wobbled by the distortion.

*Fix.* `composite_surface_radius`: the eye's water's surface where it has
one, else the sea's tided surface, never nought, so the composite's
waterline is the tided sea whether the eye stands, jumps or flies. The
sheet's side test (`sea_eye`) reads the same lane and gives the same answer
it gave per fragment for an eye in air. Pinned by
`a_jump_over_a_beach_keeps_the_composites_waterline_at_the_tide`; shown in
the `jump-beach` video at low tide, three jumps at play speed and a slow
rise through the reach.

**What it costs.** It removes work: per pixel of the underside, a trace of
up to 32 steps through the history and one more history read, and in the
composite one history read for the near rays. It is measured with a held
under-water view (`--frame-log`, release build, the two builds interleaved)
when the Mac is quiet; otherwise it is owed (tasks 15.3).

**Not changed.** The look from above, the shafts, the snow, the window's
edge (the same Fresnel ramp, 0.55 to 0.75), the river bed.

**Validation** (the waterline videos' method, B19). Before and after MP4s
at 1080p60 at the owner's two spots, the place and the look fixed, the eye
going from 0.6 m over the surface to 0.4 m under it at 0.1 m/s, on down,
and back up through it, its height burnt in. On Drive as
`underwater-waterline-<place>-<before|after>-1080p60.mp4`; contact sheets
and the frame-by-frame list in `docs/screenshots/underwater-waterline/`.

**Validated** (2026-11-16, `docs/screenshots/underwater-waterline/README.md`):
at the owner's shallow-sea spot the before video shows the split frame for
every frame under the surface (the mirrored floor over the waterline, the
streak band); the after video shows the floor under water and the murk
ceiling over the waterline, no mirror and no streaks. Under the sea in rain
the dark mirror and its speckled band are gone; the rings shade the murk.
On the beach the before video's beach band steps 2.5 to 3.1 (of 255) in
colour at every flip into air and back; the after video's steps none over
1.0. **Open:** the murk ceiling is darker than a bright shallow floor; a
lighter ceiling would be a look change, for the owner with the sheets.

### B22. Holes in the sea at caverns beside it (owner, 2026-11-16)

The owner, on a beach at `41.636 11.664` (1.6 m up, on the column tier):
"theres some holes here in caverns next to the ocean". A narrow channel
cut into a grass and dirt bank, a small cavern under the bank's overhang
open to the sea, and the sea's surface broken in both: patches of it
missing, the stone and cobble walls and floor showing through as stepped,
hard-edged blocks, worst under the overhang and at the cavern's back.

**Reproduced.** A measurement (`sea_cave_report`, an ignored test that
builds the fine set at a place and maps, metre by metre, what is drawn over
each cell's water at the sea's top layer) at the owner's spot, and a scan of
39 coastal places round it and further south, then captures with the eye
path at the default and the high tide (`--tide high`). The owner's place is
a cave system of dry pits and roofed air pockets (`D`, `C` on the map) two to
twelve metres west of the beach, whose floors are 2 to 5 m under sea level;
the sea's own caves (`W`: the sea's water under a roof) are as common along
other shores (`35.952 18.251`, `40 10`, `44 6`). Two causes, one family: the
sea's surface is decided in three places that do not agree.

1. **The tide overtops pockets the column holds dry.** The tide is drawn,
   not simulated: the sheet stands at `sea - 0.5 + tide`, up to 1.5 m over
   the mean, over every cell whose ground is under the high water
   (`planet_visibility.wgsl`, the sheet's listing), but not over a cell
   marked dry (`DRY_SHEET_BIT`, `planet_column.rs` `mark_sheet`): one whose
   ground is under sea level and whose column holds no water at the sea's
   top. The build's flood (`flood_from`) fills only what is joined to the
   sea UNDER sea level, so a pit or cavern behind a beach whose sand stands
   at sea level stays air, which is right at the mean. At any tide over the
   beach the sheet runs up to the pit and stops: a hole in the sea with the
   pit's walls and floor in it (`D`). Under a bank whose top stands over the
   high water (`C`, `h >= 1.5`) the sheet is not listed at all, so under the
   overhang the eye looks below the sea's surface into dry stone. The eye's
   own test (`eye_water_at`, the "open air over ground the high tide
   reaches" branch) meanwhile calls an open pit under the tide's sheet the
   sea's water, and the swimmer (`swim_surface`) reads the height field's
   ground: three answers for one place.
2. **The sea's water under a roof is drawn at the wrong height.** A cell of
   the sea's own water under a roof (`W`) has the roof's top as its record
   height. Over the sea level, the mesher draws it (`water_flow.rs`
   `cell_pieces`: the sea's top is the sheet's only where the record is
   under sea level) at its layer's top, 0 m, while the sheet beside it
   stands at the tide's height, anywhere from -1.5 to +1.5 m. Neither draws
   a face between them (the mesher skips a side toward water; the sheet has
   no sides), so at the cave's mouth the eye sees past a step of up to 1.5 m
   into the cave under its surface. Where the roof's top is under the high
   water the cell is drawn twice: the sheet at the tide's height and the
   mesher's top at 0 m.

**The fix: one answer, the sea's reach.** At the tier's build, and after
every edit, a walk from the open sea (cells whose column holds the sea's
water at its top layer with open air over it, under sea level) through every
cell that is not solid, at every layer up to the high water's, sideways and
up and down. A cell the walk reaches is one the sea's surface reaches at
some tide: its finest record is marked (`SEA_REACH_BIT`).
- **The sheet** is listed over a reached cell whatever its ground's or
  roof's height, and over an unreached cell under the high water only if
  its column holds water at the sea's top (a sealed pool someone poured).
  A dry pit or cavern behind a beach is under the sea's sheet; a hollow
  inland that no water reaches is no longer flooded at high tide.
- **The sea's own top under a roof is the sheet's**: the mesher leaves the
  sea's top layer to the sheet wherever the sheet is listed (one Rust
  function, `sheet_listed`, read by the mesher and the tests, beside the
  listing it mirrors). One surface, at one height, inside the cave and out.
- **Under a roof the sheet is lit by the cave**: the record carries the sky
  light over the water's top in its column (four bits), so the sea in a
  cave is as dark as the mesher's water in it was, not the open sky's.
- **The eye and the swimmer** read the same mark: on the tier, air under the
  high water in a reached cell is the sea's water at the tided sheet
  (`eye_water_at`, `swim_surface`); in an unreached cell it is dry.

Derived, like the flood: never saved, and the same tier marks the same way
however often it is built. The columns, the edits and the generator are
untouched, so saves are too. The tide is still drawn, not simulated: at low
tide a pit behind the beach holds the sea's surface at the tide's height,
like a tide pool.

**Not changed.** The flood, the water rule and the generator. Rivers: a cell
with inland water over the sea's level keeps its own surface (B18). The
sheet's look.

**Validation.** Before and after MP4s at 1080p60 at the owner's spot and two
other places, slowly across the cavern's mouth, under the overhang and
through the waterline, the eye's height burnt in; on Drive as
`sea-cavern-<place>-<before|after>-1080p60.mp4`, contact sheets and the
frame-by-frame list in `docs/screenshots/sea-cavern-holes/`.

### B23. Fish under the water at full brightness (owner, 2026-11-17)

The owner, under a shallow sea by day, looking level at a school of
yellow and orange striped fish 2 to 6 m off: "fish under water have full
light brightness, the water should lower light brightness". The seabed,
the far wall and the surface from below are dimmed and tinted teal by the
water; the fish stand out of it like stickers.

**Reproduced** (`docs/screenshots/underwater-fish-light/`, the "before"
frames): the reef at `13.096 94.896` with `--life reef:3` and its anthias,
a sea shore with `--life perch:3`, about 8 m down at `-4.125 -11.125`, and the
reef at 23:00 with a torch. The composite's murk does reach the fish (they
write depth like the reef, and the compose runs after the main pass); what
differs is how they are lit and tinted before it. Four causes:

1. **Lit by another light.** Every creature and every sea piece is drawn
   with the water life's material under `LitByField` but not
   `LitLikeTerrain` (`fish.rs` the school root, `fish/places.rs` the tile
   root), so `field_lit.wgsl` gives them Bevy's PBR picture
   (`apply_pbr_lighting`, `field_lit.wgsl:339`): Bevy's 15,000 lx sun,
   its own ambient and the mirrored sky, gated by the field. The seabed
   beside them is lit by the terrain's fill and its sun after the air
   (`sun_air`). Two lights in one view, by day and at night.
2. **Half the water's tint.** The terrain under water takes the daylight
   left at its depth AND a share of the water's deep colour
   (`planet_surface.wgsl:3846`: `mix(c*att, deep*light*att,
   (1 - mean(att))*0.5)`). Sea life takes only the first (`places::dimmed`
   in `fish.rs:1561`, `fish/places.rs:1259` and `:1277`), so it keeps its
   saturated colours where the bed beside it greys toward the water.
3. **Ambient fish are not dimmed at all.** The reef tile's anthias cloud,
   its clownfish and a seal swimming off its floe are emitted with a white
   tint (`fish/places.rs` `ambient`, `:1378`, `:1462`): the full colour of
   the atlas at any depth.
4. **Depth from the wrong surface.** A school's depth is measured from the
   mean sea sphere (`sea.radius - |p|`, `fish.rs:1561`). In a lake or a
   river over sea level that is negative, so fresh-water fish (perch,
   minnows) are never dimmed, and in the sea the tide is ignored.

**The fix: one rule for what water does to a colour, and the terrain's
light.**

- **`pbd_core::ocean::under_water_albedo(colour, downwell, deep, depth)`**:
  the terrain's own rule, `att * mix(colour, deep, (1 - mean(att)) * 0.5)`
  with `att = light_left(downwell, depth)`, floored at 0.04 as `dimmed`
  was so a shape still reads in the dark. A test holds its expression to
  `planet_surface.wgsl`'s. `places::dimmed` becomes a call to it, and every
  sea-life colour goes through it: the schools, the reef's pieces, its
  swaying pieces, the kelp, and the ambient anthias, clownfish and seals.
- **Depth from the water the creature is in**: a school reads its depth
  from `PlanetWater::surface` at its anchor, the same surface it swims
  under (a lake's or river's own top, else the sea's tided one). A tile's
  ambient life reads the sea's tided surface over the tile.
- **Lit as the terrain is**: the school root, the sea-places tile root and
  the `--life lineup` instrument carry `LitLikeTerrain`, so
  `field_lit.wgsl` lights them with the terrain's fill, the sun after the
  air, the sun's cascades and the lamps, the seabed's numbers. The look is
  a per-mesh row in the one shared material: no new pipeline.
- **Unchanged**: the murk by distance and Snell's window (the composite,
  already applied to them), the water's transmission seen from above (the
  sheet), the atlas, the palette and the bodies. No outlines. The vents'
  glow and smoke stay unlit (they are light). Saves are untouched: this is
  drawing only.

**Validation**: before and after stills of the same frames in
`docs/screenshots/underwater-fish-light/`: near the surface (the owner's
shot), about 9 m down, from above looking into the water, and at night
with a torch; looked at for the fish sitting in the water like the reef
and kelp. A test of the rule; READY with no new `LATE_PIPELINE`.

**What the frames showed (2026-11-17, built).** The largest part of the
"full brightness" was cause 1's mirrored sky: Bevy's picture gave every
creature and sea piece the open sky's light off its back (`sky_specular`),
and nothing in the water dims that, so a ray 8 m down was a glowing cyan
sliver over a dim bed and the reef fish's fins a bright blue rim. Lit as the
terrain, the ray is as dark as the bed beside it and the reef's own pieces
sit at the seabed's brightness. Fish 1 to 2 m under the surface stay about
as bright as before: most of the daylight is left there, so they are lit
more than a bed 3.4 m down, by the water's own rule. Not filmed: a school in
a lake or river over sea level (cause 4), and a lamp beside a school under
water (the instrument's torch landed 7 m off).

**The owner's view, measured (second pass).** The owner's fish are perch,
a school. At their spot (`24.000 122.66`, `docs/screenshots/underwater-
fish-light/owner-*`) the school is darker than the sand behind it (luma 34
against 45) and keeps its yellow and red. A white body lying on the sand
takes the sand's own transfer (linear 0.008, 0.13, 0.14 against 0.004,
0.15, 0.19), so the fish are treated as the seabed is: the same light, depth
tint and murk by distance. The difference is depth and distance: perch swim
0.3 to 1 m under the surface, 3 to 4 m off, and keep about three times the
red of a bed 2.4 m down and 5 m off. Tinting them further than the water
over them would be a look of its own, for the owner to decide with the
frames, and is not built here. The `--life lineup` instrument is tinted by
its depth too, so a body can be read on the bed beside the sand.
