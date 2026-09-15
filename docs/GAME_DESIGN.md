# Tenebris: Comprehensive Game Design Document

## Document control

| Field | Value |
|---|---|
| Genre | first/third-person exploration, survival, building, and vehicle sandbox |
| Setting | a procedurally generated star system of compact, kilometer-scale worlds |
| Core technology | hexagonal-prism voxel terrain, seamless streaming, assisted Newtonian flight |
| Modes | single-player first; cooperative and dedicated-server compatible by design |
| Design status | pre-production baseline; numeric balance is provisional |

## 1. Vision

Tenebris gives the player a continuous sense of place: a mineral vein can be discovered from orbit, approached through atmosphere, mined beneath a hex-voxel surface, refined into ship parts, and used to reach a visible moon. Worlds are large enough to sustain journeys and regional identity, but compact enough that players can understand and alter them.

The fantasy is **to read, inhabit, and reshape a living miniature star system**. Complexity belongs in systems and consequences, not in opaque controls.

### Experience pillars

1. **Every destination is a place.** Bodies have climate, geology, ecology, history, routes, and recognizable horizons.
2. **See it, reach it.** Visible planets, moons, stations, storms, and landmarks are traversable without a level-select fiction.
3. **Readable systemic depth.** Terrain, weather, logistics, power, and ecology interact through rules players can learn.
4. **Constructive permanence.** Mining, building, discoveries, and settlements persist and become the player's history.
5. **Approachable spaceflight.** Momentum and gravity matter, while flight assists make ordinary travel intentional rather than an orbital-mechanics exam.

### Anti-pillars

- Not a real-scale universe or astrophysics simulator.
- Not an undirected infinite content treadmill.
- Not per-voxel micromanagement for its own sake.
- Not survival through constant meter maintenance.
- Not a promise that every generated location is equally important.

## 2. Audience and accessibility

The primary audience enjoys exploration, base construction, resource logistics, and systemic sandboxes. The default experience supports short goal-oriented sessions and long expeditions.

Accessibility requirements include remappable input; hold/toggle alternatives; adjustable camera motion, FOV, shake, flashes, and UI scale; color-independent resource/alert language; subtitles and directional audio indicators; difficulty controls for hazards and resource pressure; and flight assist presets. Automation should reduce repetition without erasing planning.

## 3. Core loops

### Minute-to-minute

Observe terrain and instruments → choose a route/tool → move, fly, mine, scan, fight, or build → respond to a local complication → collect information/resources → update the plan.

### Expedition loop

1. Select a contract, mystery, resource need, or self-directed objective.
2. Inspect destination gravity, atmosphere, weather, daylight, and known hazards.
3. Configure ship modules, cargo, crew/drone roles, power reserves, and supplies.
4. Travel through surface, atmospheric, and space regimes.
5. Establish a safe operating point: landing zone, beacon, shelter, or orbital station.
6. Explore and extract while conditions and discoveries alter priorities.
7. Return, trade, research, construct, and publish/map findings.

### Long-term loop

Expand capability → connect distant regions → specialize settlements and vehicles → unlock harsher bodies/depths → reveal the system's history → make a system-scale choice that changes factions, ecology, or infrastructure.

## 4. World structure and scale

A campaign seed creates one curated-scale star system. Recommended initial scope is one primary planet, two smaller moons, several minor bodies, and two to four stations; production may expand only after traversal and content density are proven.

Bodies span several kilometers in playable circumference/diameter according to archetype. Scale is intentionally compressed. Each body has:

- an on-rails trajectory and spin/day cycle;
- gravity and optional atmosphere/magnetosphere;
- global geology, temperature, moisture, and hazard fields;
- named macro-regions and watersheds;
- a navigable surface plus selected subsurface depth;
- authored narrative anchors placed under procedural constraints;
- local ecology, resources, factions, ruins, and anomalies.

Planets, moons, and stations continue on rails when unloaded. Their position is a deterministic function of simulation time. Local populations, production, weather, and ecology use tiered simulation: detailed nearby, aggregate far away, and event-based while dormant.

## 5. Hex-voxel terrain design

### Why hexagonal prisms

Six horizontal neighbors produce natural radial growth, routes, influence, fluids, and area effects. Vertical layers retain digging and construction. The art direction should embrace the geometry in cliffs, foundations, fields, and tactical readability rather than disguising every cell.

### Player-facing spatial rules

- A cell has six lateral faces, top/bottom, material, integrity, and optional sparse state.
- Tools operate on a highlighted cell, column, ring, line, or brush preview with cost and stability feedback.
- Slopes, stairs, lifts, ladders, and vehicle ramps bridge discrete elevation.
- Structural rules are regional and legible; unsupported spans warn before collapse.
- Fluids and gases simulate in coarse volumes/columns, not expensive particle-perfect voxels.

### Planetary topology

Most terrain reads as regular hexes. A spherical world inevitably contains seams or exceptional cells; these are hidden in natural transition zones where possible, explicitly visualized in developer tools, and never allowed to break pathfinding, building adjacency, rivers, or ownership.

### Destruction and construction

Mining removes or converts cells and may expose gas, water, heat, artifacts, or unstable strata. Construction places terrain-like materials or modular structures snapped to cell faces/centers. Repeated edits create dust, sound, heat, and faction/ecological responses. Restoration tools let players refill, reinforce, reclaim, or rewild terrain.

## 6. Procedural generation

Procedural generation provides coherent context; authored rules provide meaning.

### Seed hierarchy

`campaign → system → body → region → feature → chunk → spawn`. Each branch has independent streams for terrain, climate, resources, structures, ecology, and encounters, so changing one content table does not reroll unrelated facts.

### Body generation

1. Select an archetype constrained by campaign pacing.
2. Establish radius/shape, gravity, atmosphere, orbit/rail, rotation, axial conditions, and palette.
3. Generate tectonic provinces, basins, elevation, geothermal fields, and crust layers.
4. Derive circulation, temperature, moisture, drainage, ice, and biome regions.
5. Route rivers, cave networks, migration corridors, and traversable passes globally.
6. Place resources from geological history, not uniform noise.
7. Place settlements, stations, ruins, and roads based on access, faction goals, and history.
8. Validate reachable critical paths, resource availability, safe starts, and unique landmarks.

### Region identity

Every macro-region needs a silhouette, traversal verb, resource proposition, hazard, ecological relationship, and at least one landmark or mystery. Examples include a storm-cut basalt crown navigated by sheltered lava tubes, or a frozen brine basin where heat extraction wakes dormant organisms.

### Content grammar

Points of interest combine a role (shelter, extraction, transit, ritual, research), origin culture, age/event, current occupant, environmental modifier, and reward/revelation. Constraints prevent nonsense combinations. Hand-authored set pieces can reserve terrain and supply bespoke interiors while adopting the generated context.

### Validation

Headless generation rejects or repairs seeds with unreachable starts, missing progression resources, disconnected mandatory routes, flooded critical structures, extreme spawn slopes, topology errors, or unacceptable streaming cost. A campaign records its generator version so updates do not silently rewrite it.

## 7. Traversal

### On foot

Walking, sprinting, crouching, mantling, climbing aids, short jet/boost tools, and environmental protection form the base kit. Surface angle, gravity, material, weather, carried mass, and suit modules affect movement. Falling remains predictable under variable gravity.

### Ground and utility vehicles

Rovers carry cargo and scanning/building modules; crawlers trade speed for steep terrain and drilling; hoppers handle broken low-gravity terrain. Vehicles interact with hex scale through readable clearance, suspension, ramps, and route tools, without being forced to snap to cells.

### Ships

Ships transition continuously among landed, surface-relative, atmospheric, local-space, and cruise presentation modes. These are control/reference-frame modes, not loading screens.

#### Flight model

- Thrusters apply force and torque; gravity falls off with distance.
- The pilot commands desired translation/rotation, interpreted by installed control systems.
- Inertial dampeners use available thrust to reduce unwanted relative velocity.
- Rotational dampeners arrest spin and hold attitude within available torque.
- A configurable flight envelope limits **commanded** acceleration and assisted velocity using smooth braking, not instant clamps.
- External forces can exceed the envelope; damage, low power, heat, cargo mass, or lost thrusters reduce control authority.
- Advanced players can reduce or disable assists for drifting, efficient coasting, towing, and emergency maneuvers.

HUD symbology displays reference frame, relative velocity vector, stopping distance/time, gravity direction, assist state, limit saturation, target motion, and collision terrain. Default controls must make hovering, landing, matching a station, and stopping reproducible.

#### Travel pacing

Surface flight rewards terrain reading. High altitude reduces obstacles but increases exposure and energy/heat considerations. Inter-body cruise may use a high-authority cruise assist or compressed presentation once safely clear, but it remains built on deterministic departure/arrival states and cannot bypass hazards, pursuit, or navigation decisions without a declared game rule.

## 8. Survival and hazards

Survival creates planning, not chores. Core pressures are suit/vehicle integrity, energy, thermal state, atmosphere/toxicity, and context-specific exposure. Food and rest may support expedition bonuses rather than relentless death clocks.

Hazards telegraph cause, trend, threshold, and mitigation. Examples:

- storms interfere with flight, power, visibility, and radio;
- geothermal zones provide energy but threaten heat and instability;
- vacuum demands pressure integrity and heat management;
- corrosive or biological regions require filters/material choices;
- radiation creates forecastable windows and sheltered-route decisions;
- deep strata introduce pressure, gas pockets, collapse, and ancient systems.

Failure prefers recoverable stories: damaged modules, forced landing, rescue beacon, lost cargo cache, injury/debuff, or changed local conditions. Difficulty modes can enable harsher loss.

## 9. Resources, crafting, and logistics

Resources fall into geological bulk, refined structural, conductive/electronic, chemical/volatile, biological, and anomalous categories. Distribution follows body history and biome, making trade and travel valuable.

Crafting layers:

1. field recipes for survival and repair;
2. workshop components for tools/modules;
3. industrial processes for bulk materials and propellants;
4. specialized facilities whose location matters (vacuum, geothermal, biological, orbital).

Recipes have material-function alternatives rather than endless one-off ingredients. Quality derives from feedstock, process, facility, and operator/automation. Logistics evolves from carried inventory to vehicle cargo, depots, scheduled routes, launch links, and station networks.

## 10. Building, power, and settlements

Construction occurs at three scales:

- **terrain works:** excavation, fill, roads, retaining walls, drainage;
- **modules:** rooms, machines, storage, defenses, farms, landing aids;
- **networks:** power, fluids, data, transport, and logistics routes.

Placement previews structural support, terrain conflicts, network reach, environmental sealing, required materials, and streaming/persistence ownership. Power sources have location-specific strengths; batteries and load priorities make outages manageable. Automation uses inspectable filters, routes, and production orders rather than mandatory programming.

Settlements track safety, shelter, power, supply, purpose, morale, and faction relationship. NPC growth is capped by infrastructure and simulation budget. Players may create outposts alone; larger communities require governance choices and supply resilience.

## 11. Tools, research, and progression

Progression expands verbs and reach rather than simply multiplying damage. Sources include scans/samples, recovered designs, faction teaching, experiments in relevant environments, and reverse engineering.

Capability bands:

1. survey and shelter;
2. reliable surface travel and extraction;
3. automation and harsh-region access;
4. atmospheric/space flight and orbital logistics;
5. remote-body settlement and system networks;
6. anomaly interaction and campaign resolution.

Research presents hypotheses and required evidence. The player knows why an expedition advances a technology. Essential progression has multiple sources so generation, faction hostility, or lost cargo cannot hard-lock a campaign.

## 12. Ecology

Biomes define producers, consumers, decomposers, resources, tolerances, and disturbances at an abstract level; nearby actors receive detailed behavior. Harvest, pollution, roads, introduced species, weather, and restoration alter regional state over time.

Ecology should produce visible, useful feedback: migration, regrowth, exhausted ground, contaminated water, predator displacement, or improved crop viability. It must not require simulating every organism off-screen. Rare species and anomalies create exploration goals, while common life supports readable material cycles.

## 13. Factions, NPCs, and economy

Factions have material needs, territorial preferences, beliefs about the system's past/future, and relationships that respond to observed acts. Reputation is contextual by faction and region rather than one morality number.

NPCs use schedules and needs while loaded; settlements aggregate labor, production, consumption, security, and travel when distant. Markets price local stock, demand, danger, access, and faction rules. Arbitrage exists but transport time, capacity, route hazards, and market response prevent trivial infinite loops.

Contracts arise from systemic needs—survey, delivery, rescue, repair, escort, containment, recovery, diplomacy—and can reference generated locations. Narrative arcs add authored stakes and consequences without disabling sandbox solutions.

## 14. Conflict

Conflict supports the exploration/logistics game rather than replacing it. Threats include fauna, automated defenses, environmental phenomena, raiders, and faction disputes. Players can often evade, negotiate, disable, distract, or change terrain.

Weapons and tools interact with cover, material, pressure, heat, power, and terrain integrity. Vehicle/ship damage targets modules and control authority, creating escapes and rescues. AI must understand local gravity, hex terrain affordances, atmosphere, shelter, and streaming boundaries.

## 15. Narrative

The system contains a layered history inferred through geology, ruins, archives, oral accounts, orbital infrastructure, and ecological discontinuities. The main story asks what should be preserved, restored, exploited, or transformed.

Narrative delivery follows three levels:

- macro: body histories and system-wide mystery;
- regional: factions, settlements, environmental crises;
- local: objects, spaces, logs, creatures, and player-created consequences.

Critical story sites are authored or authored assemblies with procedural placement constraints. They never depend on a single randomly generated clue. Major choices update faction goals, available technology, rail-station services, ecology, and ending state.

## 16. Multiplayer and social play

The design supports drop-in cooperative expeditions and persistent dedicated worlds, subject to production scope validation. Roles emerge from loadouts—pilot, navigator, engineer, surveyor, builder—not locked classes.

Required policies:

- server authority for world edits, inventory, damage, and rails;
- personal and shared permissions for ships, containers, terrain, and settlements;
- edit history and rollback tools for persistent servers;
- join compatibility checks for seed/generator/content versions;
- local interest replication rather than global object streaming;
- contracts and discoveries that credit meaningful group participation.

PvP is opt-in by world/region policy. Cooperative collision and construction griefing need permissions and rate limits from the first multiplayer prototype.

## 17. User interface and feedback

The UI has four layers:

1. diegetic/local: tool reticle, cell selection, suit/vehicle state;
2. navigation: compass, terrain map, horizon/space markers, route profile;
3. operations: inventories, crafting, networks, settlement and logistics;
4. system map: rail positions, destinations, forecasts, station services.

The map is knowledge, not omniscience. Scanning improves elevation, composition, hazards, routes, and confidence. Players can annotate and share discoveries. Any warning includes cause and suggested response. Flight mode always exposes current frame and assists; build mode always exposes validity, cost, ownership, and consequences.

## 18. Audio and art direction

The visual identity combines readable hex geology and human-scale machinery with vast skies and compact celestial vistas. Each body has a restrained material palette, atmospheric color logic, characteristic erosion, and landmark silhouettes. Generated repetition is broken at region scale, not through noisy prop scatter.

Audio communicates material, pressure medium, weather, machine load, structural stress, and unseen life. In vacuum, external events are conveyed through structure-borne sound, suit/ship systems, and instrumentation. Music layers respond to discovery, exposure, settlement safety, and transit rather than combat alone.

## 19. Difficulty and game modes

Presets adjust independently:

- survival consumption and recovery;
- environmental severity and forecast accuracy;
- combat damage and hostility;
- resource abundance and fabrication efficiency;
- flight assistance and collision forgiveness;
- construction integrity and maintenance;
- death/cargo recovery rules.

Creative mode offers generation inspection, free building, time/weather control, and flight debugging. Scenario mode uses curated seeds/objectives. Custom settings are shown before world creation and stored in the manifest.

## 20. Pacing and campaign shape

An illustrative campaign:

- **Opening (0–3 h):** survive landing, survey local region, establish power and shelter.
- **Surface mastery (3–12 h):** rover, deeper extraction, first settlement relationships and regional mystery.
- **Flight (10–25 h):** restore/build ship, atmospheric training, station contact, first moon expedition.
- **System network (20–60 h):** specialized outposts, logistics, faction commitments, harsh bodies.
- **Resolution (40+ h):** anomaly network and system-scale decision, with sandbox continuation.

Overlap is intentional. Skilled or returning players may sequence-break through knowledge and preparation, while tutorial contracts teach capabilities without mandatory hand-holding.

## 21. Metrics and playtest questions

Instrumentation should protect player privacy and be optional where required. Measure:

- time from intent to meaningful destination/discovery;
- travel time split by active decisions versus passive waiting;
- landing success, stopping overshoot, assist toggles, and collision causes;
- abandoned objectives and missing prerequisite resources;
- chunk stalls, visible LOD transitions, frame-time spikes, and memory high-water marks;
- frequency and geographic spread of terrain edits/builds;
- recovery from failure versus save abandonment;
- procedural repeats and regions crossed without interaction.

Every milestone asks: Can players form a plan from available information? Does travel contain decisions? Can they explain failure? Does a region remain recognizable? Did generation create a story rather than merely acreage?

## 22. Content scope and production rules

For the first complete release, prefer fewer bodies with strong identities over many shallow planets. Every added body archetype needs distinct generation rules, materials, hazards, landmarks, traversal implications, audio/visual language, and progression purpose.

Content data requires schema validation, stable IDs, localization keys, deterministic spawn constraints, and fallback assets. Designers need seed search, region previews, rail/time scrubbing, heat maps, spawn explanations, and one-click reproduction from a bug report.

## 23. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| spherical hex seams break systems | corrupted paths/builds | topology API, property tests, seam-first prototype |
| scale causes precision/jitter | poor flight and landing | typed `f64` frames, floating origin, local Avian bubbles |
| generation feels repetitive | exploration loses meaning | macro-region grammar, authored anchors, validation and rarity budgets |
| editable terrain overwhelms storage/meshing | stalls and huge saves | sparse edit overlays, debounced rebuilds, compaction, budgets |
| assisted flight feels artificial or unstable | core traversal fails | bounded controllers, clear HUD, tunable presets, fixed-step test harness |
| seamless travel creates dead time | player churn | route decisions, scanning, hazards, cruise rules, compact scale |
| scope exceeds team capacity | unfinished content | vertical slices, body content checklist, defer multiplayer breadth |
| simulation diverges in multiplayer | corrections/griefing | server authority, version handshake, interest bubbles, edit revisions |

## 24. Vertical-slice acceptance criteria

The first representative slice contains one visually distinct planetary region, one cave/resource chain, one settlement or ruin, editable hex terrain, shelter and power construction, a rover, and a ship capable of surface-to-local-space flight around an on-rails body.

It succeeds when:

- a fresh seed is playable without manual repair;
- the player can identify a distant goal, reach it, alter it, save, reload, and return;
- terrain streams without collision holes inside the safety envelope;
- ship landing and stopping are learnable with default assists and possible without them;
- rails seek deterministically and the body/frame transition does not pop or add velocity;
- generation/edit persistence reproduces exactly on another machine with the same supported version;
- profiling meets agreed frame, memory, and streaming budgets on the reference machine.

## 25. Open design questions

1. What is the smallest body radius that preserves convincing horizons without unacceptable topology distortion?
2. Is inter-body cruise continuous player control, a conditional time-compression assist, or both?
3. How much subsurface depth is systemic versus selected authored/deep regions?
4. Which survival pressures remain compelling after automation?
5. What target cooperative player count can share a physics bubble, and how are distant groups handled?
6. How visible should exceptional cells and hex geometry be in the final art style?
7. Which campaign consequences modify rails/stations, and how are those migrations saved?

These questions are prototype gates. They should not be answered solely through documentation.
