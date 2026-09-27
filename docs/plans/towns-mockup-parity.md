# Towns mockup parity

**The owner (2026-09-27): "can you also make sure all features in the mockup
are implemented too? make sure its in the roadmap and when you do the videos,
post the prototype too so I can compare/contrast stuff".**

Every feature of `docs/mockups/towns.html`
([published](https://claude.ai/artifact/59zS9ERBTkrVrzPedfoUL2)), with the
change and task that builds it in the game. A row is closed when its video
shot shows the game beside the prototype (`step-videos`) and the owner has
seen it.

Status:
- **planned**: a change's task builds it;
- **built**: in the game today;
- **ask**: waiting on a survey answer;
- **mockup only**: a tool for judging the mockup, kept out of the game with
  the owner's agreement.

`cities-in-the-world`'s gate is not passed while any row reads planned or
ask.

## Settlements

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| Walled town in the fields: inn, sixteen houses in eleven kits, smithy, moot hall, three-storey keep, wall towers, curtain wall with merlons, quay, market | `tenebris-towns` 3c, placed by `cities-in-the-world` | planned |
| A hamlet of straw and mud huts outside the walls | `tenebris-towns` 3b, 3c | planned |
| Village round a green: inn, smithy, hall, windmill, pond, fields and fences | `tenebris-towns` 3c | planned |
| Desert oasis town: sandstone houses with roof stairs, adobe domes, domed caravan hall, cactus | `tenebris-towns` 3c | planned |
| Tundra: igloos with tunnels, granite longhouse under turf, ice keep, curtain wall and towers, snow benches | `tenebris-towns` 3c | planned |
| Jungle: platforms in three kapok trunks at 8 m, a stair tower of poles, rope bridges, kapok huts, buttress roots | `tenebris-towns` 3c | planned |
| Swamp: alder stilt houses with decks and porch stairs, boardwalks on piles, a jetty, reeds | `tenebris-towns` 3c | planned |
| Fishing harbour: piers, finger piers, pier head, mole with a light, boathouses, shipyard slip, fish huts on stilts, fish market, terraces with street steps | `tenebris-towns` 3c, 3d | planned |
| Mound houses in the fields (the owner, T6): nine houses under turf domes dug into a hill, the great smial with a back room through a tunnel, gardens, a pond, the party oak | `tenebris-towns` 3c; placed as a fields settlement by `cities-in-the-world` | planned |
| Cliff village (T6): five terraces 3 m apart joined by a switchback of stairs, granite houses under slate, rooms cut into the rock, a rope bridge over a gorge, a beacon | `tenebris-towns` 3c; placed by `city-sites` (T12) | planned |
| Cave town (T6): a chamber inside a mountain entered by a tunnel, carved houses, a gallery, a chasm bridge, a pool under a daylight shaft | `tenebris-towns` 3c; placed by `city-sites` (T12); its chamber from its stored record, `cities-in-the-world` 3.1a (T13) | planned |
| Each settlement from its own biome's materials and plants | `tenebris-towns` 3c | planned |

## Buildings and pieces

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| Walls on cell edges, 0.25 to 0.6 m thick, one per shared edge; corner posts | `tenebris-towns` 3 | planned |
| Floors in cells; a storey of three layers; huts of one storey of two layers | `tenebris-towns` 3 | planned |
| Doors that open and close (E), saved as world edits | `tenebris-towns` 5 | planned |
| Windows with sills proud of the wall, and thresholds in every doorway | `tenebris-towns` 4 (trim) | planned |
| Straight flight, newel stair, street steps, porch stair, outdoor roof stair | `tenebris-towns` 3 | planned |
| Gable, pyramid, cone, flat with parapet, turf, dome roofs; chimneys; marble columns | `tenebris-towns` 3, 3b | planned |
| Turf dome cut by a limestone front, with a round door, round windows and a chimney through the turf; a vaulted tunnel between two domes | `tenebris-towns` 3b, built as the igloo and adobe domes are | planned |
| Rock overhead: a room cut under a terrace, a cave roof, a shaft to the sky | `cities-in-the-world` 3.1a: hollows in the settlement record (T13) | planned |
| 21 kits: straw, mud, timber, half-timber, three bricks, fieldstone, ashlar, clay, marble, whitewash, driftwood, sandstone, adobe dome, granite longhouse, ice, alder stilt house, kapok hut, mound house, mountain house (granite under slate) | `tenebris-towns` 3b | planned |
| Decks on piles, rope bridges with sag, boardwalks, jetties, piers, gangplanks, rails, fences | `tenebris-towns` 3c | planned |
| Furniture: tables, benches, beds, chests, shelves, barrels, crates, hearths, anvil; stood clear of walls | `tenebris-towns` 4 | planned |
| Market stalls, smithy shed, net and fish racks, lobster pots, crops, fire pits | `tenebris-towns` 3c | planned |
| Trees, cactus, reeds and shrubs placed by the layout | `tenebris-towns` 3c; `cities-in-the-world` 3 clears the generated ones | planned |
| No roof overlaps; headroom at every walkable point; no two faces in one plane | `tenebris-towns` 3, 4 | planned |
| Buildings cut from each cell's real corners, and none on a pentagon | `tenebris-towns` 3; `cities-in-the-world` 2 | planned |

## Harbour craft

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| 23 moored boats that bob on the swell, with a hull solid that does not move | `tenebris-towns` 3d, 4 (moving parts) | planned |
| Any boat can be used (the owner, T7) | `cities-in-the-world` 4.2b: the small boats are the game's sailing and paddle craft | planned |
| The cog: boarded over a gangplank, walked onto its deck, castles and stair | `tenebris-towns` 3d | planned |
| The cog sailing, with you walking on its deck (T9: "YES") | `sail-the-cog`, its own change after the roadmap | planned |

## Light

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| Block light per room: a hearth lights its room and not through its walls | `lamps-and-lanterns`; `tenebris-towns` 4 | planned |
| Lanterns by every outside door, facade lanterns spaced by settlement size, street lanterns | `lamps-and-lanterns` 5; `cities-in-the-world` 5.2 | planned |
| About half the windows lit, a candle in the room behind each | `cities-in-the-world` 5.1 | planned |
| Small lanterns on the jungle's rope bridges | `lamps-and-lanterns` 5; `cities-in-the-world` 5.2 | planned |
| Torches, braziers, forge, fire pits, per biome; a beacon on the mountain summit | `lamps-and-lanterns` 5 | planned |
| In a cave every light burns all day and reaches the chamber's full height | `lamps-and-lanterns` decision 4 already keeps torches, braziers and hanging lanterns always lit. Outdoors the mockup lights them at dusk instead; the game's rule wins | planned |
| A beam of daylight down a cave's shaft | the game's sky light through the open cell; the beam itself is `tenebris-towns` 4 (trim) | planned |
| Flames that flicker | `tenebris-towns` 4 (moving parts) | planned |
| A dim moon, so the lamps read | the game's ambient floor, kept dim by `lamps-and-lanterns` | planned |
| Day and night: a time-of-day slider, Day and Night buttons, L | `lamps-and-lanterns` 4.3: a time control on the pause panel | planned |

## People and moving parts

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| 16 townsfolk in the walled town, and the other settlements' people: walkers on paths, workers (hammering, wiping), idlers, a gate guard with a spear | `townsfolk` | planned |
| Townsfolk are solid to the walker | `townsfolk` 3.1 | planned |
| Windmill sails that turn | `tenebris-towns` 4 (moving parts) | planned |

## Walking

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| A stair walked on its pitch line (the owner, T3: "prefer glide") | `tenebris-towns` 2 | planned |
| A grounded walker held to a floor up to 0.35 m below | `tenebris-towns` 2 | planned |
| A refused move slides along the wall | `tenebris-towns` 2 | planned |
| Thin geometry blocks the body | `tenebris-towns` 1 | planned |
| Walking and running speeds (the owner, T1, T8, T8b, T8c): run 5 m/s; Caps Lock switches to a walk at 3; Shift sprints at 8 over either; Ctrl crouches at 1; walk under a roof | `tenebris-towns` 2 | planned |
| Wading at half speed in water over 0.3 m | `walking.rs` `water_movement_mult` | built |
| Walkable roofs (the owner, T4) | `tenebris-towns` 3 | planned |
| Jump | the game's jump | built |

## Views and tools

| Feature in the mockup | Built by | Status |
| --- | --- | --- |
| Walk view (V) | the game's first-person walking | built |
| Overview camera: orbit by drag, pan along its heading (right-drag, WASD, two fingers), zoom, double-click to walk there | `player-building` 4.5: the build camera | planned |
| Cutaway height in the overview; a cave's roof cut away in the overview | `player-building` 4.5 | planned |
| Places list: go to a named place | `world-map` 5.4: a site's named places on the map, and "go there" in debug | planned |
| Kits tab: every kit side by side | `player-building` 4.1: the palette's kit picker | planned |
| Collision view (C) | `tenebris-towns` 4 (debug keys) | planned |
| Fly through walls (N) | `tenebris-towns` 4 (debug keys) | planned |
| Readouts: feet, cell, headroom, speed, eye jumps | `tenebris-towns` 4 (debug keys) | planned |
| T: today's walker against the proposed one | dropped from the game (T11) | mockup only |
| Touch controls: move stick, Jump and Door buttons | none: "we're building a desktop game" (T10) | mockup only |
