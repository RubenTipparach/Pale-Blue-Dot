# Design: a playtest per mockup

## Context

- The mockups are single HTML pages under `docs/mockups/`. They use three.js
  r128, and most use the shared `building/scene-kit.js`.
- All but two run classic scripts, so their state is a global that
  Playwright can read. `world-map.html` and `held-tools.html` are IIFEs and
  need a hook.
- Hooks some pages already expose:

  | Hook | Page |
  | --- | --- |
  | `window.__build` | building |
  | `window.__town`, with `GOTO` and `teleport` | towns |
  | `window.__npcs` | npcs |
  | `window.__sail` | drydock |
  | `window.__hangar`, with `?test` and `run(sec, script)` | hangar |
  | `window.MG`, with `force(stops)` | minigames |
  | `window.__fishBake`, with `?bake` | fish |
  | `setLoad`, `PRINT_PASS`, `__frames` | construction |
  | `window.__pbd`, with `contexts` and `frames` | ocean |

- Scripts that exist: `tools/mockup_probe.js`, `tools/mockup_towns_checks.js`,
  `tools/mockup_sites_test.js` (world map), `tools/mockups/hangar_flight_test.js`
  and `tools/mockups/bake_fish_sheet.js`. Every other check made so far was a
  scratch script in a session's scratchpad.
- Pages with no 3D view: M3, M11, M12 and M16.
- Pages with more than one 3D view, in tabs: M5 (2), M9 (3) and M17 (3).
- Pages with no night: M18 and M19 (dated before the rule); M2 and M17 have
  no day and night control.

## Decisions

**1. A playtest is data.** `tools/mockups/playtests/<page>.json`:

```json
{ "page": "industry.html", "viewport": [1280, 800],
  "milestones": [
    { "id": "mine-tiers", "requirement": "industry-tiers 1: five tiers, one dig rule",
      "steps": [ {"hash": "mining-crew"}, {"wait": 3000},
                 {"click": "[data-mt=quarry]"}, {"run": true}, {"wait": 5000} ],
      "assert": [ "ST.mineTier === 'quarry'",
                  "markedYield().dig.every(k => hexDist(...k.split(',').map(Number)) <= QUARRY_R)" ],
      "shots": [ {"caption": "The quarry frame takes every cell inside it", "seconds": 6} ] } ] }
```

- The step kinds are `hash`, `click`, `drag` (from, to, button), `key`
  (down, up, hold for some milliseconds), `eval` (a page expression), `run`
  (press the page's Run or Play), `night` and `wait`.
- An assertion is a page expression that must be true. A pixel check (decision
  3) is written `{"pixels": ...}`.
- A step uses the page's own controls wherever one exists, as a player
  would. `eval` is for what a viewer could not do by hand quickly, such as
  flying a planned course to its end or ageing a dam by 40 days.

**2. One runner, two modes.** `tools/mockup_playtest.js <page> [--milestone id]
[--record out/] [--all]`.
- It serves `docs/mockups/` itself on a free port, so it needs no server
  started by hand.
- It runs every milestone in a fresh page, so one milestone's state never
  leaks into the next.
- **Check mode** prints one line per milestone and assertion, and exits
  non-zero on the first failure. It names the milestone, the assertion's text
  and the value it read.
- **Record mode** sets Playwright's `recordVideo` at the viewport size and
  plays the same steps. A shot's length is a wait while recording, so the
  video holds on what the caption names. It writes:
  - `<id>.webm` for each milestone;
  - `contact.png`, one frame per shot with its caption;
  - `playtest.json`, each assertion's result.
  It records even when an assertion fails, and the contact sheet marks that
  shot FAILED, so a failure can be seen rather than only read.
- `--all` runs every playtest file in turn, which is the batch the owner
  watches.

**3. The standing rules, checked on every page.** These run before any
milestone, from the page's first load, with nothing in the playtest file.

| Rule (CLAUDE.md) | How it is checked |
| --- | --- |
| No errors | Every `pageerror` and console error fails the run. |
| The tracker link | `a[href*="EwNcNzHQU3B7C3jyi629Wk"]` sits in the first screenful. |
| One 3D view at a time | Count WebGL contexts by wrapping `getContext` before load, and count drawing canvases by `requestAnimationFrame` calls that render. In each tab, exactly one canvas draws; hidden tabs have no running loop. A tab's renderer does not exist until its tab first opens. |
| Tabs are links | Each tab's `#anchor` opened cold lands on that tab. |
| Nothing spins on its own | With Run off, two screenshots of the 3D canvas 2 s apart differ by under 0.5% of pixels. A page whose world moves on its own (sea, wind turbines, a station ring) lists those regions as `"moving"` masks in its file, with the reason. |
| Lit at night | At night the canvas's mean luminance is below a set level, and the brightest 1% of pixels are near the page's lights. Where a page has a light list (`PBD.setLights`), every light within view lights its ground: a 9-pixel patch under each light, projected to the screen, is brighter than the mean. |
| Run is off by default | The Run or Play button starts `aria-pressed="false"`. |

Pages that predate a rule (M18 and M19 have no night) say so in their file,
`"exempt": {"night": "fixed daylight, dated 2026-09-24"}`, and the gate page
prints the exemption.

**4. Video that says what it is.** A cloud recording's first frame carries
the label "SwiftShader, no GPU: shows the look, not the smoothness". The
runner adds it as an overlay before recording. Frame rate is never asserted
in the cloud. On the owner's desktop, the runner uses
`--use-angle=d3d11 --enable-gpu --ignore-gpu-blocklist` (CLAUDE.md, "Render on
the GPU").

**5. One gate page format.** `step-videos`' `tools/gate_page.py` takes a
playtest's `playtest.json`, videos and contact sheet, and builds the page,
with the live mockup linked at the top (the owner's "post the prototype
too"). Each gate page is listed on the tracker under Misc in the step that
publishes it.

**6. A hook only where state is hidden.** A page whose script is an IIFE, or
whose state a test cannot reach, gets `window.__test = { state(), ... }`. It
gives read access, plus the few drives a check needs. It never changes what
the page does for a viewer.

**7. When playtests run.**
- In check mode, before every republish of a mockup. A mockup is not
  republished while its playtest fails.
- In record mode, when a mockup reaches a stage the owner reviews, and for
  the owner's batch.

## 4. The milestone plans

Each mockup's milestones, what is asserted, and what is recorded. Milestone
ids are the playtest file's. "Shots" are the recorded video, in order.

### M1 The Aubade System (`planets.html`)
- **bodies**: every body on the tracker is listed and selectable. Assert that
  each tile's measured cell is 2.833 m within the geodesic spread, and that
  each radius is on the ladder (`R = 300 * 2^(L-7)`) or names its level.
- **gas-giants**: selecting a gas giant shows no landing, and the airship over
  the hydrogen sea.
- **star-map**: Tallow and Gemel are on the star map.
- Shots: the system overview; Tenebris and Sequoia side by side; the gas giant
  airship at night; the star map.

### M2 Aubade Transit (`travel.html`)
- **liftoff-to-dock**: a scripted flight. Lift off, light the drive, coast,
  flip, brake, and dock. Assert each phase is entered in order, and that the
  docking ends with relative speed under the dock limit.
- **no-planning**: no maneuver-node UI exists (the invariant).
- **lanes**: the star lanes are locked until their tier.
- Shots: the whole flight at 1x for the first 10 s, then sped up, with the
  phase captions.

### M3 Forge and Fabricator (`crafting.html`, 2D)
- **dig-gives**: each block's drop table matches the recipe sheet.
- **chain**: crafting each station's recipe moves items in and out, and no
  count goes negative.
- **up-to-modules**: a ship module is craftable from dug items alone.
- Shots: a dig, a forge cycle, and the full chain to a module.

### M4 Shipyard and Ship Builder (`ship-crafting.html`)
- **place-modules**: placing modules updates mass, thrust and turn rate.
  Assert they follow from the parts: mass is the sum, and thrust over mass is
  the acceleration shown.
- **limits**: a test flight respects the configured linear and angular limits.
- Shots: building a small ship; its numbers; the test flight.

### M5 Townsfolk and Peoples (`npcs.html`, 2 views in tabs)
- **day-cycle**: aging world time moves each NPC through its schedule (`__npcs`).
- **talk-trade**: opening a dialogue and a trade changes stock and standing.
- **tabs**: the second view's renderer is made only when its tab opens.
- Shots: a morning to a night in a town; a trade; a falling-out.

### M6 Hex House Builder (`building.html`)
- **walls-floors**: walls snap to cell edges and floors to cells; a storey is
  three layers (`__build`).
- **ghost-why**: a piece that cannot fit shows its reason, and is not placed.
- **night**: the built house's lamps light its rooms.
- Shots: a one-room house going up, a refused piece and its reason, the house
  at night.

### M7 Herds, Hunters and Glowmoths (`animals.html`)
- **biomes**: each biome shows only its roster, with no fallback species.
- **tame-drop**: taming and drops match the sheet.
- **night**: the glowmoths light what is near them.
- Shots: a herd grazing, a hunter's chase (after Play), glowmoths at night.

### M8 Furrows and Greenhouses (`farming.html`)
- **till-plant**: tilling, planting for the climate, watering and harvest
  advance the plot's stage.
- **greenhouse**: a crop outside its climate grows only under glass.
- Shots: a season sped up; the greenhouse at night.

### M9 Fish Lab and Aquariums (`fish.html`, 3 views in tabs)
- **seeded**: the same species and seed draw the same fish (`__fishBake`),
  pixel for pixel.
- **tank**: a caught fish placed in a tank is that fish.
- **tabs**: three tabs, one renderer each, made on first open.
- Shots: a catch, the lab, and the same fish in a tank.

### M10 Stations and Outposts (`stations.html`)
- **dock**: docking at Meridian, Kessler and Gatehouse.
- **shadow**: in the planet's shadow the station is lit by its own lights.
- **ring**: the station ring spins (a world motion, masked for stillness).
- Shots: an approach and docking; sunlight to shadow.

### M11 The Long Fall: Games and Jobs (`minigames.html`, 2D)
- **odds**: each game's shown odds match a forced run of 10,000 plays within
  1% (`MG.force`).
- **shift**: an ore-sorter shift pays what the job board says.
- Shots: each of the five games played once, and a shift.

### M12 Exchange Terminal (`market.html`, 2D)
- **prices**: a station's price follows its stock.
- **events**: an event widens the gap between stations.
- **haulers**: haulers close the gap over time.
- Shots: the board across a market day, and an event.

### M13 Foliage Detail Playground (`foliage.html`)
- **biomes**: each biome's ground cover.
- **layers**: each layer's slider changes only its layer.
- **night**: scrubbing to night, the lamps light the cover.
- Shots: each biome; a scrub from noon to night.

### M14 Pale Blue Dot Lighting (`lighting.html`)
- **today-vs-new**: both models render the same frame side by side, by day and
  at night.
- **lamps**: at night every lamp lights its ground (the standing check, with
  this page's lamp list).
- Shots: the comparison by day, at dusk and at night.

### M15 Tenebris Towns (`towns.html`)
- **goto**: every `GOTO` place loads and stands the walker on a floor
  (`__town.teleport`). This replaces `mockup_towns_checks.js` without losing a
  check.
- **stairs**: the inn stair and the keep's newel climb storey to storey.
- **night**: towns section 7, the reference for "lit at night".
- Shots: `step-videos` 3.2's three walks (the inn stair, the harbour
  gangplank, a night overview).

### M16 Tenebris World Map (`world-map.html`, 2D, approved for build)
- **sites**: the 55 city sites are on the map; `mockup_sites_test.js`'s checks,
  moved into the playtest.
- **zoom**: zooming in shows finer detail without seams.
- **overlays**: each weather overlay draws.
- Shots: a zoom from the planet to a town; the overlays in turn.

### M17 Hex Tool Bench (`held-tools.html`, built in the game)
- **tools**: each of the pickaxe, axe, shovel and rod renders in its tab.
- **still**: the previews do not turn on a turntable.
- Shots: each tool, turned by a drag.

### M18 Rod, Shovel and School (`fishing.html`, fixed daylight)
- **cast-bite**: a cast, a bite and a catch run in order.
- **bait**: digging for bait gives bait.
- Exempt from the night check (it predates the rule).
- Shots: the whole loop from a dig for bait to a fish in hand.

### M19 Kestrel, Tern and Loon (`vehicles.html`, built in the game)
- **board-park**: boarding, flying and leaving a craft; it stays where it was
  parked.
- **gusts**: gusts move the craft and not the parked ones' rotors.
- Exempt from the night check.
- Shots: each craft flown, parked and re-boarded.

### M20 Pits, Plants and Power (`industry.html`)
- **mine-tiers**: each of the five tiers shows only its machine, tags and
  lamps. Each digs only marked cells, or for the quarry the cells inside its
  frame. A cell harder than the tier is left marked and counted.
- **chain**: ore to plate in the refining tab.
- **ether**: stage 1, then stage 2 at night, where the extractor is the
  brightest light.
- **ladder**: each generator on the ladder opens its scene. The platform opens
  the offshore scene. Switching grid nodes moves supply and demand, and a
  brownout slows every load by the same share.
- **oil**: the field, the well, the column, the drilling rig, the offshore
  platform. The derrick moves only while running, and the flare burns always.
- **carriers**: with Run on, marks move in the pipes, belt and tube; with Run
  off, nothing moves but the flare and the sea.
- **freighter**: hired shows the crew, automated shows the mast, and the hash
  keeps it.
- Shots: the five tiers in turn, each with Run on; the platform by day and at
  night; the transport yard with Run on, hired and then automated; the grid
  in a brownout.

### M21 Rockets and Landers (`ship-tiers.html`)
- **field**: the 2.4 km field and Pad B; the walk view at night.
- **planner**: planning to Taper puts nodes on the integrated line. Flying
  them ends in a bound orbit of Taper (eccentricity under 0.1), and returning
  plans home. This is the scratch `mission.js` from 2026-11-14, kept.
- **no-warp-drive**: no ship has a warp drive; the planner's time warp only speeds the clock, and it holds at 1x through a burn.
- Shots: a flight to orbit; the planned transfer to Taper flown to capture;
  the return.

### M22 Cranes and Printers (`construction.html`)
- **three-ways**: the treadwheel crane, the tower crane and the printer each
  raise their build (`setLoad`, `PRINT_PASS`).
- **still**: the cranes stand still until Run.
- Shots: each build from start to finish, sped up.

### M23 Drydock and Hex Ship Builder (`drydock.html`)
- **build**: building hulls from rowing to battery (`__sail`).
- **sail-gps**: sailing to a waypoint, the GPS coaching helm and sail; arriving
  within the waypoint's radius.
- **deck**: walking the deck.
- Shots: a build, a sail to two waypoints, and walking the deck at night.

### M24 Hangar and Chibi Aircraft (`hangar.html`)
- **build**: placing parts on the 1 m grid.
- **fly**: `hangar_flight_test.js`'s flight, kept as a milestone
  (`__hangar.run`).
- **crash**: flying into a solid hill ends in a crash, not a pass-through.
- Shots: each class flown once; a crash.

### M25 Seas, Rivers and Marshes (`ocean.html`)
- **environments**: reefs, kelp, polar ice, deep vents and wrecks, each in its
  tab or view, one context (`__pbd.contexts === 1`).
- **dam**: the beaver dam goes up over 40 days, scrubbed.
- **roster**: 61 species, each with a drawing.
- Shots: each environment; the dam over 40 days; the marsh at night.

## Risks / Trade-offs

- **SwiftShader is slow.** A 3D page records at a few frames a second at
  1280 × 800 and 2x. Record mode uses 1x and the page's own `?shot` low-cost
  path where it has one. Videos are of the look, as `step-videos` already
  says.
- **Pixel thresholds can be brittle.** Stillness and night checks compare a
  page with itself in one run, never with a stored image, so a change to the
  art does not break them. A page's own golden images are not part of this.
- **Assertions reach into page globals.** A rename in a page breaks its
  playtest. This is wanted: the playtest is updated in the same edit, which
  is when it is cheap to fix.

## Migration Plan

- The scratch scripts made so far are folded in:
  - `mockup_towns_checks.js` becomes M15's file;
  - `mockup_sites_test.js` becomes M16's;
  - `hangar_flight_test.js` becomes M24's;
  - M21's `mission.js` and M20's `ind.js` become their files.
- The old scripts stay until their playtest passes the same checks, and are
  then removed in the same commit.
- `step-videos` 3.2 points here: a walk file is a playtest with no
  assertions.
