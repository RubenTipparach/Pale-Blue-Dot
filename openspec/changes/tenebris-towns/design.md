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
windows 0.8 by 1.0 m on a 1 m sill.

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
| brushing a wall at 8.6° for a second | slides 7.09 m | 0.73 m, then stops |
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
light. At night the town has 8 moving point lights in the mockup, the nearest
to the camera; the engine's propagated block light has no such limit.

## 8. State and saves

- A door's open or shut state is a world mutation: it enters the durable
  transaction path when it changes (CLAUDE.md), keyed by the door's piece ID.
- A town's layout is data with a version, part of the saved world ID like the
  topology and generator versions, so a town is never regenerated under a
  save that has changed it.
- Townsfolk in the mockup walk fixed paths or work in place and are solid to
  the walker. What they do beyond that is a separate change.
