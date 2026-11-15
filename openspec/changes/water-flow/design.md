# Design: Luanti's liquid rule on the hex grid

## Context

**Luanti's rule** (`src/servermap.cpp` at `a10dc8d`,
`transformLiquidsLocal` and `get_max_liquid_level`):
- **Node states.** A node is a liquid *source*, a *flowing* node with a
  3-bit level (7 nearest the source, 0 the thinnest film) and a "flowing
  down" bit, or something else. That something is either *floodable* (air)
  or *neutral* (solid, or another liquid).
- **The queue.** Only nodes in a queue are looked at. An edit queues the
  node, and a changed node queues its neighbours. A tick takes at most the
  queue's length at its start (`liquid_update`, 1 s; `liquid_loop_max`,
  100,000), so water spreads about a node a second.
- **One node's update.** It reads its six neighbours: up, four sides, down.
  - A source stays a source.
  - A node with **two or more same-level or upper sources around it** becomes
    a source if the liquid is renewable. The source below is not counted.
  - Else a node beside a source takes level 7.
  - Else it takes the highest level its neighbours offer:
    - a flowing node above gives its level plus 4 (`WATER_DROP_BOOST`),
      capped at 7;
    - a side neighbour gives its level less 1, if it is not itself falling.
  - It is liquid only if that level is within `liquid_range` (8, so levels
    0 to 7 all count); otherwise it is air again.
- **Falling first.** "Flowing down" is set when the node below is air or
  flowing. A falling node offers nothing sideways, so water drops before it
  spreads.
- **Viscosity** (water 1, lava 7) slows how fast the level changes.

**Here today**: water is one static material at sea level (proposal).

## Decisions

**1. The hex neighbourhood is eight: up, six sides, down.**
- Luanti's order is kept: upper before same level before lower.
- A pentagon cell has five sides. The arrays hold eight and count what is
  there, as the topology's explicit 5/6 neighbour tables already do.

**2. Renewing takes three sources on the hex grid, not two.**
- On Luanti's square grid, two sources side by side share no neighbour, so a
  pair does not spread.
- On the hex grid, two adjacent cells share two neighbours. With Luanti's
  threshold of two, each shared cell becomes a source, which makes new
  pairs, and any two sources poured side by side flood every floodable cell
  on their level. The mockup shows this runaway (the preset "Luanti's
  threshold on hex").
- With three: a cell beside a straight row of sources touches two of them,
  and a cell beside a triangle of three touches at most two, so neither
  grows. A cell in a concave corner touches three and fills. This is the
  behaviour Luanti's two-of-four gives on squares: pools fill their corners,
  and lines stay lines.
- Recommendation taken (ask only with screenshots). The mockup has the
  toggle, so the owner can see both.

**3. Range and levels stay Luanti's: 8 levels, range 8.**
- Seven cells of spread is 20 m on the hex grid (2.833 m cells), where it is
  7 m in Luanti. That is a gentler sheet of water for the same 1 m layer.
- The mockup has a range slider (1 to 8) to judge it. Recommendation taken:
  8, Luanti's default, until the owner says otherwise from the mockup.

**4. A flowing cell's surface stands at (level + 1) / 8 of its 1 m layer.**
- That is Luanti's own drawing rule. A source and a falling cell are full.
- On the hex grid, each cell's top is one flat hexagon at that height, so a
  spreading sheet steps down by eighths. Luanti instead slopes its corners
  between neighbours, which is not a cell look.

**5. Only edits start flow; old worlds do not move.**
- Generated water is all source and is never queued. The queue starts only
  from an edit: a dig, a placed block, or a poured or removed source.
- So opening a world never changes it. A world made before this change has
  the same water cell for cell until someone digs beside it, and then only
  that water moves (CLAUDE.md, "Saved games survive every change").
- No generator change, so no new generator version.

**6. Sources are saved; settled flowing water is derived.**
- Placing or removing a source is a world edit. It takes the durable path
  and names its author. A source the renew rule makes is authored by the
  water. The rule only floods floodable cells (air), never a player's block,
  so the world's change never overwrites the player's.
- Flowing cells are not saved one by one. On load, every saved source
  beside air is queued, and the flow re-runs to the same settled state.
- This relies on settling being a function of the sources and the solids
  alone. The mockup checks it: "Settle from scratch" re-runs every source
  and compares the result with the incremental state, cell for cell.
- Water still moving when the game closes picks up where the sources put
  it. Nothing a player made is lost; at worst a falling sheet restarts.

**7. Save format, rendering and swimming (later, not in the mockup).**
- Save format: flowing water needs its level. Either a material per level or
  a level field. A new save format version is needed, with the old one
  readable: its water is all source.
- Rendering: the water top is today the top of the column's water run
  (`Column::water_surface`, the shader's water word). It must read the
  flowing level instead. That is a shader interface change, so the pipeline
  is validated when it lands.
- Swimming: the player swims in flowing water as in a source. Boats float
  on the sea's table only, as now.
- Cost: the queue is bounded per tick like Luanti's. It is measured on real
  hardware before it ships (perf rule).

**7a. The mockup draws water as M25's water sim does (owner, 2026-11-15).**
"sync m28 with these changes", after the owner asked of M25's Water sim
tab for "the frothy walls and edges like the river demo you had and show
flow direction" and found it "missing the wall/waterfalls part". The look
is shared, as the rule is: `docs/mockups/water-flow/water-look.js` holds
M25's water material (waves, foam, lamps, haze), its hex caps with a
foaming fall sheet and lip on every step down, and the sim's water builder
(each column's top water surface, a flow vector towards the neighbours
whose water stands lower, by how much lower, and a chevron on each flowing
cap behind a Flow arrows toggle). Both pages load it, as both load
`water-flow/liquid.js`. M28 feeds it its own lamps, a depth map baked from
the patch, and its fog. One fix rides with it, for both pages: a source is
drawn 6 cm below the level-7 water beside it, so the flow is worked out on
the head (a source an eighth above level 7) and no fall is drawn for a step
under 7 cm; arrows no longer run into a pool of sources. The rule is untouched: every preset settles in the
same ticks to the same cell counts as before. This is the mockup's look,
not a decision about the game's water renderer, which stays decision 7.

**7b. The mockups' water reads right from any side (owner, 2026-11-15).**
Looking at M28 on a phone at its default view, the owner wrote four notes.
They are the mockup's look, and the rule is untouched (every preset settles
in the same ticks to the same counts):

- *"The very front part of the cell doesn't have any geometry. Front lines
  of generation?"* M28 drew every column's side down to the patch's base,
  but a sea sheet ran on past the patch to the horizon at sea level and,
  near-opaque at a glance, covered those walls from 0 down to -6, so the
  near edge read as a grass lid on nothing. M25's sim skipped the bottom
  layer's outer sides outright, so its 1-layer front had no wall at all.
  Now the patch is cut like a cake: every side down to a stone base, a stone
  floor under it, the outer sea sheet gone (M28's sea is its own sea cells),
  and where water meets the patch's edge it stands as a still wall down to
  its bed (`simWater`'s cuts, drawn by `makeCaps` as a third kind of face).
- *"some zdepth issues with the waterfall/transparency sorting"*. The caps,
  lips and falls were one transparent mesh with no depth written, so its
  triangles blended in build order and a fall behind a cap could paint over
  it. `makeCaps` now draws a depth-only twin of the mesh first (the same
  shaders, so the same waves; render order 3.5, after every opaque thing),
  and the colour pass keeps only the nearest water at each pixel. Water
  behind water is hidden, as the game's single water layer is; the arrows
  test against that depth, so a fall in front hides the arrow behind it.
- *"Add button to toggle arrows on off, the game should have this too."*
  The toggle reads "Arrows: on" / "Arrows: off" with a lit dot when pressed,
  in both pages. The game's overlay is requirement "A flow-arrow overlay" in
  `specs/planet/water` (task 3.4).
- *"the edges also dont have flow ... we can see 1 arrow in a cell with 4
  pads."* A cap pointed only toward lower water, so the spreading front
  (dry beyond) and a ring at rest against a wall came out with no vector.
  The vector is now the way the water is going, down the head: toward lower
  water; away from higher water that reaches this layer; toward a dry cell
  it can spread into (open at this layer, and a level above 0 to give) or
  fall into (open at this layer and the one below). A source points only
  where water leaves it, and a still pool (every head equal) has none.
  Basin: 18 caps with arrows where there were 6; spring: 230 where there
  were 186, with every flowing cap that has somewhere to go carrying one.

**7c. Flow is shown by panning a water texture, Tenebris C's way (owner,
2026-11-15).** "Add UV scrolling and water texture or shaders to show the
flow map. This is how tenebris c does it." Read at `RubenTipparach/tenebris`
commit `ef9865166349c7a3c4ae09acdd20640daf85b4fd` (2026-08-15, "Merge pull
request #89"), cloned read-only:

- `tenebris-c/src/client/hex_mesher.c`, `bake_water_flow_uv`: at mesh time
  each water cell's top-cap vertices get the cell's flow direction, from
  `world_water_flow_direction` (`src/core/world_water.c`: the cell's centre
  minus the mean position of the source cells it descends from, normalised),
  projected onto the tile's tangent plane and written into the vertex's UV
  channel. Side walls, bottom caps and cells with no origin (generated sea,
  sources) get (0, 0): "spawn blocks stay static, flow blocks pan".
- `tenebris-c/src/shaders/water.glsl`, `water_fs` (the desktop path): no
  texture. The surface is a three-octave gradient-noise fBm (`gnoise3`,
  `fbm3`) whose finite-difference gradient bends the normal. Flow moves the
  noise's sample point: on a cap, `flow_pan = -flow_world * u_time * 0.35`
  (weighted by how horizontal the face is); on a vertical face, a shift along
  the radial by `u_time * u_flow_speed_falling` (`flow_uv_speed_falling`,
  1.0 in `assets/config/water.yaml`), so the pattern streams down a fall.
  One phase, no reset: the vector is constant over a cell, so the pan never
  stretches the pattern and no two-phase blend is needed.
- `tenebris-c/src/shaders/water_mobile.glsl`, `water_mobile_fs` (the phone
  path): the same idea with a texture, `assets/textures/water_normal.png`
  (128 px, made by `tools/gen-water-normal.py`), sampled triplanar in two
  layers at two scales and drift speeds, the fall shift added to the sample
  point on vertical faces (no cap pan on mobile).
- `tenebris-c/water.md` section 8 plans the rest: the cap's UV scrolled along
  the net flow at `flow_uv_speed_per_level` per level of drop (0.25; parsed
  in `config.c`, not yet used by the shader), a falling side scrolled from
  V 1 to 0, still and flowing atlas slots.
- `tenebris-c/src/client/renderer.c`, `renderer_draw_water_flow_debug` and
  `flow_arrow_visit`, toggled by F5 in `src/client/main_web.c`: a debug
  overlay of a red line arrow on every water cell with a flow direction, a
  hair above the cap, depth-tested and not written.

The mockups do it the same way with a pixel texture. `waterMat` samples
Tenebris's own water tile (`tenebris-rs/assets/textures/water.png`, 16 x 16,
three colours; byte-identical in `docs/mockups/planets/tenebris/water.png`),
embedded in `water-look.js` as its 256 colour indices so no file travels with
it, nearest-sampled, 2 m a tile. The three indices are drawn as three tones
of the water they are on (its own colour, a ripple lifted toward the sky, a
glint lit as the foam), so the pattern is palette-true and lit by the lamps
at night. Each cap pans the tile along its own flow vector at 1.6 m/s per
unit of flow, the tile's streaks laid along the flow; a falling sheet pans
it down at 2.4 m/s, streaks standing; a still cap and a cut stand still. As
in Tenebris, the vector is constant per cell, so one phase serves. The fall
froth now comes from the tile (its old hash streaks are kept only where a
page asks for no texture), the cap streak foam is lighter, and the pattern
fades out between 45 and 120 m, where a 12 cm pixel would shimmer. The sea
(`tex` 0) is unchanged. Measured in M28 (SwiftShader, two frames a second
apart): a flowing cap changes 10.8 % of its pixels by more than 24 levels,
still sea 0.4 %.

**7d. The game's flowing water (proposed, not built).** The game's cap
already has a flow hook (requirement "The water surface streams along a
per-cell flow"). When flowing water lands, its look is Tenebris's: the
flow vector baked per cell at mesh time, the top's pattern panned along it
by speed, a falling face panned down, still water still, and a flow-arrow
overlay to check it (tasks 3.4, 3.5). Proposed only; no game code until the
owner approves the mockup and asks.

**7e. Water is meshed per cell, as Luanti's liquid mesher does (owner,
2026-11-15).** "Same issues as before for the leading edge, also theres no
flow maps on the edge as well and no walls, note that you missed a critical
part of the luanti/minetest water implementation that shows the water
falling", and "water should generate blocks vertically as it falls too".
The flat caps of 7a to 7c drew one cap per *column*, at its top water, with a
sheet from that cap down to the next column's top: every cell under the top
of a column was never drawn, so a falling column showed as one flat cap
hanging at the ledge's height a cell out from the cliff, and the front of a
sheet ended in a flat, wall-less edge.

*The rule was not the problem.* Measured on M28's spring (a Node run of
`water-flow/liquid.js` with the page's ground, no page code): the first
falling cell over the cliff appears at tick 4 (`(3, -1)` at layer 6, level
4, `down` set); by tick 6 the same column holds falling cells at layers 6, 5
and 4, by tick 8 at 6 to 2, by tick 10 at 6 to 0, the column growing one
layer a tick, every cell below the top at level 7 (its upper neighbour's
level plus `WATER_DROP_BOOST`, capped) with `down` set, standing on the
plain's own sheet. Settled (tick 40), twelve columns fall the six layers
from the ledge to the plain. Luanti's rule already makes real falling cells,
block by block; they were simply not drawn. The rule is unchanged, and every
preset settles in the same ticks to the same counts.

*Luanti's mesher*, read at `a10dc8dbff3b0af8295ae55df44f65b1d9ee92c7`,
`src/client/content_mapblock.cpp` (LGPL-2.1-or-later):
- `drawLiquidNode` draws one *flowing* node (`NDT_FLOWINGLIQUID`):
  `prepareLiquidNodeDrawing`, `getLiquidNeighborhood`,
  `calculateCornerLevels`, `drawLiquidSides`, then `drawLiquidTop` unless the
  node above is the same liquid, and `drawLiquidBottom` if the node below is
  neither the same liquid nor solid. A *source* (`NDT_LIQUID`) is drawn by
  `drawSolidNode<true>` as a full cube, a face wherever the neighbour is not
  the same liquid, not solid and not `CONTENT_IGNORE`.
- `prepareLiquidNodeDrawing`: `top_is_same_liquid` (the node above is the
  source or flowing form) and `draw_bottom`.
- `getLiquidNeighborhood`: the 3 x 3 nodes round it on its own layer, each
  with a `level` in node units (-0.5 the floor, +0.5 the top): a source
  +0.5, a flowing node `-0.5 + (level + 0.5) / range` (range 8, so level 7
  stands at 15/16 of the node), anything else -0.5; and whether the node
  above that neighbour is liquid. `CONTENT_IGNORE` is left out.
- `getCornerLevel`, for each corner over the 2 x 2 nodes sharing it: if any
  of them has liquid above it, or is a source, the corner is full (+0.5).
  Else the flowing nodes' levels are averaged, and air is counted; two or
  more air of the four pull the corner to the floor (`-0.5 + 0.2 / BS`, 2 cm
  up). No flowing node and fewer than two air: the middle (0).
- `drawLiquidSides`, for each side: no face against the same liquid, unless
  this node has liquid above and that neighbour does not; no face against a
  solid. The face runs from this node's two corner heights (or the top, if
  liquid is above) down to the floor, or, against the same liquid, down to
  the corner heights. These are the walls at the front of a sheet and the
  sides of every falling node.
- `drawLiquidTop`: a quad through the four corner heights. Its flowing
  texture is turned to the flow: `dz` is the corners on -Z less those on +Z,
  `dx` those on -X less those on +X, so the direction is down the corner
  gradient; a symmetric top gets (1, 0). Without reflections the normal is
  `(dx, 1, dz)` normalised. The sides carry the flowing texture, which runs
  down.
- `drawLiquidBottom`: a flat face at the floor.

A falling node therefore has liquid above it, so all its corners are full,
it draws no top, and its sides stand the whole node down to its floor: a
falling column is a stack of whole blocks, each drawn with its own sides,
and the node at the top of the fall, with nothing above it, slopes from the
lip down to its own level.

*On the hex grid* (`PBDWater.liquidMesh` in `water-flow/water-look.js`, which
`simWater` now calls, so M25's Water sim and M28 both take it):
- A cell has six corners, each shared by three cells: the cell and its two
  side neighbours on either side of that corner (corner *i* lies between
  sides *i - 1* and *i*). `getCornerLevel` runs over those three: full if any
  has liquid above or is a source; else the mean of the flowing levels; air
  pulls the corner to the floor at two or more of the three (Luanti's "half
  or more" of the cells round a corner); a cell outside the patch is left
  out, as `CONTENT_IGNORE` is.
- Heights are Luanti's: level *L* stands at `(L + 0.5) / 8` of its layer, a
  source and any cell with liquid above it at the full layer. (Decision 4's
  `(level + 1) / 8` and 7a's 0.94 for a source are retired with the caps.)
- The top is six triangles from the corners to a centre at their mean, drawn
  unless the cell has liquid above. Its normal follows the slope.
- A side is drawn per Luanti's `drawLiquidSides` across each of the six
  edges: against air down to the cell's floor; against the same liquid only
  where Luanti draws it. The patch's own boundary is the one change: Luanti
  draws no source face against `CONTENT_IGNORE`, but the mockups cut the
  patch like a cake (7b), so every water cell at the boundary draws its side
  there, down to its floor, for sources and flowing cells alike.
- The bottom is drawn where the cell below is air (Luanti's `draw_bottom`).
- The flow vector is Luanti's: down the corner gradient. On hex it is the
  least-squares slope of the six corners, `-sum(h_i c_i) / (3 R^2)` with
  `c_i` the corner offsets; a symmetric top has none (a pool stands still,
  where Luanti turns it to +X). It drives Tenebris's tile and the arrows: the
  tile pans along it, faster on steeper water; the arrow lies on the slope.
- Walls: a flowing cell's sides run Luanti's flowing texture downward, the
  Tenebris tile panning down with froth, and every falling cell is a whole
  block of it; a source's sides stand still. A falling cell's open sides
  also carry a down-pointing arrow, so a fall shows its flow with the
  arrows on.
- The depth pre-pass, the night lamps and the arrows' toggle are unchanged;
  the column-top record (`SW.cells`) is kept for M28's depth map.

The game's water mesher should do the same (requirement "Flowing water is
meshed per cell from its corner levels", task 3.6). Proposed only; no game
code until the owner approves the mockup and asks.

**8. The game's code is written from the rule, not translated.** The mockup
is a line-by-line port of LGPL-2.1-or-later code and says so. The Rust
version is written from decisions 1 to 6 and Luanti's rule as described in
Context, and the mockup is the reference for behaviour. Recommendation taken
(ask only with screenshots). The owner can choose to take the LGPL code into
the game instead, and that is the owner's call.

## Risks / Trade-offs

- **Infinite water.** Renewing makes infinite water, as in Luanti. With
  three sources it needs a real pool, not a pair, to start.
- **Large floods are slow.** A flood moves at a cell a second, by design. A
  dam break across a valley takes minutes, which is the look Luanti players
  know.
- **The sea is a source everywhere.** Dig a channel from the sea and it
  fills to sea level, then flows on down any slope below it. Draining the
  sea is impossible, which matches the sea's rails.

## Migration Plan

- Mockup first (tasks section 1). The game waits for the owner's approval of
  the mockup, then a request (CLAUDE.md, the mockup tracker).
