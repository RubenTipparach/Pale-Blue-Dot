# Pale Blue Dot — game design

Revision 0.1 · Design baseline, September 2026 · Working title

This document defines the intended game. The repository currently contains a tested engine foundation, shader ports, and generated biome art sources; the complete game and GPU terrain renderer are staged work. See [engine architecture](../openspec/changes/voxel-engine-foundation/design.md), [shader status](shader-port.md), and [migration record](source-migration.md) for the implementation boundary.

## 1. The experience

A first-person exploration, building, and survival sandbox across a small, persistent solar system. Walk through chunky hexagonal terrain, excavate a hillside workshop, build a spacecraft, and watch your home shrink into a pixel-painted planet. Fly to a moving station or another world, return with unfamiliar materials and biological discoveries, and build a network of outposts whose changes remain in the world.

The player is a surveyor restoring a scattered observatory network. Their long-term project is an atlas assembled from geology, ecosystems, ruins, and orbital observations. Each restored observatory reveals one destination and one practical technology. The story supplies reasons to travel without imposing a single building style or a linear ending.

The key promises are tangible terrain, distinct worlds, approachable spacecraft, and a continuous sense of place. Hexagonal prisms are the editable world itself. Planets span kilometers and have actual curvature, horizons, caves, and distant silhouettes. Pixel art supplies material character; atmosphere and water establish scale. A player can always identify their parked craft, find their last landing site, and understand why the ship is moving.

## 2. Scope and audience

Desktop first, keyboard/mouse and gamepad. Target a readable 60 FPS experience at 1080p on a representative discrete GPU; this is a validation target, not a measured result. WebGPU browser support is a later capability tier with lower residency and draw budgets. WebGL is outside scope. Single-player comes first; cooperative play uses the same authoritative simulation and world edits. Competitive PvP and cross-platform deterministic rigid-body lockstep are outside the initial release.

The first complete release targets four planets, a compact family of moons, an asteroid field, two station archetypes, 14 surface/environment biomes, modular ships, outpost production, and a discovery progression of approximately 15–25 hours before open-ended building. The vertical slice is deliberately smaller: one terrain region, one cave, an editable base, a flyable craft, one moon, and one station.

Do not commit to an entire seamless galaxy, planetary fluid simulation, realistic rocket staging, reentry heating, orbital transfer planning, or fully simulated distant wildlife. Those features do not serve the current flight and performance goals.

## 3. Player loops and pacing

| Scale | Player action | Reward / next decision |
|---|---|---|
| 10–30 seconds | Read terrain, mine a vein, place a beam, scan a lifeform | Useful matter, a safer route, a discovery entry |
| 5–15 minutes | Explore a landmark, finish a workshop, prepare a sortie | Tools, power, storage, a mapped landing site |
| 20–45 minutes | Fly an expedition, establish a remote outpost, return cargo | New fabrication capability and a new destination |
| Several sessions | Restore observatories and connect worlds | Full atlas, better logistics, ambitious construction |

The opening places the player in Tenebris fields near water, stone, and a sheltered ruin. Within ten minutes they can craft a tool and a powered beacon. The first ship is a damaged survey skiff repaired from accessible materials; it teaches translation, braking, landing, and boarding before asking for a long journey. A nearby moon mission teaches oxygen and power reserves. Sequoia introduces biological materials, Crag industrial minerals, and Frostbite cryogenic resources.

Every required progression material must have a discoverable, reachable source without already owning the technology it unlocks. World generation validates this dependency graph and repairs the initial region deterministically if necessary. Exploration rewards alternatives: a rare ruin can provide a component earlier, but never the only irreplaceable key.

## 4. Solar system and scale

All dimensions below are design values in meters, not inherited Tenebris dimensions. The system is deliberately compressed for readable travel. Planetary routes are roughly 80–300 km; a 600 m/s cruise crosses 100 km in about 167 seconds before acceleration and braking. A 4 km radius planet has about 25 km circumference, giving several minutes of low-altitude flight and many walking expeditions.

| Body | Radius | Surface gravity target | Atmosphere / water | Regions and role |
|---|---:|---:|---|---|
| Tenebris | 4,000 m | 9 m/s² | Breathable; blue oceans | Eight inherited climate biomes; starter engineering and agriculture |
| Sequoia | 6,000 m | 7 m/s² | Breathable; green-tinted basins | Redwood forest and sporewood; unique living materials |
| Crag | 3,000 m | 5 m/s² | Airless; no surface water | Basalt wastes; metals, impact structures, industrial outposts |
| Frostbite | 2,500 m | 3 m/s² | Airless; solid ice | Frozen wastes; volatiles and insulation materials |
| Moons | 500–1,500 m | 0.5–2 m/s² | Airless | Lunar regolith; navigation and low-gravity building |
| Asteroids | 40–250 m | 0.01–0.1 m/s² | Airless | Metallic asteroid material; salvage and compact mining |
| Stations | 60–250 m extent | Deck-local 6 m/s² | Interior life support | Docking, trade, research, navigation references |

Planets, moons, and stations use analytic on-rails ephemerides. A body's orbit and spin continue while its detailed terrain is unloaded. Orbits belong to a parent hierarchy and are evaluated at an explicit simulation epoch. Time acceleration, if later added, is disabled around undocked player physics until a safe policy exists. Asteroid field anchors may use the same background ephemeris; mined fragments use local physics.

Ships never enter a Keplerian rocket-flight mode. There are no required periapsis burns, patched-conic transitions, or orbital insertion maneuvers. A station moves because it is on rails; reaching it means flying to and matching a target reference frame. Navigation draws current destinations and predicted intercept markers, not a rocket orbital path.

## 5. Hexagonal voxel terrain

The surface is the dual of a subdivided icosahedral grid: almost all columns have six neighbors, with twelve unavoidable five-neighbor defects. These pentagons are normal world cells, support mining and building, and must appear in seam tests. A spherical world cannot be tiled exclusively by regular hexagons. Regular axial hex coordinates are a convenient local-patch representation, not a global planet address.

A voxel is a radial interval in one surface column. Near the surface, target approximately 1–2 m tangential spacing and 1 m layer height. The cell footprint changes slightly with radius and topology. Place blocks on their actual polygonal footprint; do not disguise square geometry with a hex texture. Underground rooms, ledges, overhangs, tunnels, and open shafts are supported by sparse occupied intervals and material overrides.

The initial planet build depth is up to 512 m beneath local terrain, with an unbreakable geological boundary below the editable shell. Moons and asteroids use shallower body-specific limits that keep every prism radius positive; they do not inherit a 512 m excavation depth. Fully destructible asteroid cores require a separate local-body representation in a later milestone. The first release does not promise excavation through a planet's center. Above-surface construction has a configurable radial limit and resource budget. Distant rendering simplifies the same body; it never replaces edited land with a different generated shape.

Players target a face and see its highlighted prism, placement orientation, material cost, and support state. A radial top face is a convex pentagon or hexagon; side faces follow neighbor edges. Building rotation advances in six local steps where appropriate, with explicit placement handling at pentagons. Blueprints use a local construction frame and can refuse a curved/seamed placement with a readable reason.

No automatic terrain-wide collapse. Natural terrain remains stable after mining. Player structures may have a bounded support graph with visible warnings; detached ship hulls become one aggregate body with simplified collision, never a rigid body per voxel.

## 6. Procedural generation

The world key is `(world_seed, generator_version)`. Body seeds derive from stable body IDs; independent streams cover geology, climate, hydrology, flora, fauna, and landmarks. Adding a cosmetic feature cannot shift ore or ruin placement. Persist the generator version alongside all edits, and refuse silent regeneration against an incompatible version.

The generation pipeline is: body parameters → spherical macro geology → elevation and drainage basins → latitude/altitude temperature and moisture → biome classification → strata and ore → caves → landmarks → ecological scatter. Sample macro fields in body-local directions, including across base-face edges, so there is no longitude seam. River basins use coarse global drainage with deterministic local refinement; independently noisy per-chunk rivers are not acceptable.

Use hierarchical cells rather than allocating a full finest-resolution planet. A coarse globe determines climate, ocean coverage, skyline, and landmarks. Only requested fine chunks materialize voxel buffers. Generate a collision neighborhood ahead of the player and flight corridor; visual detail may arrive later while an old valid representation remains visible.

Biome boundaries mix material patches over 20–80 m while dominant geography remains legible. Ore forms connected deposits governed by depth and host rock, not uniform speckle everywhere. Caves include chambers, fissures, and occasional aquifers, with entrance frequency controlled separately from total void volume. Starter-world repair preserves the world seed and writes its own versioned rule.

Landmarks are authored templates placed procedurally under constraints: observatories on visible ridgelines, wrecks near viable approaches, sealed ruins above flood level, and cave discoveries reachable with a reasonable tunnel budget. Landmark GUIDs are independent of chunk load order. Authored templates are data, not renderer-specific scene code.

## 7. Complete biome and art catalog

Every row has a generated 16-material source atlas under `assets/tilesets/`. [The interactive atlas](biome-atlas.html) displays the sheets and material slots; [biomes.json](../assets/tilesets/biomes.json) is the machine-readable catalog. The generated sheets establish palettes and material direction. They require pixel-grid, repetition, and in-engine UV review before being promoted to canonical 16×16 production textures; generation alone does not prove seamlessness.

| Biome ID | Geography and gameplay | Art and signature materials | Life |
|---|---|---|---|
| `fields` | Gentle ridges, accessible copper, farming and landing clearings | Olive pasture, warm dirt, limestone, oak | Tenebris oak, meadow flowers, grazer, field bird |
| `desert` | Dunes, sandstone terraces, exposed mineral seams | Ochre sand, rust sandstone, salt, cactus | Tenebris cactus and sand runner |
| `mountains` | Steep ascents, exposed iron/quartz, beacon sites | Blue slate, granite, scree, sparse snow | Tenebris alpine lichen and cliff bird |
| `tundra` | Frozen soil, low visibility storms, short growing season | Sage moss, thin snow, cold granite, dwarf willow | Tenebris willow and tundra grazer |
| `swamp` | Shallow channels, peat fuel, slow walking routes | Olive moss, wet peat, alder, reeds | Tenebris alder, frog, marsh insect |
| `jungle` | Dense canopy, vines, caves obscured by foliage | Emerald litter, red loam, kapok, basalt | Tenebris kapok, lizard, canopy bird |
| `ocean` | Shelves, submerged ruins, diving and fishing | Turquoise water palette, silt, coral rubble, kelp | Tenebris kelp, reef fish, shell grazer |
| `beach` | Safe early landing, tidal flats, coastal resources | Ivory sand, shell beds, driftwood, sea glass | Tenebris beach grass and shore bird |
| `redwood_forest` | Giant vertical forests, resin, canopy research | Teal needle litter, wine humus, copper-red bark | Sequoia redwood, silver fern, Shellkin, Crane |
| `sporewood` | Fungal basins, flooded channels, luminous caves | Mint mycelium, violet humus, cyan spore accents | Sequoia Sporewood, Sparkit, Raptor, basin ribbonfish |
| `basalt_wastes` | Impact trenches, exposed metals, no breathable shelter | Charcoal basalt, red scoria, sulfur, obsidian | None; Crag is lifeless |
| `frozen_wastes` | Ice escarpments, crevasses, buried volatile deposits | Blue-white snow, navy stone, layered ice, violet salt | None; Frostbite is lifeless |
| `lunar_regolith` | Crater rims, sheltered lava tubes, low-gravity travel | Warm-gray dust, breccia, anorthosite, impact glass | None; moon surfaces are lifeless |
| `asteroid` | Compact mineral bodies, exposed vacuum, anchoring puzzles | Carbon grit, nickel-iron rock, olivine, ice pockets | None; asteroids are lifeless |

Sequoia's climate and hydrology vary within its two art regions; a flooded sporewood basin does not import Tenebris's ocean species or its terrain tileset. Additional Sequoia climate biomes require their own explicit catalog entry and art. Moons can share nonliving geological material families while retaining their own seed, crater history, color configuration, and discoveries.

Wild flora and fauna species belong to exactly one planet. This includes fish and fishing loot, closing the aquatic roster gap documented upstream. A transplanted specimen, if introduced later, retains origin identity and can exist only in a player enclosure; it never joins another planet's wild spawn table. Lifeless body gates precede every biome spawn rule. No new species ships without a model/texture, icon, sound requirement, and roster entry.

## 8. Art direction

Materials use compact pixel clusters, restrained palettes, and visible texels. Terrain albedo targets 16×16 logical pixels per material; hero machinery may use 32×32. Geometry supplies the hex silhouette. Do not paint hex borders onto every texture, use photographic noise, or bake a second directional sun into the albedo. Distant surfaces preserve each world's palette and broad strata instead of dissolving into gray shimmer.

Use nearest magnification and controlled per-material mipmaps/minification. Store terrain in texture arrays to avoid atlas bleeding; material IDs select layers. Radial top UVs map the actual polygon into a square texture; side UVs follow edge length and radial height. Stable material-local UVs prevent swimming when chunks rebase. Choose integer texel-density rules before artists polish individual tiles.

The standard source sheet has four rows: surface/edge/soil/rock; resource and structural variants; ecological or geological details; path/building/subsurface variants. The exact 16 slot names for each biome are in the catalog. Source sheets remain separate from final texture-array layers, emission masks, and normal maps. The new generated art is not copied Minecraft content.

Water keeps Tenebris's depth absorption, Fresnel character, shore response, and underwater color cues. Atmosphere keeps readable sunset scattering and the planet rim. Voxel lighting keeps baked skylight, propagated block light, and local occlusion, and extends the upstream scalar block channel to colored emission. Pixelated materials should remain readable under these effects. Weather and clouds exist only on bodies that have atmosphere. Airless ice is illuminated by sun and local lights without atmospheric fog or snowfall.

## 9. Walking, tools, and survival

Walking follows local radial gravity; stations supply a deck-local frame. Target walking 4.5 m/s, sprint 7 m/s, and a 1 m step/jump obstacle budget, adjusted through playtesting. Avian handles local rigid-body collision; a capsule controller uses stable grounding and swept tests. Tree trunks and ground are evaluated in one authoritative collision neighborhood so stepping into a tree cannot remove the floor.

Tools: a mining cutter, builder, scanner, and interaction hand. Mining time reflects material hardness and tool tier; resource drops consolidate into stacks. A visible fracture pattern communicates progress. Unloaded chunks cannot be mined or walked into as if empty: retain safe collision and display a brief streaming state if the safety margin is exhausted.

Survival emphasizes preparation. Oxygen matters in vacuum and underwater, suit power runs lights/tools/dampener interfaces, temperature affects exposure on extreme worlds, and food provides useful endurance rather than a rapid starvation timer. Accessibility presets can soften needs without disabling exploration or construction. Storms alter visibility and power generation; they do not silently erase a base.

Failure drops recoverable cargo at a marked location and returns the player to a powered shelter. Unique progression records and scanned discoveries persist. The recovery beacon is visible from the map and ship HUD. A starter rescue option prevents permanent stranding after losing the first craft; its cost is time or common materials, never deletion of the save.

## 10. Inventory, construction, and production

The hotbar and backpack are one inventory with two views. Crafting, machines, trading, and storage show and consume from both. Stack transfers are atomic and validated by authoritative state. Item icons match the visible material or component; no invisible items or temporary text-only inventory entries in a playable milestone.

Construction starts with hand-placed prisms, then supports blueprints and multi-cell structural pieces. The builder previews costs and attachment points before confirming. Rotation, undo for recent unconsumed placements, and clear invalid-placement feedback matter more than a large initial block catalog. Persist every accepted edit through a journal rather than waiting for a periodic full-world save.

Production chain: raw stone/wood → basic tools and shelter; copper/iron → power, cutter upgrades, skiff repair; Sequoia resin/fiber → lightweight structure and life support; Crag alloys → cargo and industrial machines; Frostbite volatiles → efficient energy storage and long expeditions. Recipes have alternatives where a biome would otherwise become a hard gate. Simulation of distant machines uses elapsed-time batches capped by input inventory, output space, and supplied energy.

Power is a small explicit network of generators, batteries, consumers, and cables. UI shows demand and supply in understandable units. Solar generation responds to day/night and occlusion at a coarse validated cadence. Mining resources are finite by default; optional regeneration is a world setting and never rewrites player-edited cells.

## 11. Ship design and flight

Ships are assembled from structural cells plus functional modules: cockpit, power, thrusters, dampeners, cargo, landing gear, lights, and optional shielded life support. Aggregate hull mass, center of mass, inertia, and collision are recalculated only when construction changes. A thrust-direction indicator and center-of-mass overlay explain unstable builds. The first skiff has balanced predefined modules so learning flight does not depend on a perfect custom hull.

Flight is bounded Newtonian motion. Releasing thrust leaves momentum unless translational dampeners are active. Rotational dampeners bring angular velocity toward the commanded turn rate; they do not force the ship to face a privileged global axis. A normalized quaternion owns orientation and preserves roll through any pitch or pole crossing.

| Parameter | Initial design value | Meaning |
|---|---:|---|
| Surface target speed | 120 m/s | Near terrain; continuous acceleration-limited braking when entering this mode |
| Cruise target speed | 600 m/s | Open space; absolute normal-operation envelope |
| Docking target speed | 15 m/s | Relative to selected station/ship frame |
| Commanded acceleration budget | 20 m/s² | Combined gravity, thrust, and assistance under the accessibility flight envelope |
| Angular speed cap | 1.5 rad/s | Magnitude, not a separate cap on each axis |
| Angular acceleration budget | 3 rad/s² | Rotation input plus dampening |
| Physics rate | 60 Hz | Fixed tick independent of render FPS |

Gravity points toward the selected gravity body's center and falls off approximately as `g_surface * (R / max(r, R))²` outside its radius, with a bounded interior continuation. A smooth outer influence fade and hysteresis prevent abrupt owner changes. This is a gameplay field, not n-body ship orbit simulation. The selected navigation frame is shown in the cockpit; changing bodies or docking frames preserves composed position and velocity.

At each fixed tick, compute pilot acceleration, gravity, and dampening in one frame. Combine and limit the commanded acceleration vector once. Choose the reachable next velocity inside the applicable speed sphere; do not independently clamp axes. Avian integrates the body once. In vacuum, releasing input exponentially approaches rest when dampeners are on; under gravity, velocity damping alone permits descent and powered hover needs separate lift compensation; a precision mode adjusts response without creating instantaneous velocity changes.

Dampeners consume power and have a stated acceleration budget. Pilots may disable comfort dampening, but the absolute speed envelope remains enforced for streaming and collision safety. If gravity is stronger than available powered lift, the craft descends visibly; the accessibility setting that limits total acceleration must be explicit, since it is an assisted gameplay rule. Thruster damage reduces available commanded acceleration rather than secretly changing inertia.

A lower mode target is approached by bounded braking, not a snap. There is a mathematical exception: an arbitrary collision impulse or externally teleported invalid velocity cannot always satisfy both an instantaneous speed limit and a finite acceleration limit. The post-solver absolute-envelope guard may clamp that discontinuity; it logs a diagnostic and is not represented as normal thruster acceleration. Damage and impact effects use the pre-guard contact result. The foundation implements the controller and safety distinction; fuel, power allocation, and ship damage are future gameplay.

Stations are static in their own local walking frame. A player who jumps from a deck returns toward that deck rather than inheriting a leaked orbit velocity. Docking attaches the craft to a station-local anchor after an aligned low-speed capture. On release, compose station linear velocity and rotational point velocity with the local launch velocity. Reference-frame transfer uses a hysteresis region and explicit state transitions; a craft has only one active local physics owner.

Exiting never despawns the ship. The ship remains a persistent world entity with hull, cargo, power, motion, and boarding point. A landed craft sleeps when stable; an unattended flying craft either continues nearby physics or uses a documented coarse safety policy outside the active region. It is never silently converted into a refunded launch-pad copy. Teleport tools carry a boarded ship and its occupant together.

The HUD shows frame-relative speed, altitude, vertical speed, braking distance, dampener state, power reserve, and a clear destination marker. At 600 m/s and 20 m/s², stopping distance is 9 km before safety margin, so navigation begins braking well before docking or terrain and streams the route accordingly. Surface speed 120 m/s needs 360 m of braking distance under the same ideal budget. The UI uses actual available deceleration when gravity, damage, or power changes it.

## 12. Water, lighting, weather, and atmosphere

Water bodies are generated basins with a stable macro surface. Near edits use a budgeted local fluid state for buckets, small channels, and flooded caves. There is no promise of solving a planet-wide fluid grid each frame. Neighbor chunks exchange boundary states; unloaded regions use a coarse reservoir model. Saved fluid edits cannot reset when the visual mesh rebuilds.

Separate opaque terrain depth from water and atmosphere composition. Water samples its own planet-local camera and sun direction; an offset planet must look identical to the origin planet. Use depth reconstruction appropriate to Bevy's actual projection/reverse-Z convention, with sky/no-depth handling and a defined underwater transition. Screen-space refraction is optional by quality tier; absorption and Fresnel remain coherent without it.

Baked lighting stores separate sky openness and colored emissive channels. Multiply skylight by current celestial illumination rather than rebaking the entire world at sunset. Edits enqueue bounded local propagation and neighbor invalidation. Dynamic characters and ships sample a light field or probes consistent with nearby terrain. Do not apply baked darkness twice through both albedo and runtime AO.

Weather seeds derive from a body weather stream and simulation epoch. Rain, snow, fog, and clouds use atmosphere capabilities and local temperature; airless bodies bypass the entire system. Storm fronts can be perceived from orbit at a coarse scale and become local particles near the player. Particle counts and transparent overdraw are capped.

## 13. Discovery, encounters, and economy

The atlas records scanned species, geological samples, landmarks, landing sites, and a rotating planet preview. Discovery credit is unique per record and contributes research at observatories. A world map begins coarse and reveals surveyed terrain; it always preserves player beacons even if the surface representation is unloaded.

Early wildlife avoids the player or warns before attacking. Hostile encounters come from territorial animals, damaged security systems, or hazardous ruins. Combat supports the exploration loop: cover, retreat, clear telegraphs, and equipment choices. A full faction war or swarm RTS is not inherited from swarm-demo. Its GPU instancing and simulation patterns can inform bounded cosmetic flocks, debris, and distant traffic.

Stations buy survey data and common surplus, sell convenience components, and issue contracts with actual destinations. Prices are stable enough to plan a trip; distant simulation is coarse. There is no dependency on a live-service marketplace. Players can complete essential progression with exploration and fabrication even if they ignore trade.

## 14. Interface, audio, and accessibility

One interaction key boards, uses, or opens the highlighted target; conflicts are resolved by explicit focus priority. Builder mode and pilot mode have distinct readable input hints. Every remappable action appears in settings, including dampeners, precision flight, roll, and emergency brake. Gamepad radial tools avoid a pointer requirement for routine play.

Menus include new/continue world, seed and assistance options, settings, and a visible save status. Inventory operations work with keyboard, mouse, and gamepad. The pause menu explains whether a multiplayer world continues. Coordinate/technical profiling overlays are development tools and stay out of ordinary player decisions.

Provide independent UI scale, subtitle size, high-contrast targeting, color-independent resource indicators, reduced camera motion, optional horizon assistance, and configurable flight inversion. Pixel art does not justify illegible fonts. Motion sickness settings control camera sway, roll-follow strength, and field of view.

Audio communicates materials and environment: short distinct digging and footstep sounds, muffled suit conduction in vacuum, cabin machinery, wood/resin forest ambience, and filtered underwater movement. Directional warnings survive reduced visual effects. Music is sparse on the ground and broadens on ascent, with world-specific instruments and no mandatory copyrighted reference tracks.

## 15. Persistence and cooperative play

Persist seeds, generator version, simulation epoch, terrain/material edits, fluid deltas, machines, inventories, discoveries, and stable ship IDs. Procedural baseline is regenerated; player changes are journaled and periodically compacted. Include checksums, schema versions, and atomic replace/recovery behavior. Save failure remains visible and never falsely acknowledges durable progress.

Single-player commands take the same validation path as future multiplayer. A server owns edits, crafting, inventory transfers, and damage. Clients predict movement and show pending construction, then reconcile against revisioned authoritative results. Avian is not assumed bitwise deterministic across hardware. Networking uses snapshots and reconciliation; GPU-generated visuals are never the authoritative source of ore, collision, or inventory.

Co-op initially targets 2–8 players with interest management by body, region, and local physics island. Players on separate planets load separate neighborhoods. Shared stations, ships, and cross-island transfers require explicit ownership; a universal origin shift must not move another player's unrelated simulation. Disconnected pilots leave a persistent craft with a documented safe parking policy, not a deleted entity.

Offline catch-up applies only to bounded coarse systems such as machines and ecology statistics. It is limited by stored inputs/energy and a configured elapsed-time cap. It does not integrate every abandoned ship tick or apply years of starvation to an offline player.

## 16. Performance as a design constraint

Bevy ECS holds bodies, chunks, ships, machines, and gameplay agents. Voxel occupancy lives in compact chunk data; a voxel is not an entity. Use independent system access, change queues, bounded background work, and explicit fixed scheduling. A distant planet costs a coarse representation and orbit evaluation, not millions of ticking blocks.

Keep render geometry resident on the GPU. CPU simulation supplies canonical material/occupancy pages and small dirty-range updates. Compute shaders classify exposed prism faces, compact visible descriptors, build indirect arguments, and reconstruct vertices from descriptors and planet topology. Avoid round-tripping generated vertex buffers to the CPU. Avian collision proxies are built separately from authoritative CPU data near active actors; their geometry can be simpler and much smaller.

GPU-only visuals do not mean GPU-only world truth. CPU generation, edit validation, collision, saves, and networking must agree on occupancy without synchronous GPU readback. Use explicit buffer capacities, overflow flags, deferred reclamation, topology halos, and per-chunk revisions. A chunk's previous valid representation remains until replacement succeeds. Reject stale asynchronous jobs after edits or unload/reload.

Budgets for an initial 60 FPS desktop target: GPU terrain 4 ms, opaque nonterrain 2 ms, water/atmosphere 3 ms, lighting/culling compute 2 ms, effects/UI/post 2 ms, leaving approximately 3.7 ms headroom. CPU critical work aims below 8 ms with streaming jobs off-thread; GPU and CPU budgets overlap rather than being summed. Budget 2 GB terrain/render residency inside a provisional 4 GB VRAM tier, plus at most 1 GB CPU chunk/cache data. These are hypotheses to benchmark, not claims about this foundation.

Report actual p50/p95/p99 frame time, fixed-tick overrun, GPU pass times, upload bytes, resident pages, visible faces, draw count, collider rebuild cost, backlog, and overflow. Wall-clock timings use an actual monotonic clock. A software adapter can validate correctness but cannot establish hardware performance. Capture traversals, edits, waterline transitions, station approach, and worst-case checkerboard geometry.

## 17. Milestones and acceptance gates

| Milestone | Playable outcome | Required evidence |
|---|---|---|
| M0: foundation (this repository) | Headless bounded-flight/physics smoke; reviewable design/art/shader modules | Rust tests and smoke; WGSL validation; documented integration gaps |
| M1: terrain laboratory | Walk and edit one hex region including a pentagon and seam | Collision never opens during replacement; edit/save/reload equality; GPU overflow test; deterministic captures |
| M2: planetary surface | Circumnavigate Tenebris with all eight climates and caves | Kilometer streaming; near/far continuity; persistent remote edits; texture and light review |
| M3: water and sky | Surface-to-orbit ascent with water and atmosphere | Offset-body camera test; underwater transition; reverse-Z depth checks; day/night and airless captures |
| M4: skiff and station | Build, board, fly, exit, reboard, dock and walk on a moving station | Full-roll/pole flight; bounded braking; jump on moving deck; persistent dismounted craft; safe frame transfer |
| M5: solar expedition | Travel to all four worlds, moon and asteroid content | Unique rosters including fish; 14 reviewed material sets; no airless weather; no progression seed traps |
| M6: survival and logistics | Complete first observatory chain and recover from failure | Craft/inventory atomicity; failure recovery; discoverable recipes; machine catch-up limits |
| M7: cooperative alpha | Two players build together and visit separate bodies | Revision conflicts, reconnect, persistent ships, concurrent islands, network replay checks |
| M8: performance and polish | Complete guided expedition on target hardware | Recorded hardware/configuration; percentile budgets; accessibility and controller review |

## 18. Regression scenes and decisions still to validate

Keep fixed-seed captures for fields at noon/sunset, a jungle tree step collision, a mine with colored lamps, an edited cave crossing a chunk seam, shallow/deep water on an offset Sequoia, Frostbite without sky effects, a planet across LOD changes, and a station jump while the station moves. A fixed-tick script repeats flight through a pole, exits midair, reloads, and reboards the same ship ID. Headless tests verify invariants; visual captures and interactive play verify appearance and feel separately.

Before promoting the full renderer, measure GPU face-descriptor bandwidth against a conventional chunk mesh on the same hardware. Test pentagon and LOD stitching before expanding world count. Before balancing survival, play a complete 30-minute expedition with realistic inventory friction. Before production art import, review each atlas's tile order, pixel density, repetition, and planet identity. These are concrete gates, not reasons to leave unbounded implementation promises.

Open tuning decisions are the exact ship power cost, assistance behavior under heavy gravity, editable depth per body, orbit periods, starting equipment, and cooperative bandwidth targets. Defaults in this document are sufficient to build the slice and should change only with recorded playtest or benchmark evidence.
