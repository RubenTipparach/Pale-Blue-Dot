# Design: land-transport

## Context

Phase 5 is mass transportation. Its land half is carts and horses, cars and
trains on their tracks (the owner, 2026-11-21: "add cars and cars and carts,
and horses and trains (and tracks) mockups to phase 5"; 2026-11-22: "Roads
and cars mockup?"). It runs on the roads phase 3 builds (`townsfolk` design
10, 10a, 11, 31, 34):

- a road cell is a hex cell 2.833 m flat to flat with a dirt or gravel top;
- where a road climbs, the cell's top is one plane rising one layer (1 m)
  from the middle of the edge it is entered by to the middle of the edge it
  leaves by: 19° straight through (a 35% grade), 22° round a 60° bend;
- a ford dips the road a layer into shallow water on a gravel bed;
- a timber bridge is a plank deck on posts whose top is the road's height;
- lamps stand by the roads near towns.

**Status: nothing is built.** Phase 5 starts when the owner asks. The mockup
is M38, `docs/mockups/land-transport.html`. Every choice below that is not
the owner's own word is marked **recommendation taken (ask only with
screenshots)**: CLAUDE.md's rule is that a question goes to the owner only
with something to look at, and the mockup's screenshots are that something
when the owner reviews it.

## Goals / Non-Goals

Goals:
- show every land vehicle the owner named at the game's scale, on the game's
  road cells, beside a 1.55 m townsperson;
- settle the geometry questions a hex world raises for wheels and rails;
- say what is stored, so a cart left on a road is still there next session.

Non-Goals:
- driving controls, gears, fuel, or a vehicle's menu (a later change, after
  the owner picks from the mockup);
- road building by the player (that is `townsfolk` design 31);
- airships, sea ships and sailing (M23, M24, `steer-and-sail`);
- performance: nothing here is measured.

## Decisions

### 1. Scale and look

| Thing | Size |
| --- | --- |
| Townsperson (M5's block head) | 1.55 m |
| Horse | 2.4 m nose to tail, 1.5 m at the withers |
| Two-wheeled cart and horse | 5.5 m; wheels 1.2 m; track 1.56 m |
| Covered wagon and horse | 7.0 m; wheels 1.1 and 0.84 m; 2.5 m to the cover |
| Early car | 3.6 m, wheelbase 2.4 m, 1.6 m wide, 1.9 m to the roof |
| Small truck | 4.9 m, wheelbase 3.0 m, 1.9 m wide, 2.3 m to the crates |
| Tank engine, coach, open wagon | 5.1, 4.6 and 4.0 m; 1.8 m wide; 2.95 m above the rail |
| Track | metre gauge, sleepers 1.8 m, ballast bed 2.2 m wide |

Low-poly, flat-shaded, pixel-textured at 16 px a metre with nearest
sampling, in the ground's palette; no outlines. Every vehicle fits inside
one road cell's width (the widest, the truck, 1.9 m in 2.833 m).

### 2. What a road vehicle needs from a road cell

A road vehicle reads only the road: per cell, its top (flat, or the slope's
plane), whether it is a bridge deck (whose top is then the road height), and
its surface (dirt, gravel, ford) for speed and sound. All of it is CPU
terrain data the road already stores; nothing needs a GPU readback.

- **Slope.** A layer a cell is the steepest road. A horse walks it at about
  70% of its pace; an early car climbs it in low gear at about 60%. The
  mockup slows both on the grade. **Recommendation taken (ask only with
  screenshots):** no road is steeper than a layer a cell, as phase 3 already
  rules; vehicles do not stall on it.
- **Bends.** A road bends 60° in one cell, never more. A vehicle drives the
  bend as an arc from the middle of the cell before it to the middle of the
  cell after it, tangent to both straights: a radius of `W / tan 30° = 4.9 m`
  (twice the 2.45 m arc that fits inside the one cell). The arc passes 0.76 m
  inside the bend cell's centre, so it never leaves the road. When the next
  cell bends too, the arcs halve (2.45 m), which carts and cars take at
  walking pace. A 120° turn in one cell is not a road. **Recommendation
  taken (ask only with screenshots):** wheels follow this arc rather than the
  cell centres, so a road needs no extra bend cells.
- **Inside corners.** A 1.6 m car on the 4.9 m arc puts its inside wheel
  about 0.1 m past the bend cell's inner edge. So the cell on a bend's inside
  must hold nothing taller than a wheel's hub (a wall, a fence, a tree): the
  road builder keeps it clear. **Recommendation taken (ask only with
  screenshots).**
- **Width.** Every vehicle fits in one cell, so two cannot pass. **Recommendation
  taken (ask only with screenshots):** roads stay one cell wide, with a
  two-cell passing place at each roadside lamp on the world's roads between
  towns (`townsfolk` design 34 already spaces those lamps).
- **Bridges and fords.** A bridge deck is the road at its own height, so a
  vehicle crosses it as it crosses gravel; no weight limit in phase 5. A ford
  is driven at half speed by a car and full speed by a horse. **Recommendation
  taken (ask only with screenshots).**

### 3. How a vehicle sits on the road

Every vehicle is placed from points on its path, never from physics on
voxels:
- a four-wheeled vehicle stands on its two axles: yaw from the line between
  them, pitch from their heights on the road;
- a two-wheeled cart points from its axle to its horse's collar, so on a
  climb it tilts with the horse;
- a wagon's front axle turns on its kingpin toward its pole and horse;
- a horse stands on its own two ends, and walks its legs only while it moves;
- wheels turn by the distance travelled; a parked vehicle's wheels stand
  still.

The mockup places vehicles this way; the game does the same in `pbd-core` as
a pure function of the road and the distance along it, so single player and
the multiplayer server agree.

### 4. Track on a hex grid

- **Straights** run from cell centre to cell centre along one of the six hex
  directions, 2.833 m a cell. So every straight lies on the grid.
- **Curves** turn 60° and start and end on cell centres: `k` cells along the
  old heading and `k` along the new one. The chord is then `k·√3·W` at 30°
  to both, so the arc's radius is `k·√3·W`: 4.9 m for `k = 1`, 9.81 m for
  `k = 2`, 14.7 m for `k = 3`. Six curves close a loop; the mockup's loop is
  straights of 10, 5, 5, 10, 5, 5 cells and six `k = 2` curves (147 m round).
  **Recommendation taken (ask only with screenshots):** track comes in
  straights and the `k = 2` and `k = 3` curves; `k = 1` (4.9 m) is too tight
  for a train, and is a road's bend instead.
- A curve does not pass through the cells between its ends at their centres.
  Its cells are those its bed covers, and they belong to the track piece.
- **Switches** (not in the mockup) are a straight and a curve sharing their
  first cell.

### 5. Grades, and the cells under the track

A train cannot climb a layer a cell. **Recommendation taken (ask only with
screenshots):** the steepest track is one layer over six cells (1 in 17,
5.9%); one over eight is the default the track tool lays.

The world keeps whole cells. Track lies on a ballast bed whose top follows
the grade:
- cells under the bed taller than the layer below the bed are **cut** down to
  it, making a cutting one cell wide with the terrace's own cell sides as its
  walls;
- cells lower than that layer are **filled** up to it, making an embankment of
  whole cells;
- the bed's sloped bank makes up the part of a layer between, as a road
  cell's wedge does.

The mockup's loop climbs a one-layer terrace on its south straight and comes
back down on its north: half of each grade runs on a bank over the low ground
and half in a cutting through the high. The cut and fill are terrain edits by
the player (or a town), on the durable path, as a road's are.

### 6. Stations

A platform is a row of cells one layer above the bed, beside a straight. The
hex boundary leaves a gap of 0.4 to 0.6 m to a 1.8 m train, so the platform
carries a timber edge that reaches to 0.2 m. A hut, a bench and two lamps.
**Recommendation taken (ask only with screenshots):** platforms are built on
straights only.

### 7. Light

Everything placed in the world is lit as the terrain is (CLAUDE.md,
2026-11-18): `LitByField` and `LitLikeTerrain` on every vehicle, animal and
piece of track. Lamps on vehicles (a cart's lantern, headlamps, the engine's
lamp, a coach's windows) are lights in the cell light field like any lantern:
a cart in a cutting at night is as dark as the cutting, and its lantern
lights it and the cutting's walls. Headlamps are a cone of 30° to 20 m.

### 8. How vehicles are saved

Following `world-persistence` and `vehicles` (getting out of a vehicle
changes who is in it, never whether it exists):
- **A road vehicle** is a stored record: its ID, its kind, its pose in the
  body's frame (`WorldPos`, f64, frame named), its cargo, damage and owner,
  and the ID of the animal in its shafts, if any. It is written on the
  durable path when it stops, is loaded, or changes hands; not every frame.
- **A horse** stays an animal record (`animal-husbandry`); hitching writes
  both records in one transaction, so neither can be saved half-hitched.
- **A train** is an ordered list of its cars' IDs and its place on its track:
  the track piece's ID and the distance along it. Each car is a vehicle
  record of its own, so a wagon left in a siding is still there.
- **Track and its bed** are stored edits, laid by the player: the pieces
  (straight or curve, start cell, heading, `k`), and the cut and fill as
  ordinary cell edits naming their author.
- Nothing about this changes an existing save's format; it adds record kinds,
  read as absent by an older save.

## Risks / Trade-offs

- **One-cell roads jam.** Two carts meeting on a one-cell road cannot pass.
  The passing-place rule handles the world's roads; a player's road needs the
  player to leave room.
- **The 4.9 m bend is tight for a truck.** A real 3.0 m wheelbase on a 4.9 m
  arc tracks its rear wheels 1.0 m inside its front ones (0.6 m for the car's
  2.4 m), which would put them off the road. The mockup keeps both axles on
  the arc, as the game would. **Recommendation taken (ask only with
  screenshots):** both axles follow the drive line; there is no off-tracking.
- **Cuts and fills are big edits.** A grade across hilly ground cuts many
  cells. They go through the same edit path as digging, and are undone by
  the same means.
- **The mockup's light model** adds every lamp to every surface near it with
  no occlusion; the game's cell light field carries occlusion. The look is
  the target, not the method.

## Open questions

None for the owner yet. Every choice above is a recommendation taken (ask
only with screenshots); the mockup's screenshots go to the owner with M38 on
the tracker, and any choice they turn down comes back to this design.
