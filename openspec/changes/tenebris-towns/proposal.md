# Proposal: Tenebris towns, with interiors, proper stairs and collision

## Why

**The owner's request: "Make html mock up of cities. I want interiors, proper
stairs and collisions. The tenebris people have sort of medieval level tech so
their stuff is not super advanced."** Then, on the first mockup: "Roof is
clipping here and the move stick isn't working when I tap on my phone. Can you
make a couple of different kits? Like a small straw huts or dirt huts or wooden
houses, or brick houses. Try to mix with different brick types, stone, marble,
clay." And on the kits: "I would think different biomes would have different
building materials. Do you have sets like igloos, ice castles? What about
deserts? Jungles with big tree houses and wooden bridges? I also want to see
what a small town by itself would look like as well. Also a swamp would have
like houses easier on wooden platforms like the bayou."

The world has terrain, trees, water and weather, and nothing anyone built.
Three things stand between the engine and a town you can walk into:

1. **A whole cell is too big to be a wall.** A cell is 2.833 m flat to flat
   (CLAUDE.md's gold standard). A house built from solid cells has walls
   2.8 m thick, and a house one ring of cells across has one cell of room
   inside it. Interiors need walls much thinner than a cell.
2. **The walker cannot climb a proper stair.** `walking.rs` climbs anything up
   to `step_height` (1.05 m) in a single tick, snaps down only within 3 cm, and
   drops all tangential motion when a move is refused. On a real stair
   (0.19 m risers on 0.35 m treads) that is 22 jumps of the eye a second going
   up, a fall at every riser coming down, and a dead stop at every door jamb.
   The mockup measures all three (design section 5).
3. **Collision has no thin geometry.** `PlanetContact::stand` answers a floor
   and a ceiling per hex column. A wall on a cell edge, a door, a table or a
   stair rail is not a column.

## What changes

- **A mockup came first, as asked.** `docs/mockups/towns.html`, published at
  <https://claude.ai/artifact/59zS9ERBTkrVrzPedfoUL2>, is a three.js walled market town on
  three terraces: an inn with a hall and rooms upstairs, twelve houses, a
  smithy, a moot hall, a three-storey keep, two wall towers, a quay, a market
  and sixteen townsfolk. Every room can be entered. The walker is
  `walking.rs`'s, and a switch (T) swaps between today's rules and the
  proposed ones on the same stairs. The collision near you can be drawn (C).
- **Buildings are cut to the cell, not scaled to it.** Walls sit on cell
  edges (0.3 m timber, 0.5 m stone), one wall per shared edge. Floors are
  per cell. A storey is three layers: 3 m floor to floor, 2.8 m clear. Doors
  are 1.0 by 2.2 m. The town wall and the keep's footing are whole masonry
  cells, as a real curtain wall is 2 to 3 m thick.
- **Three stairs, each landing on a layer line.**
  - A **straight flight** over two cells climbs one storey in 16 risers of
    0.1875 m on 0.354 m treads (28°). It needs a cell before it and a cell
    after it, in line.
  - A **newel stair** in one cell turns once per storey: 15 winders of 0.2 m.
    A metre of rise turns 120°, two edges, so a doorway at any layer line is
    centred on an edge. It ends in a 30° landing and a rail.
  - **Street steps** climb one 1 m terrace in one cell: 6 risers of 0.167 m on
    0.472 m treads.
- **Collision gains a second primitive and three rules.**
  - *Thin solids*: a convex outline in plan, extruded over a height range
    (walls, jambs, doors, rails, furniture, posts, people), beside the column
    runs `stand` already answers. *Surfaces*: pieces whose top is a function
    of position (stairs, roofs).
  - A stair's walking surface is its **pitch line**. The treads are drawn,
    and the eye climbs smoothly with no camera easing.
  - A grounded walker is **held to a floor up to 0.35 m below**, so going
    down a stair never leaves it.
  - A blocked move **slides**: the body is pushed out along the face it hit,
    and only the motion into the face is lost.
- **Doors are world state.** E opens and closes one. Open or shut, it is a
  world mutation and goes through the durable save path.
- **Kits: what a building is made of, not how it is cut.** Eleven: straw hut,
  mud hut, timber boards, half-timber, red brick, buff brick, clinker brick
  (Flemish bond), fieldstone, ashlar, clay (a flat roof with beam ends) and
  marble (columns at every corner). A kit sets the wall faces per storey, the
  corner posts, the roof and gable, and the floor; walls on edges, floors in
  cells and the stairs are the same for all of them. Huts are the one exception
  to the storey: one storey of two layers, 1.9 m doors, a cone or turf roof.
  The town mixes the house kits; a hamlet of huts stands outside the east wall.
- **A settlement per biome, from that biome's materials.** Six, switched in
  the mockup: the walled town and a village standing by itself in the fields,
  a desert town round an oasis (sandstone houses with roof stairs, mud-brick
  domes, a domed caravan hall), a tundra camp (igloos, a granite longhouse under
  turf and snow, a castle of ice), a jungle village on platforms in three kapok
  giants joined by rope bridges, and a swamp village of alder stilt houses on
  boardwalks. Each takes its materials and plants from the biome catalogue in
  `docs/game-design.md`, so no settlement borrows another biome's.
- **Four new pieces carry them:** a dome (its underside the ceiling), a raised
  deck on piles, a walkway between two points (a rope bridge that sags, a
  boardwalk, a jetty), and an outdoor stair between two points. All four
  collide as floors, ceilings and rails the way the rest do.
- **Roofs never cut into each other.** The first mockup's clipping was two
  houses in touching columns: on the hex grid their plans interlock by half a
  cell, and the eaves overlapped. Every roof's plan, eaves included, is now
  checked against every other when a town is laid out, and the keep and towers
  count as roofs.

## Impact

- **Specs.** `world/settlements` is new. `player/walking` gains the pitch line,
  the hold-down and the slide.
- **Code.**
  - `pbd-core`: a `settlement` module (pieces, the stair set, the layout of a
    town as data) and thin solids and surfaces in the contact query, with the
    stair heights as functions both the walker and the tests read.
  - `pbd-app`: piece meshes built from each cell's real corners, the three
    walker changes, doors and their save records.
- **Nothing here changes terrain, digging or the flight model.** Street steps
  and buildings sit on the existing terrace grid.
- **Out of scope:** the player building with these pieces, townsfolk who do
  more than walk a path or work in place, trade, and where towns go on the
  planet.

## Questions for the owner

1. **Walking speed indoors.** The engine walks at 8 m/s, which crosses a
   2.5 m room in a third of a second. Keep it, or add a slower walk?
2. **What Tenebris people look like.** The mockup's figures are blocky
   placeholders in tunics.
3. **Towns hand-made, generated, or both**, and in which biomes?
4. **Should the player be able to build with these pieces** (walls on edges,
   floors, the three stairs) through dig and place?
5. **Stairs:** is the pitch line the feel you want, or do you want to feel each
   tread? The mockup's T key compares them on the same flight.
6. **Roofs:** should they be walkable? The mockup lets you stand on them.
7. **Kits:** which should be in the game, and which belong to which places
   (huts outside the walls, brick and marble for the rich)?
8. **Biomes:** are these the right settlements for each? Mountains, beach and
   ocean have none yet, and the other planets' biomes are not Tenebris's.

## Status

Proposed. The mockup is built and published for the owner's verdict. No Rust
has been written. The next step is that verdict, then `/opsx:apply`.
