# Proposal: seas, rivers and marshes (ocean-environments)

## Why

The owner, 2026-11-14, in four messages on the same task:
- "Draft up mockups for advanced ocean environments".
- "Coral reefs, clams, lobsters, crabs, bottom feeders. More marine life.
  Octopus, squid, tube worm thingies on vents".
- "Oh yea, rivers are important too, like beavers and dams, kelp forests,
  marshes".
- "And ducks".

Today the game's water is one sheet at sea level over a bare seabed of hex
columns:
- **The sheet is good.** `pbd_core::sea` and `water.wgsl` draw one table of
  wind-driven waves that hulls float on, with shoaling, crest and slope foam,
  Snell's window, rain ripples, the rain volume and lens drops
  (`openspec/specs/planet/water`).
- **Under it is plain.** The murk is `deep_color` lit by the open water's
  light and faded by one absorption (`absorption_per_m` 0.32 / 0.035 / 0.012)
  to the eye's depth, so at the column's floor 19% of the blue light is still
  there and the deep never goes dark. There are no caustics, shafts, snow or
  turbidity.
- **Eight fish swim in schools**, spawned by four water classes (river,
  shallows to 6 m, shelf to 40 m, deep) and a temperature window
  (`assets/config/fauna.ron`, `openspec/specs/world/fauna`). Nothing else
  lives in the water.
- **Rivers are channels at sea level.** The generator carves them only below
  40 m altitude and down to the sea's height; their water is the sea's, flat
  and still. The flow hook is ported and zero (`water-flow`). Water above sea
  level cannot exist until volumetric water's phases 2 and 3
  (`volumetric-water`).
- **The column ends 145 m below the datum** (`column::BASE_M`, which never
  moves, so every saved edit stays indexed). The ocean floor is 60 to 145 m
  down.

## What Changes

One mockup, `docs/mockups/ocean.html`, "Seas, Rivers and Marshes", and this
write-up. No game code: that waits for the owner's approval and a request
(CLAUDE.md, "Write it up before touching code").

The page has fourteen tabs in three rows, one 3D view live at a time:
- **Sea**: open sea by sea state (calm to storm), coast and tide, warm coral
  reef, kelp forest, polar sea, the deep and its vents, trench, caves and
  wrecks.
- **Fresh**: a river from source to mouth, beavers and dams, marsh and estuary.
- **Guide**: how underwater looks (light by depth, today against proposed),
  water life (a bestiary of 61 entries with a marine, brackish and fresh
  filter), where each place occurs (a chart of a sample planet), and in the
  engine (today against new, saves, decisions).

Each place is a hex-cell scene by day and at night, lit at night only by its
own lights, with its life moving. What each scene proposes:
- **Light that fades with depth**, by a downwelling coefficient of its own,
  so the deep is dark; turbidity per place; caustics, god rays and marine snow.
- **Depth zones fitted to the column**: sunlit to 40 m, twilight to 90 m,
  midnight to the floor. Nothing about the column changes.
- **Tides**: an offset of the one sea function, 3 m on Tenebris (the home planet, in the Aubade system),
  derived from the clock.
- **Places from the planet's data**: reefs in warm shallows, kelp on cool
  rocky shelf where the current upwells, sea ice from the climate normals,
  vents on the ridges, trenches where plates meet, rivers from drainage, marshes
  in flat wet lowland, beaver streams in temperate forest.
- **Water life**: 41 new entries beside the eight species in the game and the twelve
  from M7 and M9, appended to the roster, with environment tags beside the
  water classes; ducks with families, tipping up, flocks and sleep.
- **Beaver dams as authored world changes**: a colony is a record and a world
  process whose dams and lodges are journal entries, refused on any cell the
  player touched.

Everything that changes what the ground is made of is a **new generator
version, for new worlds**; every old world keeps its own version cell for cell
(CLAUDE.md, "Saved games survive every change").

## Capabilities

### New Capabilities

- `world/water-environments`: where each water place occurs and what it is
  made of, the light under water, tides and depth zones, and beaver dams.

### Modified Capabilities

- `world/fauna`: species gain environment tags, bed crawlers, surface
  floaters and water birds; protected species cannot be caught.

## Impact

- **Data:** a new `assets/config/ocean.ron` (environment rules, turbidity,
  tide range, zones); `water.ron` gains `downwell_per_m`, `caustics`,
  `shafts`, `snow`, `spray_from_mps`; `fauna.ron` gains species (appended) and
  `env:` tags.
- **Shaders:** `water.wgsl` (downwelling, turbidity, shafts, snow) and
  `planet_surface.wgsl` (caustics on wet ground).
- **Generator:** a new terrain version for reefs, kelp beds, vents, trenches,
  sea ice, stepped rivers and marsh pools. The last two wait for volumetric
  water.
- **Persistence:** the beaver colony and its edits through the
  `world-persistence` journal; nothing else here is saved.
- **Performance:** not measured. This is a mockup in a cloud session with no
  GPU; the screenshots ran on SwiftShader and say nothing about frame time.
