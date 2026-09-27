# Design: Tenebris towns

A walled market town for a people with medieval tools: stone, timber, wattle and
daub, thatch, iron hinges, a forge. Every building can be entered, every floor
reached by a stair a person would recognise, and every wall, door and table
stops the walker. The JS prototype is `docs/mockups/towns.html`
([published](https://claude.ai/artifact/59zS9ERBTkrVrzPedfoUL2)). Nothing in the engine is
built yet.

## 1. What the engine has today

| need | what exists | where |
| --- | --- | --- |
| cell | 2.833 m flat to flat, 1.000 m a layer | CLAUDE.md, `planet::tile_widths` |
| walker | 0.3 m radius, 1.8 m tall, eye 1.6 m; 8 m/s, 14 sprinting; a 12 m/s jump under 25 m/s² | `walking.rs` |
| step | `step_height` 1.05 m (one terrace plus 5 cm), climbed in one tick | `walking.rs`, `planet_column::STEP_M` |
| floor and ceiling | `PlanetContact::stand(feet)`: the top of the solid run at or below the feet, the bottom of the one above; five footprint points, MAX floor and MIN ceiling | `planet_contact.rs`, `walking.rs::footprint` |
| going down | a floor within 3 cm below is kept; anything more and the walker falls | `walking.rs::resolve_ground` |
| a refused move | "Only block tangential motion": the walker stays where it was and loses all horizontal velocity | `walking.rs::resolve_ground` |
| headroom | a candidate under `floor + 1.8 m + skin` is a wall | `walking.rs::headroom` |

Nothing in this list is wrong for terraces and caves, which is what it was built
for. A town asks three new things of it, and section 5 measures each.

## 2. Scale: cut to the cell

**Walls go on edges.** A wall made of cells is 2.833 m thick. With walls on the
shared edge of two cells instead, a one-cell room is 2.5 m across inside
(timber, 0.3 m) and a house of two rows of four cells spans 12.7 by 5.7 m.
An edge has one owner, as the neighbour tables already require
(CLAUDE.md: "shared edge ownership"), so two buildings that touch share one
party wall. Where walls meet at a cell corner, a post covers the 120° joint;
in timber that post is the frame's own corner post.

**Floors go in cells**, as boards 0.2 m thick on joists. A floor that is a whole
cell is a column run, which `stand` already answers. A floor that is part of a
cell (beside a stairwell, the moot hall's dais) is a thin solid (section 4).

**A storey is three layers.** 3 m floor to floor, 2.8 m clear under the boards,
1 m more than the walker. Two layers (1.8 m clear) would not fit the walker;
four would make the stair run 2.7 cells. Door openings are 1.0 by 2.2 m;
windows 0.8 by 1.0 m on a 1 m sill. Huts are the one exception (section 9):
a single storey with no boards over it, so two layers leave 2.0 m.

**Masonry is still cells** where the real thing is that thick: the town wall
(one cell, 2.833 m, a real curtain wall's thickness), its gate passages
(open to 4 m, masonry above), and the wall walk on top.

**Roofs** are gables along the building's rows over the footprint's bounding
box. A footprint of hexes has a zigzag outline; a flat soffit at the wall top
closes the notches between the zigzag and the straight eave, and is the top
storey's ceiling from inside. Pitch is 45° for thatch and shingle, about 40°
for slate.

**Found in the mockup: an eave is a ceiling for whatever is under it.** A
house's overhang reached 0.8 m into the stair tower next to it, and the roof
underside read as a ceiling 1.2 m over the stair, which stopped the climb at
4.4 m. A town layout SHALL therefore keep every walkable point's headroom, not
only every cell's: a roof may not overhang a cell whose walkable surface is
within a body height of the eave.

**Found in the second mockup: two roofs can cut into each other.** On the hex
grid a building's odd rows stand half a cell east of its even rows, so two
houses in touching columns interlock by half a cell and, with 0.45 m eaves each
side, their roofs overlapped (the owner's "roof is clipping"). A layout SHALL
keep every roof's plan, eaves included, clear of every other roof and of the
keep and towers. One empty column between two-row houses is enough: it leaves
0.5 cell, 1.42 m, for two 0.45 m eaves. The mockup checks every pair when the
town is laid out; 34 roofs, no overlaps.

## 3. The three stairs

All three land on layer lines, so a stair never leaves a floor at a height the
grid cannot hold.

### Straight flight

Two cells in line, flat to flat: 5.667 m of run for 3 m of rise.

| | value |
| --- | ---: |
| risers | 16 of 0.1875 m |
| treads | 15 of 0.354 m, then the landing |
| pitch | 27.9° |
| 2R + T | 0.729 m (a comfortable stride is 0.60 to 0.70 m) |
| width | 1.636 m: the strip between the two cells' flats |

It needs a cell before its foot and a cell after its landing, in the same row,
so a house with a flight is four cells long. The flight is enclosed below by
walls either side, a boxed stair; the triangles of the two cells outside the
strip are floored upstairs and the well is railed except at the landing.

### Newel stair

One cell, one turn per storey.

| | value |
| --- | ---: |
| winders | 15 a turn, 24° each |
| riser | 0.2 m |
| tread at the walk line (0.72 m out) | 0.30 m |
| newel | 0.2 m radius |
| headroom under the next turn | 2.74 m |

**A metre of rise turns 120°, two edges.** So a doorway at any whole layer is
centred on an edge, and the exit's edge follows from the entry's:
`exit = (entry + 2 * rise_in_layers) mod 6`. The wall towers use this: from the
street at layer 2 to the wall walk at layer 9 is 7 layers, 14 edges, so the
street door is two edges round from the wall-walk door. From layer 3 it would
be 6 layers and the same edge, and the street door would open into the wall;
the layout checks it.

**The top landing is 30°, then a rail.** Without one, a walker who keeps turning
at the top walks off the last winder into the well. A landing over the turn
below costs that turn headroom, and the footprint makes it cost more than it
looks: the highest floor and the lowest ceiling can come from footprint points
about 72° apart at the newel's inner edge. The mockup's first try, 75°,
stopped the keep climb at 9.91 m on the headroom check. The limit works out
at 39°; 30° climbs clean.

### Street steps

One cell climbing one terrace, toward a neighbour in the row above: 6 risers of
0.167 m on 0.472 m treads, cut across the whole hexagon so no cheek walls are
needed. The side neighbours stand at most 0.5 m above the steps, inside the
step height.

## 4. Collision

### Two primitives

- **Column runs**, as today: terrain, masonry cells, whole-cell floors and
  ceilings. `stand` already answers them.
- **Thin solids**: a convex outline in plan, extruded over `[y0, y1]`. Walls,
  jambs, lintels, door leaves, rails, part-cell floors, furniture, posts and
  people.
- **Surfaces**: a region in plan with a top that is a function of position,
  answering one or more solid intervals at a point. The straight flight, the
  newel (one sheet per turn), street steps and roofs. The mockup's town has
  3,197 solids and surfaces.

`stand` becomes: every interval over each footprint point, from columns,
solids and surfaces alike. The floor is the highest top within the step
allowance above the feet, and the ceiling is the lowest bottom of anything
taller than that, exactly as now. Each piece keeps the cell it belongs to, so
the query stays a per-cell lookup; the mockup uses a 2 m hash in plan.

### Three rules

**A stair is walked on its pitch line.** A stair's top at a point is the line
from the foot of its first riser to the nosing of its landing, not the tread
under the point. The treads are drawn, and the eye climbs at the stair's slope.
Before: at 8 m/s the walker takes 22 risers a second, each a 0.19 m jump of the
eye. The engine's camera rule, "no interpolation, easing, head bob, follow
spring", stays as it is: the pitch line removes the jumps at the source rather
than smoothing them after. The feet sit up to one riser inside a tread; in
first person the feet are not drawn.

**A grounded walker is held to a floor up to 0.35 m below.** Going down a
flight at 8 m/s the floor drops 0.038 m a tick even on the pitch line, over the
3 cm band, and a whole riser at once on the treads, so today the walker leaves
the stair at every step. 0.35 m covers the steepest line in
the set (the newel's inner edge, 44°) at a sprint, with margin, and is well
under a terrace, so a walker still drops off a 1 m ledge. It does not apply in
the air or on the way up from a jump.

**A refused move slides.** A body that meets a thin solid or a column face too
tall to step is pushed out along the face's normal, and keeps the motion along
it. A hex town has almost no straight walls: every building's outline zigzags
at 60°, so a walker who stops dead on contact catches on every corner. The
headroom check stays a hard stop.

The mockup samples nine footprint points (the centre and eight at 0.28 m) to
the engine's five; the rules do not depend on the count, but the landing limit
in section 3 was measured with nine.

Unchanged: the 1.05 m step for terraces, which walls and furniture do not get
(a walker does not step onto a table), and the MAX floor / MIN ceiling
footprint.

### Speeds, sprint and crouch (the owner, T1 and T8)

The owner, first (T1): "8 is running, 2.5 is walking. slow automatically when
indoors or if I hold shift". Then, asked how that sits with today's sprint
(T8): **"caps to toggle sprint off, shift to sprint, run at 5 ms, sprint at
8ms, walk at 3 ms, ctrl is crouch 1 ms"**.

| gait | speed | key |
| --- | ---: | --- |
| run | 5 m/s | the default |
| sprint | 8 m/s | Shift, held |
| walk | 3 m/s | see below |
| crouch | 1 m/s | Ctrl, held |

- **Crouch** also lowers the body, so it fits where a standing walker does
  not. Proposed, to be tried in the mockup first:
  - the body 1.2 m tall and the eye at 1.0 m, from 1.8 m and 1.6 m;
  - letting go of Ctrl under a ceiling lower than 1.8 m keeps the walker
    crouched until there is room.
- **Two things are not settled, and are asked again:**
  - which gait Caps Lock toggles (survey T8b). The recommendation reads
    "toggle sprint off" as turning the run down to a walk, with Shift
    sprinting over either. The other readings are a sprint lock-out and a
    sprint lock-on;
  - whether the walker still slows to a walk under a roof (T1), now that
    Caps Lock chooses (survey T8c).
- The pitch-line and snap rules (above) are unchanged by speed. Sprint at
  8 m/s is today's running speed, the one section 5 measured.

## 5. Measured in the mockup

The same scripted walks, run in the page at the engine's 8 m/s on a 120 Hz tick,
with today's rules and with the proposed ones. An eye jump is a rise or drop of
over 0.1 m inside one 0.1 m piece of the sweep; the steepest pitch line in the
set climbs 0.097 m in one.

| walk | proposed | today's `walking.rs` |
| --- | ---: | ---: |
| inn flight, up: eye jumps | 0 | 16 |
| inn flight, down: ticks in the air | 0 | 84 (0.7 s) |
| street steps, up: eye jumps | 0 | 6 |
| keep newel, 3 turns up: eye jumps | 0 | 44 |
| keep newel, down: ticks in the air | 0 | 199 (1.7 s) |
| tower newel, 7 m up: eye jumps | 0 | 34 |
| out of the top door, not square to it: ticks stuck | 0 | 77 to 86 |
| brushing a wall at 8.6° for a second | slides 7.35 m | 0.73 m, then stops |
| a closed door | stops | stops |
| the same door, opened | walks in 2.67 m | walks in 2.67 m |
| a jump in the inn's hall | head stops at the boards: feet 1.985 m | same |

Both walkers reach the top of every stair. The difference is the eye and the
jamb.

## 6. On the sphere

The mockup's grid is flat and regular. The planet's is a Goldberg polyhedron
whose cells vary about ±9% and which has twelve pentagons. So:

- Pieces are built from each cell's **real corners**, not a template hexagon:
  a wall is as long as its edge, a flight's strip is the band between its two
  cells' actual flats, and its riser count stays 16 with the tread taking up
  the difference. At ±9% the tread runs 0.32 to 0.39 m.
- A newel's turn is by edges, not degrees, so it holds on a distorted cell.
- **No building on a pentagon**, or on a cell whose neighbours include one: a
  five-edge cell breaks the 120°-per-metre rule and the flight's row.
- Everything is in the body-local frame the terrain uses; heights are layers.

## 7. Light

Interiors are lit by their openings and their hearths. The mockup dims the sky
term on every face under a roof to 0.26; in the engine that is the baked
skylight of the voxel-light work, and the hearths, forge and lamps are block
light.

**Every light lights what is near it, wherever the camera is.** The owner, on
the harbour at dusk: "All your mockups should have lighting at night". The
first mockups lit a settlement with the 8 point lights nearest the camera, so
from the overview a town was dark but for one or two pools, and four of the
seven settlements had almost no lamps. The mockup now bakes block light into
the vertices when a settlement is built, the way the voxel-light change carries
it:

- Each hearth, forge, lamp, torch, brazier and the harbour's beacon adds its
  colour to the vertices within its reach: 7 m for a hearth, 9 for a forge or
  a lamp, 10 for a torch, 11 for a brazier, 18 for the beacon. It falls off as
  `(1 − (d/R)²)²`, weighted by how squarely the face turns to it, and softened
  within half a metre so a wall beside a lamp is not burnt white. A fire's
  strength follows its size, so an igloo's lamp does not light it like a
  hall's hearth.
- A light reaches only faces in its own room (indoor cells joined to their
  neighbours of the same name), or only outdoor faces if it is outdoors, and
  indoors only within its own storey. A hearth does not shine through its wall
  or up through the floor.
- Fires burn all day. Lamps, torches, braziers and the beacon are lit after
  dusk.
- The 8 point lights nearest the camera stay, dimmer, for the flicker and to
  light the people, boats and doors, which the bake does not reach. The moon
  is dimmed from 0.28 to 0.16, and the night sky light from 0.2 to 0.13, so
  the lamps read.

**Every settlement carries its own lights, of its own kind.** A lantern on a
bracket beside every house door that opens outdoors (huts have none);
lanterns on posts along the streets, placed once a settlement is built where no
door, step, solid or named place is in the way; and per settlement, braziers
round the desert's plaza, torches at the ice castle's gate and on the tundra
path, torches at the jungle platforms' rails and the stilt huts' stairs, a
hearth in every swamp stilt house and a lantern on its jetty, lanterns at the
harbour's pier ends and on the cog's stern, and the light on the mole.

**The bigger the town, the better lit its streets.** The owner, on the lit
settlements: "I expect bigger cities to have well lit streets and partially lit
windows. The forest bridges should have lighting too. Maybe smaller lights."

- **Street lanterns on the house fronts.** Every house front that faces a
  street (cobble, flagstone, a lane, the desert's packed sand) at its own level
  may carry a lantern on a bracket at 2.6 m, and does unless another night
  light is nearer than the settlement's spacing: 3.4 m in the walled town,
  4.2 m in the harbour, 5.5 m in the desert town, 7.5 m in the village, none in
  the camps and stilt villages, which have their torches and boardwalk
  lanterns. The door lanterns count, so the fronts fill in between them. A
  bracket lamp stands on nothing, so no route is blocked.
- **Some windows are lit.** Each window is lit or dark by a hash of where it is,
  so a settlement is lit the same way every time: 55% in the walled town, 50%
  in the harbour, 45% elsewhere. A lit window is a warm pane set in the middle
  of the wall's thickness, facing out, of a brightness between 55% and 100%,
  shown only after dusk; from inside you look through it. Behind it a candle
  lights the room (5 m reach), and a little of its light falls on the street
  outside (4.5 m). These two are baked only, never one of the point lights.
- **Small lanterns on the jungle's rope bridges,** hung outside the rope rails
  on alternate sides every 5.5 m or so, 5.5 m of reach at 40% of a lamp. The
  planks between them are lit, and the bridge reads as a line of lights from
  the platforms.

| settlement | lights before | lights now | windows lit |
| --- | --- | --- | ---: |
| walled town | 31 fires, 7 lamps | 31 fires, 61 lamps | 276 of 491 |
| village | 13 fires, 2 lamps | 13 fires, 19 lamps | 110 of 251 |
| desert town | 2 lamps | 24 lamps, 5 braziers | 31 of 58 |
| tundra camp | 7 fires | 7 fires, 2 lamps, 4 torches | 9 of 11 |
| jungle village | 2 fires | 2 fires, 9 bridge lanterns, 7 torches | 7 of 15 |
| swamp village | 10 lamps | 5 fires, 17 lamps | 21 of 44 |
| fishing harbour | 15 fires, 6 lamps | 15 fires, 48 lamps, the beacon | 174 of 356 |

The bake takes a settlement from about 0.2-0.3 s to build to about 0.3-0.6 s
in headless Chromium; each frame costs the same bar two more vertex attributes.

The engine's propagated block light has neither the 8-light limit nor the room
rule: its occlusion is the voxels themselves. None of the scripted walks
changed: no lantern, torch or brazier stands on a route.

## 8. State and saves

- A door's open or shut state is a world mutation: it enters the durable
  transaction path when it changes (CLAUDE.md), keyed by the door's piece ID.
- A town's layout is data with a version, part of the saved world ID like the
  topology and generator versions, so a town is never regenerated under a
  save that has changed it.
- Townsfolk in the mockup walk fixed paths or work in place and are solid to
  the walker. What they do beyond that is a separate change.

## 9. Kits

A kit is what a building is made of. It sets the wall faces per storey (the
last repeats), the corner posts, the roof and its gable end, and the ground
floor. It does not change how a building is cut: walls on edges, floors in
cells, the storey and the stairs are the same for every kit, so a stair or a
door is placed the same way in a brick house as in a timber one.

| kit | walls, thickness | corners | roof | floor |
| --- | --- | --- | --- | --- |
| straw hut | bundled reed, 0.3 m | timber | six-sided thatch cone | earth |
| mud hut | cob, 0.5 m | cob | low turf gable (24°) | earth |
| timber | lapped boards, 0.25 m | timber | shingle | planks |
| half-timber | stone below, framed daub above | stone, timber | thatch or shingle | flagstones |
| red brick | running bond, 0.4 m | stone quoins | clay tile | planks |
| buff brick | running bond, 0.4 m | brick | slate | planks |
| clinker brick | Flemish bond, 0.4 m | stone quoins | clay tile | clay tile |
| fieldstone | rubble, 0.5 m | fieldstone | thatch | flagstones |
| ashlar | cut stone, 0.5 m | stone | slate | flagstones |
| clay | clay render, 0.5 m | clay | flat, parapet, beam ends | clay tile |
| marble | marble blocks, 0.5 m | a column at every corner | clay tile (29°) | chequered marble |
| whitewash (coast) | lime-washed rubble, 0.5 m | whitewash | slate (42°) | flagstones |
| driftwood (coast) | silver-grey lap boards, 0.25 m | timber | thatch | planks |

Bricks are drawn 0.5 by 0.25 m, eight by four pixels, twice a real brick: at
16 pixels a metre a true-size brick is three pixels long and reads as noise.

**Huts** are one storey of two layers under an open roof. Their doors are 0.9
by 1.9 m, 8.5 cm over the walker. A cone's collision is its own underside,
2.0 m at the wall line rising to the apex, so the inside of the roof is what
you see and what your head meets. A hut's eaves hang to between 1.45 and 1.75 m outside
the door, below the walker's head, so they are drawn but do not collide past the
walls; the roof-overlap check keeps anything tall away from them. The mockup
walks into all ten huts and every kit's house through its door.

## 10. A settlement per biome

Each settlement takes its materials and plants from the biome catalogue
(`docs/game-design.md` section 7), so none borrows another biome's, which is
the same rule CLAUDE.md sets for life and textures across planets.

| settlement | biome's materials | buildings | new pieces |
| --- | --- | --- | --- |
| walled town, village | olive pasture, limestone, oak | the eleven kits; a windmill in the village | none |
| desert town | ochre sand, rust sandstone, salt, cactus | sandstone houses with flat roofs, mud-brick domes, a domed caravan hall on seven cells | dome, outdoor stair |
| tundra camp | thin snow, cold granite, dwarf willow | igloos, a granite longhouse under turf and snow, a keep, curtain wall and towers of ice | dome (the igloo) |
| jungle village | red loam, kapok, basalt | platforms on seven cells round three kapok trunks at 8 m, huts on them, a stair tower of poles | deck, rope bridge |
| swamp village | wet peat, olive moss, alder, reeds | alder stilt houses 2 m over the water with decks and porch stairs | deck, boardwalk, jetty, outdoor stair |
| fishing harbour | beach: ivory sand, shell beds, driftwood, beach grass; the fields behind it | driftwood and whitewash houses (lime burnt from the shell beds) beside the fields' fieldstone and timber, boathouses, fish huts on stilts, a light on a mole | pier, gangplank, boats, a ship |

**The new pieces are floors, ceilings and rails like the rest.** A deck is
whole-cell slabs on piles. A walkway between two points is a surface whose top
follows the planks: a straight line for a boardwalk or jetty, a parabola for a
rope bridge (0.8 m of sag over 22 m is at most 8° at the ends), with rails as
thin solids along both sides. An outdoor stair is the straight flight's rule
between two points: risers near 0.19 m, walked on the pitch line. A dome's
underside is its ceiling, with the doorway cut out.

**An igloo is not cut to the cell.** It is a dome 4.6 m across and 2.5 m high
centred on one cell and spilling over its ring, because a dome that fits in one
cell is 1.4 m high inside. You can stand wherever the dome is over 1.83 m,
which is 1.57 m from the middle; a ring of wall at that radius makes the walker
slide round the inside instead of stopping on the headroom check. Its tunnel is
2.0 m high at the crown.

**Water is waded.** The mockup slows a walker whose feet are 0.3 m under the
water to half speed, `walking.rs`'s `water_movement_mult`; it does not model
swimming.

Walked in the mockup, with no airborne tick and no eye jump over 0.1 m: up the
jungle's pole tower to 9 m, onto the platform and across a rope bridge (the
feet dip to 8.2 m); up a swamp porch stair to a 2.3 m deck and into the house;
in through an igloo's tunnel; up an ice tower to the 6 m wall walk and through
the gate into the keep; up a desert roof stair from 2 m to the 5.3 m roof; into
a domed house and the village inn. The seven settlements have no roof
overlaps.

### The harbour

The owner asked for "a coastal town/fishing hamlet where there's lots of
[docks] and boats". It stands where the beach meets the fields, so it takes
from those two catalogue entries and no third: driftwood and shell-lime
whitewash from the beach, fieldstone and timber from the fields.

**Laid out from the sea up.** Twelve rows of water over a sand bed that shelves
from 4 m deep to 0.35 m, three rows of beach at 0.25, 0.5 and 0.75 m, a stone
quay at 1 m, then three terraces a layer apart with a lane along each and an
up street climbing both terraces by street steps. The sand shows through the
turquoise water, as the ocean biome's palette asks.

**Piers are walkways at the quay's height.** Planks on piles 1 m over the water,
so you walk off the quay onto them without a step: a main pier of 33 m with
two finger piers and a pier head, a west pier with two fingers, an east pier
with two, and a hut pier out to the fish huts. They are the jungle's walkway
piece with no sag and no rails; bollards stand along the main pier's edges.

**A boat is a lofted hull and a solid.** Its hull is lofted from U-shaped
sections, half-beam `B/2 · f^0.55` and depth `D (0.35 + 0.65 f^0.4)` with
`f = 1 − |2t − 1|^2.4` along the length, and a sheer that rises to the ends.
Twenty-three are moored: rowboats (4.2 m), sailing boats with a lateen sail
(6.6 m) and canoes (5 m) by the huts. A moored boat bobs a few centimetres on
the swell, drawn only: its collision is a fixed six-sided hull up to the
gunwale, so a walker wading at one stops at its side (0.99 m from its centre
line, half its beam plus the body's radius), and one stepping off a pier can
drop into it, as you would. One more sits in each boathouse, two lie keel-up
on trestles on the beach, and
the shipyard has a hull in frame on keel blocks, half planked, the ribs of its
upper half bare, over a slip into the water.

**The cog is boarded like a building.** 15 m by 5 m, moored along the main pier.
Its deck is a floor at 1.9 m that follows the hull's plan, 0.1 m over the
gunwale, with a rail round it and a 1.4 m gangway in the rail on the pier side.
The gangplank from the pier (1.0 m) lands on the deck's edge (1.9 m) over
2.3 m, 21°, a walkway surface like a pier's. An aftcastle and a forecastle
stand 1.6 m over the deck, the aftcastle up an open stair of eight 0.2 m
risers; one mast carries a yard with its sail furled.

The mockup found one thing to fix: the deck as a wall solid refused the
gangplank. The plank met the deck 0.07 m under its top, and a wall blocks
unless the feet are within 3 cm of its top, so the walker stopped at the
gangway. The deck is a step solid instead, which the walker steps onto as it
does a floor; nothing else reaches it, because the rail stands everywhere but
the gangway and the hull's side is 2 m from the pier.

Walked in the mockup (people moved out of the way, as in section 5):

| route | feet | airborne ticks | eye jumps over 0.1 m |
| --- | --- | ---: | ---: |
| quay down the main pier to its head | 1.0 throughout | 0 | 0 |
| up the gangplank, through the gangway, onto the cog's deck | 1.0 to 1.9 | 0 | 0 |
| across the deck and up the stair to the aftcastle | 1.9 to 3.5 | 0 | 0 |
| down again and back over the gangplank to the pier | 3.5 to 1.0 | 0 | 0 |
| hut pier, porch stair, deck, into a fish hut | 1.0 to 2.2 | 0 | 0 |
| beach, quay, both street steps, the top lane | 0.5 to 3.0 | 0 | 2 |
| out of a boathouse's open side into the sea | 0.25 to −0.7 | 26 | 1 |

The two jumps up to the top lane are the beach's own 0.25 m rises from sand to
quay, terrain steps rather than stairs (on the engine's 1 m layers the beach
would be one layer). Walking into the sea drops off the shelf, 0.6 m and then
0.35 m, and the walker wades at half speed.

### Mountain, cave and mound dwellings (ninth round)

The owner, in the survey (T6): "ooh mountain or cave settlements need to be
there too, no other planets for now, revise mockups with mountain dwellings
please, and cave dwellers too! hobbit ground based houses or mounds would be
cool too!" Three more settlements, each from its own biome's catalogue
entry. None uses Sequoia's luminous sporewood caves, which belong to another
planet.

| settlement | biome's materials | buildings | new pieces |
| --- | --- | --- | --- |
| mountain village | mountains: blue slate, granite, scree, sparse snow, alpine lichen | granite houses under slate on terraces three layers apart, some with a back room cut into the rock behind; a switchback of outdoor stairs; a rope bridge over a gorge; a beacon brazier on the top terrace | a rock-cut room: cells under the upper terrace, with a rock ceiling one storey up |
| cave dwellers | under the mountains: granite, basalt, slate | a chamber inside the mountain, entered by a tunnel from a cliff. Stone house fronts along its walls, rooms carved into the rock behind them, a gallery one storey up reached by an outdoor stair, a rope bridge over a chasm, an underground pool, and a shaft of daylight through one hole in the roof | a cave roof: a rock slab over the chamber at 12 m, with the mountain above it; it is the chamber's ceiling and dims its sky light |
| mound houses | fields: olive pasture, warm dirt, limestone, oak, meadow flowers | houses dug into a green hillside: a turf dome 6.4 m across and 3 m high behind a limestone front with a round oak door and round windows, and a chimney through the turf. The largest has a second room through a short tunnel. Gardens and fences, a pond, and an oak with lanterns in it | a turf dome: the igloo's dome with its underside as the ceiling. You can stand wherever it is over 1.9 m, which is within 2.45 m of its middle. A ring of wall there makes the walker slide round the inside |

**Lit like the rest.** The cave is lit only by its own fires: braziers along
its paths, torches at doors, hearths in rooms and a fire pit in the chamber.
The daylight shaft is its one opening. The mountain village has street
lanterns on its terraces and the beacon. The mound houses have a lantern at
every round door and lanterns hung in the oak.

**Built (2026-09-27).** Three settlements join the menu: *Fields, mound
houses*, *Mountains, cliff village* and *Mountains, cave dwellers*.

- **Mound houses.** Nine houses under ten turf domes, each 3.2 m in radius and 3 m high. Each has a
  limestone front 1.7 m from its middle, which is cut round a round door
  2.2 m across and two round windows 0.66 m across with glazing bars.
  - Inside is a board-lined vault on a plank floor. There is a limestone
    hearth at the back, and its chimney comes up through the turf.
  - The walker stands anywhere within 2.45 m of the middle, where the vault is
    over 1.93 m. A ring of wall at 2.55 m makes it slide round the inside.
  - The great smial has a back room 8.5 m east, through a vaulted tunnel 5.5 m
    long, with walls to 1.3 m and the vault to 2.0 m.
  - The houses sit in two rows. The second row stands 3 m up the hill, reached
    by two flights of 16 risers cut into it. Each dome is dug in: the ground
    behind it is raised one and two layers.
  - Round the houses: gardens with fences, a pond with reeds, and the party oak
    with five lanterns hung in it.
- **Cliff village.** Five terraces 3 m apart, from the valley to a summit at
  15 m, joined by five flights of 16 risers that alternate ends: the
  switchback.
  - Thirteen two-storey granite houses under slate stand against each
    terrace's back.
  - Two houses have a room cut into the rock behind them. It is two cells
    under the terrace above, 2.6 m high under 0.4 m of rock, and the lane
    above runs over it. An open door leads in from the house.
  - A rope bridge 8.5 m long with 0.9 m of sag, lanterns on its rails, crosses
    the gorge from rim to rim to the hermit's hut.
  - The beacon stands on the summit.
- **Cave town.** The chamber, up to 33 cells across and 15 rows deep (about 93 m by 37 m), is
  under rock from 12 m up to the mountain's top at 16 to 23 m.
  - The way in is a tunnel two cells wide and 3 m high, through a dressed
    stone portal in a cliff 14 m high.
  - Eight rooms are carved into the walls behind stone fronts, each with a
    door, shuttered windows and a hearth. Two of them open off a gallery one
    storey up, which is railed and reached by a flight of 16 risers.
  - A rope bridge 5.7 m long crosses a chasm 16 m deep.
  - A pool lies under a shaft that lets in a beam of daylight. Around it are a
    plaza with a great fire, stalls and stalagmites.

**What the mockup gained to do it:**
- **Rock overhead.** A cell can carry rock from one height to another, such as
  a cave roof or the rock over a rock-cut room. It collides as a column run and
  is drawn wherever no neighbour covers it: its underside, the rock face over a
  house front, round the shaft, above the tunnel's mouth.
- **Rooms of any shape.** A room may be a dome cut by its front, or a tunnel,
  and still dim its sky light and keep its block light in. Rooms joined by a
  tunnel share one light.
- **Lights that burn all day in a cave.** The game already has this rule:
  `lamps-and-lanterns` decision 4 keeps torches, braziers, hanging lanterns and
  candles always lit, and only post and wall lanterns dusk-lit. The mockup
  follows it inside the cave. The cave's lights also reach the chamber's whole
  height, not one storey.
- **A cut-away overview.** The cave's roof is hidden in the overview, so the
  town shows from above. The air inside is dark, not the sky's colour. The sun's
  shadow covers the whole chamber, so no daylight reaches its far end through
  the rock.

**Walked the same way.** Every new route was scripted and measured with
the proposed walker. Each has no airborne tick and no eye jump over 0.1 m,
and ends within 0.3 m of its goal:

| route | height climbed | airborne ticks | eye jumps |
| --- | ---: | ---: | ---: |
| The lane, through the smial's round door, round the room, through the tunnel into the back room | 0 | 0 | 0 |
| The lane, up the steps cut into the hill, through a hill house's round door | 3 m | 0 | 0 |
| The valley, up all five switchbacks, to the beacon | 15 m | 0 | 0 |
| A house's front door, through its open back door, into the rock-cut room | 0 | 0 | 0 |
| Over the gorge on the rope bridge | 0.9 m down and up | 0 | 0 |
| Outside, through the portal and the tunnel, up the gallery stair, into a gallery house | 3 m | 0 | 0 |
| The gallery, down the stair, over the chasm on the rope bridge, into a carved house | 3 m down | 0 | 0 |

Two layout bugs were found by these walks and fixed:
- **The bridges sagged into steps.** Both rope bridges first ran from cell
  middle to cell middle, so the sag left each far end 0.39 m below the ground.
  They now run rim to rim.
- **The door was out of reach.** A mound house's "go to" spot was 2.4 m from
  its door, beyond the 2.6 m the door can be reached from once the walker
  stops. It is now 2.0 m.

The seven earlier settlements' scripted walks (the walled town, the village
and the harbour) return identical results on the old and new pages in the same
sitting. Their screenshots differ from the old page by less than the old page
differs from itself.

## 11. No two faces in one plane

The owner found a window sill flickering against the wall under it. A sill ran
from 6 cm below the window's bottom edge up to that edge, which is also the top
of the wall below, so the two top faces shared a plane and the depth test chose
between them pixel by pixel. The mockup scanned every settlement for faces of
different pieces that face the same way, lie in the same plane and overlap by
more than 30 cm². Before the fix there were 2,232 upward ones:

- **Sills** against the wall below every window, in every kit. A sill now
  stands 3 cm proud of the wall top.
- **Floors against wall tops** where the wall above is thinner (half-timber
  over stone), and **roofs against wall tops** between merlons (the keep, the
  ice keep). A floor or flat roof is now drawn 1 cm above its layer line.
  Collision stays on the line, so no stair or walk changes.
- **A doorway above a lower storey's wall,** such as a tower's door onto the
  wall walk. Every doorway now has a 2.5 cm threshold board.
- **The harbour piers' first planks** over the quay's flagstones, lifted 1 cm
  like the floors. **The town landing's post tops**, raised 3 cm.

536 overlaps remain, all hidden:
- the tops of beams and hearths under the floor above;
- wall tops under the storey above;
- faces under a sill or behind a doorway's jambs.

The overview camera's near plane now grows with its distance (2% of it,
0.5 to 4 m), so depth precision holds where the camera is far off.

For the engine: the piece mesher builds trim (sills, thresholds, floors over
wall tops) proud of the faces it meets, never in their plane, and a test runs
the same scan over a built settlement's meshes.

**Furniture stands clear of the walls.** The owner found a chest sunk into a
wall. Furniture is placed from a cell's centre plus an offset, and a hex room
narrows toward its corners, so a chest or a bed near a corner reached through
the angled wall. Each piece is now pushed clear of the walls, doors, corner
posts and hearths already built, by the least move that frees it. The move
comes from a separating-axis test for boxes, or the nearest-edge push for
round pieces, repeated until nothing overlaps.

This covers tables, benches, beds, chests, shelves, barrels, crates, hearths,
the loom, the oven, pots, rugs, bedrolls, the jungle huts' leaf beds, and the
boathouse pots and oars. With the push off, 55 of 342 pieces reached into a
wall, up to 0.35 m deep; with it on, none do. No walk changed. For the engine:
a settlement template places furniture with the same push, and a test checks
that no piece overlaps a wall, post or door.

