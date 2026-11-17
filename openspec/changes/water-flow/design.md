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

**7f. Rivers run on the rule too; M28 folds into M25 (owner, 2026-11-15).**
"water that flows - make sure the rivers use that too, so merge these
artifacts together so we get it more consistent". M25's river, dam stream and
pond, and marsh pools now run on `liquid.js` and draw with `liquidMesh`
(`ocean-environments` design 12), and M25's Water sim tab carries M28's
patch, presets and controls; `water-flow.html` points there. The rule is
unchanged and every preset settles as before.

What a river asks of the rule, measured on M25's river (a Node run of
`liquid.js` on its cells): a flowing reach runs eight cells from where it is
fed and dries beyond, and a step down feeds the reach below to the lip's
level plus three for one layer, to the full seven for two or more. A river on
cells therefore steps a layer at most four cells (11 m) apart, or two layers
at most seven apart; the river's old meanders, a layer every 22 to 28 m,
dried 17 m below each step. Still water (a spring, a pool, a pond, the bed
under a deep reach) is sources, and a reach's lip is kept one layer deep so
the still water of one reach never touches the flowing water of the next,
where renewal could turn the river to sources. The game's river pass
(`ocean-environments` task 8.1) has to lay rivers out the same way: proposed
only.

**8. The game's code is written from the rule, not translated.** The mockup
is a line-by-line port of LGPL-2.1-or-later code and says so. The Rust
version is written from decisions 1 to 6 and Luanti's rule as described in
Context, and the mockup is the reference for behaviour. Recommendation taken
(ask only with screenshots). The owner can choose to take the LGPL code into
the game instead, and that is the owner's call.

## In the engine (owner, 2026-11-15: "commence implementation of all env changes")

M25 and M28 are approved, and the request to build came with them. How the
rule goes into the game, in the order it is built (branch `claude/env-water`):

**E1. A flowing cell is a material.** `Material::Flow0` to `Flow7` and
`Fall0` to `Fall7` (codes 64 to 79, far from the blocks' run) are flowing
water at a level, still and falling; `Water` stays the source. The column
keeps two bytes a layer, so the tier costs nothing more to hold or clone.
`Material::is_water` is what every reader asks (the walker swims in both,
decision 7; the renderer's water code is both). A flowing top's surface is
Luanti's `(level + 0.5) / 8` of its layer (`liquid::top_fraction`).

**E2. The rule is `pbd_core::liquid`**, written from decisions 1 to 6
(decision 8): `transform` is a pure function of a cell and its eight
neighbours (`None` for a pentagon's missing side or off the region), and
returns the new state and which neighbours to queue, in Luanti's order;
`Queue` is the unique queue and the tick, at most the queue's length and
`LOOP_MAX` a tick. Held to the mockup by a fixture the mockup's own
`liquid.js` writes (`tools/water_flow_fixture.mjs`): all eight presets
settle in the same ticks to the same field, cell for cell. Water enters
only air: a block, a lamp or anything else is neutral, so the rule never
removes a thing a player made.

**E3. It runs over the column tier**, once a second (Luanti's
`liquid_update`): the tier is the `Grid`, a cell is a slot and a layer, the
rim and every cell off the tier are neutral. Each tick's changes go in as
an edit does (the changed records and their neighbours repacked, the
contact and the fine set's version moved), without the relight: water does
not stop light in the field.

**E4. Saves (this replaces decision 6's "derived").** Every change the rule
makes is an edit authored by the water (`@wwater:flow/0`), with no slots:
a source as `Water`, a flowing cell as its `Flow`/`Fall` code, so its level
is saved. Derived was the plan, but the tier is regenerated from the edits
each time it moves, and a derived field would re-run on every rebuild. The
water's edits do not pass the world's yield set: water filling a hole the
player dug is what water does, and it only ever fills air. Opening a world
queues every cell of the tier that holds a water edit, and its neighbours,
so water still moving at the close picks up and a settled field changes
nothing. A save from before has no water edits, so nothing is queued and
every cell is where it was (decision 5; tested).

**E5. Pouring.** There is no bucket, and nothing yields a water block,
though the place path places any held block and water has its thumbnail.
Recommendation taken (ask only with screenshots): the place path is the
pour (a held water block places a source, which queues the flow), and a
debug key, F6, pours one source where the place ray lands, through the same
edit path. A dig or a placed block next to water queues the flow the same
way. Captures pour with `--pour`.

**E6. The water mesher** (task 3.6) is a CPU mesh of the tier's water,
rebuilt when the fine set's version moves: Luanti's `drawLiquidNode` on hex
as design 7e has it (corner levels from the three cells round a corner,
tops through them, sides where water meets air, falling cells whole blocks,
bottoms over air, the flow down the corner slope). It leaves out what the
sea sheet already draws: the top of a column's water at sea level where the
cell's record is under the sea. One mesh, Tenebris's water tile
nearest-sampled and alpha-blended, unlit with the tier's light field in its
vertex colour (sky by the daylight, the block light warm), so it is dark at
night and lit by lamps. The mesh exists from the first frame (an empty
placeholder), so the loading screen builds its pipeline.

**E7. The flow is shown as design 7c says** (task 3.5): each frame the
tops' texture coordinates move along the cell's flow vector, faster on a
steeper slope, and a falling face's move down; a still cell and a source
the water does not leave stand still. The flow-arrow overlay (task 3.4) is
a second mesh of arrows over every flowing top and down every fall, hidden
unless F5 or the player setting "Flow arrows" (off by default) turns it on.

**E8. What was built, and what was not (2026-11-15).** All of E1 to E7 is
built and tested in `pbd-core` and the tier (see tasks). Not measured: the
frame cost (3.3). Not checked: the pan's "two frames a second apart" (3.5).
A set built while water is ticking would land without the ticks in flight,
so the tick waits while a set is in flight; the existing "undone when it
lands" behaviour of edits is unchanged. The sea floods a channel dug from it
as sources to sea level (`volumetric-water` phase 2) rather than as M28's
"Channel from the sea" flowing sheet: in the game the sea is the sea's
table. Recommendation taken (ask only with screenshots).

## Second pass (lead's request, 2026-11-15)

**E9. One water: the game's water is drawn by the sea's own fragment.** The
first cut drew poured water with an unlit tile mesh, and it read as dark
confetti beside the sea. Now the mesher's triangles go to the water pass as
a storage buffer (three lanes a vertex: position and the sky's share; the
face's normal and kind; the flow in the face's tangent frame) and are drawn
by a second vertex entry, `vertex_flowing`, into the cap's own fragment, in
the cap's render pass and depth. So a pool and the sea are one water: the
sea's colour by depth, its light by the hour and the cover, its sky
reflection, fog and glint, and no seam where they meet. The flow streams the
sea's waves along a top (Tenebris C's desktop way, 7c, the cap's flow hook)
and down a falling face, which carries white water; a still face neither.
A face seen from behind from dry air is discarded (its front draws the
block); from inside the water it is the murk, as under the sheet. The tile
is gone from the game; it stays the mockups' look. Recommendation taken
(ask only with screenshots).

**E10. The walker swims in what is drawn.** The walker's water was the
height field's sea against the untided sheet. It now asks the column first:
the water the body is in, a pool or a flowing cell at its own level, and the
sea's water at the drawn sheet with the tide on it; a dry column under open
sky still swims where the tide stands over its ground (a beach at high
water); off the tier, the height field's sea to the tided sheet.

**E11. The wooden bucket.** The way a player uses water, after E5's debug
key: two goods appended to the goods table, `bucket` (a wooden bucket, 4 to
a slot) and `water-bucket` (a bucket of water, 1 to a slot), with 16 px
icons from `tools/gen_item_icons.py` in the fence's wood. Made at the
workbench from 3 planks and 1 rope (`recipes.ron`, `bucket`). Held, the
right button:
- with an empty bucket, aimed at a source (the ray stops at water for it,
  as it does not for a block), takes the source and the bucket is full;
  flowing water cannot be taken, as in Luanti;
- with a full bucket, pours a source where a block would be placed, and the
  bucket is empty.
Both are one edit and one hands change on one line, the place path's own
transaction (`Hands::Swap`), so the source taken and the bucket filled reach
the disk together. The rule is `pbd_core::liquid::bucket`, tested.

## The owner's notes on falls (2026-11-15)

**E12. No rain on vertical water.** The owner: "when raining - no droplets
on vertical water." The rain's rings are scaled by how much a face looks up
(`smoothstep(0.7, 0.95, horizontal)` in `water.wgsl`), so a fall's or a
wall's face takes none and a top takes them all.

**E13. Froth on falls.** The owner: "vertical water should have a bit froth
to it, probably something easy to do via shaders." A falling face carries
white streaks of froth, noise stretched down the face and scrolled down it
with the fall, denser at a fall's lip and foot (the mesher's froth weight,
`flow_uv.x` on a falling face); a still wall keeps none.

**E14. Splash at a fall's foot.** The owner: "can you also add some particle
fx to bottom of the waterfall? more fx based on distance it fell". The mesher
records every fall's foot (water not falling with falling water on it) and
how many layers fell onto it; each frame, spray drops are thrown up and out
and fall back, and a low white mist puffs, both more, higher and wider for a
taller fall (3 + 6 per layer drops up to 160, a puff per two-thirds of a
layer up to 24), at most 4,000 quads, out to 70 m and faded over the last
15 m, lit by the sky's share and the hour. A pure function of time: nothing
is stored. One unlit alpha-blended mesh, born at startup, so the loading
screen builds its pipeline.

**E15. The rivers' water is the rule's.** The generator seeds only sources,
on their own bed and never over open air; a version 8 tier's build runs the
rule over them until nothing is queued and a sweep of every water cell
changes nothing (`ColumnTier::settle_rivers`), so falls, sheets and riffles
are exactly what the rule keeps (`a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`).
Derived, never saved; older worlds are not settled. The banks are held on
every side (the water's top is a metre under the lowest ground round the
valley's rim), yet a fall's foot can still spread its eight cells past a
river's valley (fixed in the fifth pass, E16).

## Fifth pass (lead's request, 2026-11-15)

**E13, again: froth as streaks.** The fourth pass's froth read as camo
blotches. It is now thin bright vertical ribbons: two octaves of gradient
noise stretched about seven times along the fall, scrolled down at 4 m/s,
thresholded to narrow bands, over a soft white veil, with a churned band at
the lip and the foot (the mesher's froth weight). The noise's coordinates
are taken modulo 128 m first: at planet-sized coordinates the noise's hash
loses its precision and the streaks went flat.

**E13, a third time: the lip and the speed.** The owner, on the after shot:
"can we have the edge transition into froth? and somehow show that it is
speeding up in velocity/flow in to the wall". The mesher gives each top
corner a lip weight (one where a side's water falls, falls from under it,
or this cell's own water falls below) and each falling face's corners how
far under their fall's lip they are, in the flow lane's third component.
On a top, the last metre or so before a lip whitens into streaks along the
flow that run faster toward it (two phases of the flow half a cycle apart,
blended, so the speed never shears the pattern). On a fall, the ribbons are
read at the time the water left the lip, free fall from 1 m/s,
`(sqrt(v0^2 + 2 g d) - v0) / g`, so they accelerate down the face and
stretch with sqrt(2 g d): clear just over the lip, white ribbons lower down,
the churn at the foot (`froth-lip-before-after.jpg`).

**E16. A river is held: levee and sealed rock.** Measured on four tiers
over falls (`a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`), the
water that spread past a river's banks was not on the surface: it ran down
through caves the worms open under and beside a river's bed, and spread in
them, tens of metres from the river and tens of metres under the ground. Two
changes to version 8 (not yet in any played save): every column within a
river's banks plus 15 m keeps its rock, cave-free, from four metres under
the sea's level up (`rivers::sealed_from`, applied after the worms in
`column::generate_edited`); and a 3 m levee round a valley stands a layer
over the reach's water and its fall, its banks rising to it and its outside
falling away at one in one (`rivers::carve`). With the levee, the water's
top goes back to a layer under the lowest of six points round the rim. A
plunge pool's extra depth eases out up the bank, so the seam report holds
at none of 4,000 edges. The test now
asserts no flowing water over the sea's layer outside a river's banks, on
four tiers. Water at the sea's layer in a sea cave is the sea's and is not
counted.

**E17. The capture rig flies anywhere.** `--fly` takes `--yaw` (compass
degrees, 0 north, 90 east) and `--pitch` (degrees, negative down) beside
`--at` and `--height`, so a capture can look down on a river from the air.

## The fall look, in a mockup first (owner, 2026-11-15)

Two in-game fall looks were rejected (pass 5, `c620183`: "all the afters are
bad"; the bowed strands, `b9c81ec`: "too smeared, theres also a geometric gap
at the edge"). From here a fall's look is approved in a mockup before the game
changes (CLAUDE.md, "Waterfalls are designed in a mockup first"). The mockup is
`docs/mockups/waterfall.html` (`waterfall/waterfall.js`). It shows the game's
test case (`--pour 1 --pour-ahead 12 --pit 9`: a source beside a 9-deep pit of
seven cells) and a 2-layer step, on 2.833 m cells with 1 m layers. It shows them
by day and at night under three lamps, with toggles for rain and splash. Its
screenshots are in `docs/screenshots/mockup-waterfall/`. Not built in the game.

The froth is a GLSL port of `water.wgsl` at `dc0c5cb`, the falling-face branch
(`in.kind == 2u && horizontal < 0.5`). It keeps the same `gnoise3` (integer
lattice hash), the same `fbm3`, the same tangent frame (body X projected off the
face) and the same constants. Variant A is that port unchanged.

One finding from the port: the froth's `along.y*0.35` was meant to stretch
it down the fall, but it never could. On a vertical face the radial lies in
the face's own plane, so it is already read at 5x through `tangent_u` and
`tangent_v`. The 0.35 term adds about 0.1% to the noise's rate down the face.
The approved blotches are round for that reason. B stretches that real look.

| Variant | What it is | Game change to port it |
| --- | --- | --- |
| A Today | dc0c5cb as it is | none |
| B Stretched | the same pattern, its height below the lip divided by `stretch` (default 2.0, range 1-4); a speed multiplier on the froth's time terms (default 1.0) | in the froth branch, read the noise at `p' = advected - radial*(h*(1 - 1/FALL_STRETCH))`, where `h` is the height above the lip's top (the mesher puts the lip height, or the metres under the lip it already writes in `flow_uv.z`, on each vertex). Multiply `time` by `FALL_FROTH_SPEED` in `advected`'s radial term and in the `+ time*0.6` term for the froth only |
| C Leaning | B on a sheet that leans out along the flow: offset `lean * profile(depth/drop)` with `profile(u) = (sqrt(u+0.05) - sqrt(0.05)) / (sqrt(1.05) - sqrt(0.05))`, default lean 0.8 m; nothing above the lip's ground moves, so the top and the sheet share their rim vertices (no gap) | `water_flow.rs`: move each falling face's vertex out along the fall's flow direction by that offset, the same for every face at one depth (the column's corners stay joined). Carry the unmoved position as one more vertex attribute and read the froth there, so the pattern is B's, bent |
| D Lip + speed-up | C, plus one amount (default 0.5): (1) the froth is read on one frame per fall (the face toward the flow) at the point where the flow leaves the falling cell, so the top and the face are one pattern across the edge; (2) the top whitens over its last 1.5 m (`smoothstep(-1.5, 0, s)*amount`); (3) the path below the lip is warped to time of flight, `c(s) = mix(s, (sqrt(1 + 2*3*s) - 1)/3, amount)` (1 m/s at the lip, 3 m/s^2), so the blotches lengthen as the water speeds up; on the top, `c = s - 0.6*amount*s^2` | mesher: per top vertex, the ray from the source cell's centre to where it leaves the falling cell (exit point and signed distance `s`), and per face vertex `s` = metres under the lip; shader: the froth's noise point `(exit.x, lip - c(s)/stretch, exit.z)` in body space, the ramp on tops |

Splash and mist (every variant, toggle): drops and a mist sized by the fall's
height H. There are about 26H drops and 7H mist puffs, thrown at
`sqrt(H)`-scaled speeds. They start on the open side of the column, past the
hex's corner radius from the leaned foot. They are never drawn below the
pool's surface + 5 cm ("only outside"). Rain rings are multiplied by
`smoothstep(0.7, 0.95, horizontal)` on the face's own normal, as in the game,
so no falling face takes any.

What the mockup does not port: the game's light and tone mapping (the
sand, sky light and white are matched to the screenshot by eye), the
screen-space reflection, the refraction offset, the distance fog and the
game's sky light per vertex (a depth ramp stands in). The noise is read at
a stand-in body position (a 2400 m planet at the capture's 24.1 N), so A
draws the game's kind of blotches, not the same ones in the same places.

## Smooth water between ticks, in a mockup first (owner, 2026-11-15)

The owner: *"for water is there a better way to lerp the sim? so it doesn look
like it just pops in?"*, then *"yea mock it up"*.

**Today.** The water rule ticks once a second (`liquid::TICK_S = 1.0`). Each
tick the CPU rebuilds the whole water mesh (`water_flow.rs`), and
`vertex_flowing` draws it as given. A cell that turns wet appears whole in one
frame. A level that moves steps 1/8 of a layer at once. A new fall stands at
full height at once, and a dried cell vanishes.

**Proposal.** The rule, its tick, and what is saved are unchanged; only the
drawing between two ticks moves. The mesher keeps the last tick's cells and
gives every vertex what it was as well as what it is. The vertex shader
eases from one to the other over the tick, with `t = clamp(since_tick /
TICK_S, 0, 1)` passed as a uniform and smoothstepped.

- **S1. Levels ease.** A top vertex carries its last-tick height. A pool
  rises and drains smoothly, and a slope tilts smoothly.
- **S2. New water flows in from its source.** A cell wet this tick and dry
  the last starts as a thin sheet on its source side, the neighbour it took
  its water from. Its leading edge sweeps across the cell along the flow
  vector over the tick, so the front visibly creeps. No fade in place.
- **S3. Falls pour down.** A new falling column's bottom moves from the lip
  to its foot over the tick. The splash at the foot starts when the column
  lands, not when the tick starts. *This changes how falls look, so the
  rule "Waterfalls are designed in a mockup first" applies: it is shown in
  M31 beside today's look, and ported only on approval.*
- **S4. Drying recedes.** A cell dry this tick and wet the last is drawn for
  one more tick, shrinking back toward its source, then dropped.

**Cost (estimated, not measured).**
- one more float per top vertex, the previous height;
- the dried cells of one tick kept for that tick;
- one uniform.

The mesher already runs once per tick. Saves, the rule and its tests are
untouched.

**The mockup.** M31 (`docs/mockups/waterfall.html`) gets a second 3D tab,
**Pour**. A live pour runs on `water-flow/liquid.js`, the rule the game was
ported from, at the game's 1 s tick. It is drawn with M31's port of the
game's `dc0c5cb` fall look, over a terraced slope with a drop into a pool.
A **Pops (today) / Smoothed** switch sits beside **Pour again** and a tick
speed. The first tab, the fall look, is unchanged. Only one 3D view draws
at a time.

**Approved (2026-11-15).** The owner, on M31 version 2: *"the waterfall look
is pretty good, need to be a bit careful with perf on that, on my laptop it is
a bit rough, so hopefulyy in game it will work well"*.

The mockup's cost does not carry over to the game. The page rebuilds its
geometry in JavaScript every tick and copies the framebuffer for refraction
every frame through three.js. The game already meshes on the CPU once a
tick and already draws this froth.

The port adds:
- one float per top vertex;
- one uniform;
- one tick's dried cells.

It is measured, not assumed. A `perf_suite.py` A/B on the release build
(main's water against the smoothed water), on a pour scene and the walk,
goes in `docs/benchmarks/` with the port. Any slower frame is fixed before
the push.

### Water that dries eases out too (5.4; owner, 2026-11-17)

The owner, on the merged smoothing: *"water spawning does animate correctly.
Looks great, might have some geometry weirdness but very minor. The despawn
is still step based. So if we can apply something similar to that would be
nice."* This is the smoothing M31's Pour tab already shows and the owner
approved (S1 to S4), carried to every way water leaves. It is not a new look:
the fall look stays `water.wgsl`'s as approved, and nothing here changes a
froth, a lip or a splash term.

**Measured (2026-11-17, the M4 Max, `--profile fast`).** A capture rig pours
one source 12 m ahead into a pit five layers deep (`--pour 1 --pour-ahead 12
--pit 5`) and stops it (`PBD_POUR_STOP_S`, a measurement instrument: the
poured source taken through the ordinary edit path, as a bucket takes one),
filmed frame by frame (`PBD_CAPTURE_RUN`). A frame-to-frame change measure
with the HUD masked finds a spike on every tick, 60 frames apart: change
2 to 4 times the steady water's during the pour, and up to 7.6 times (mean
change 10.0 against a median of 1.3) while the pit drains. Inside each
drain tick the fall stands still, then at the tick it is a layer shorter.

Logging every remesh in the game named three causes. The geometry itself is
continuous: at every drain tick, every triangle drawn at the end of the last
tick is drawn at the start of the next (counted on the real tier, rounded to
a millimetre). The causes are around it.

1. **An edit remeshes with nothing to ease from.** Taking water (a bucket, a
   dig, the stopped source) replaces the fine set through `apply_edit`, not
   a tick, so `rebuild_water` meshes it with no tick before it. The taken
   cell is gone in one frame, and every ease still in flight snaps to its
   end (30 visible triangles gone at once at the stop). An edit and a tick
   in the same frame are worse: the tick eases from the set the edit made,
   so the edit is never eased at all.
2. **The shading jumps at the tick.** Only the position eases. The face's
   other lanes are this tick's from the first frame: the flow lane (a top's
   pan and lip nearness; a falling face's froth weight and its metres under
   the lip), the normal and the sky. On a draining fall the lip drops a
   layer a tick, so every falling face's metres under the lip (which places
   the froth pattern, its speed-up and the Glassy lip) moves a metre at
   once, and the face that becomes the new lip goes from froth 0.55 to 1.0.
   The fall reads as stepping although its faces ease. A sloped top's normal
   steps as its levels change, so its light steps too.
3. **Nothing carries an ease across a remesh that is not a tick.** A fine
   set landing from the streaming (the player moving) or an edit mid-tick
   remeshes from scratch, and the water jumps to where the tick was going.

**The spawn's "geometry weirdness"** is found by the same count: at a new
cell's first frame, 72 triangles appear that were not drawn the frame before.
S2 squashes a new cell onto the side its water came from by projecting every
point onto that side's plane. On a hexagon the far corners project past the
side's two corners, so the starting sheet is twice the side's width and pokes
half a cell into the neighbours on either side, into the stone or the air
beside the source. M31's `squasher` does the same. S4 ends a dried cell on the
same over-wide sheet.

**The design.**
- **The tick's ease is unchanged** (M31's S1 to S4): a cell wet in both
  ticks eases its pieces; a new cell sweeps in from its wettest last-tick
  side (S2) or grows down from the water over it (S3); a dried cell is kept
  in the mesh for one tick, squashed toward the water under it (a fall
  drains down, its top sinking to its floor) or toward its wettest side
  (S4), and dropped at the tick's end. Fronts ease linearly, levels by the
  smoothstep. A falling column whose source stopped drains a cell a tick
  from the top (the rule's own order), each cell's top sinking to its floor
  over its tick, so the column's top goes down at one speed.
- **Every lane eases, not only the position.** Each vertex carries its last
  flow lane, normal and sky beside its last position. `vertex_flowing`
  mixes them by the same `t` as the position (the normal renormalised), so
  a fall's froth slides with its lip rather than jumping.
- **Any change to the water eases, not only a tick.** The mesher eases from
  the set it last meshed whenever the fine set changes (an edit, a landing),
  and from the set before the last tick when several ticks ran at once (the
  capture rig's `--water-ticks`). A taken cell is an S4 cell: it shrinks
  away over one tick from the moment it is taken. Water placed is an S2 one.
- **Each vertex has its own clock.** A vertex carries when its ease
  started, in seconds against the mesh's epoch (its last rebuild), and the
  uniform is the seconds since that epoch (`sea_frame.z`) and the tick
  length (`sea_frame.w`, `liquid::TICK_S`, one source). A remesh carries what
  is in flight: a piece of the last mesh still easing toward the same end
  keeps its start and its from; one whose end moved starts again from where
  it is drawn now; a dried piece still shrinking is kept until its own tick
  has played. So an edit or a landing mid-tick never snaps, and a dried
  cell is always given its full tick.
- **The squash stays on its side.** S2's start and S4's end clamp each point
  to the side's own span, between its two corners, so the thin sheet
  (`THIN`, a quarter of the cell's height, as M31) covers the side the water
  came from and nothing beside it.

**Bounded.**
- Six lanes a vertex instead of four (96 bytes against 64).
- The kept dried pieces are at most one tick's dried cells: each goes when
  its own tick has played, whatever remeshes happen meanwhile.
- No allocation per frame: the mesh and its piece table are built once per
  remesh (a tick, an edit, a landing). The last mesh and the set it was made
  from are kept for the next.
- Saves, the rule and its tick are untouched: this is drawing only.

**What it does not do.** A face that changes kind between ticks (a fall's
foot cell, falling face to still face when its fall has gone) changes kind
at the tick: the kind is a switch, not a number to ease. The froth weight it
carries eases.

## Rain rings that alias (owner, 2026-11-16)

The owner's valley in rain shows the river "covered in a dense white
speckle" (`docs/screenshots/rain-artifacts/owner-2026-11-16.webp`).

**Cause, measured.** The sea fragment adds the rain's ring kernel
(`rain_rings`, at `rain_ripple_scale` 3 per metre, strength 6) to the
water's gradient at full strength at every distance. The kernel's rings are
`sin(31 d)` in its uv: a wavelength of 6.8 cm on the water. A pixel at 25 m
is about 2 cm across, and at the grazing angle a river is seen from, far
more along the view: the rings fall under a pixel and point-sample into
per-pixel noise, which the Fresnel mirror and the glint turn into white
speckle. The waves' own fbm is faded by its footprint (`detail`), the
terrain's rings by a 20 m reach (`wet-near-detail`); the water's rings by
neither. Measured over the same patch of the valley's river from 25 m up
(`hf.py`, mean absolute Laplacian of the luma, the rain composite, the lens
and the shower off so only the sheet differs): dry 19.0, in rain 36.5, in
rain with the water's rings off 27.1, with the river bed's wet block off as
well 26.0. The rings are the speckle; the rest is the overcast light on the
bed's own texture. Not the falls' froth: it is gated to falling faces
(`kind == 2 && horizontal < 0.5`) and is untouched.

**Fix.** The rings fade with their own footprint, as the waves do: with
`c` the ring wavelengths per pixel (`length(fwidth(uv)) * 31 / 2pi`), they
are weighed by `1 - smoothstep(0.2, 0.45, c)`, whole while a ring spans
five pixels or more and gone by about two. Near water keeps its rings; the
far river and a sheet seen at a grazing angle keep the waves only. The
falls' froth and the fall look (`dc0c5cb`) are not touched.
## Splashes that show, and better ones (owner, 2026-11-16)

The owner, on a river in rain: "these water splash fx are weak, I expected
them to be made really well like the crab breeze fx", "they also dont render
when they are in front of water either", and "I approved of ground splash a
while ago from rain drops, I dont see them here".

Measured (read off the code, 2026-11-16):

- **The fall-foot spray and mist are the only splash particles.** CPU
  billboards rebuilt each frame (`water_flow.rs` `spawn_splash`,
  `animate_splash`), up to 4000 soft white dots within 70 m, an unlit
  `StandardMaterial` with `AlphaMode::Blend`: Bevy's Transparent3d phase,
  no depth written. Nothing splashes where the player, a craft, a float or
  a fish meets water; the fishing splash is a sound only.
- **Why a splash vanishes over water.** All water is drawn by
  `WaterCompositeNode`, after `Node3d::EndMainPass`, so after Transparent3d
  (`planet_water.rs`). The flowing water reads the finished scene (spray
  included) as "what is behind the water", absorbs it by the path through
  the water and draws over it; it is discarded only where the main pass's
  depth is nearer (`water.wgsl`, `own_depth > clip.z`), and the spray wrote
  none. So a splash standing over a pool, which is where every fall's foot
  is, is erased; it shows only against land or sky. The near rain shower
  has the same flaw over water.
- **The white ribbons along a bank** are the fall froth on the side faces of
  flowing water: the mesher marks any flowing cell's side open to air as
  falling (`falling || above_water && flow.is_some()`), so a river's edges
  and level steps get the lip's streaked froth.
- **The rain splashes** (M32, `weather-rain` 6.3, approved 2026-11-15) are
  built on `claude/rain-splashes` (`weather-rain` 6.4, `planet_splash.rs`,
  captures 6.6) and now merged into `claude/env-v2`; 6.5, the perf A/B
  that gates the push to main, is open. A build from main before that merge
  has none, which is what the owner saw. As designed they stand on ground
  tops only, so a river or the sea gets none, and drawn before the water
  composite a droplet in front of water would meet the same overdraw as the
  spray.
- **Crab Breeze's effects** are not in this repository or `.reference/`:
  the repo cites Crab Breeze for its water lighting, cloud puffs, frame
  latency and audio tools, never for particles. Matching it needs its
  source or a recording.

The plan, in order:

1. **Splashes drawn after the water (a fix, not a look change).** The
   blended effects that stand over water (fall spray and mist, the near
   shower, and the rain splashes when they land) draw in their own pass
   after `WaterCompositeLabel`, depth-tested against the water's own depth,
   so a splash in front of water shows and one behind it does not. This
   restores what was approved; captures before and after, at a fall's foot.
2. **River banks without lip froth.** Froth only on faces that really fall
   (a step of a layer or more); a level river's side keeps the plain water
   sides. This touches the fall look, so it is shown in M31 beside today's
   first (the waterfall rule).
3. **The rain splashes on water too** (built for ground in `weather-rain`
   6.4): rings already mark rain on water; a crown of droplets over it is
   the M32 crown on the water's surface. The perf A/B (6.5) that gates the
   push to main runs on the owner's desktop.
4. **Better splashes, in a mockup first.** A splash kit for every place
   water is struck: fall foot, a player or craft entering water, a float's
   cast, a fish breaking the surface, a wake. Built in M31 (a Splash tab)
   with droplets that arc and fall under gravity, a crown, foam that
   spreads and fades on the surface, and mist, all lit by the scene; shown
   beside today's spray for the owner's approval before any game code.
   Crab Breeze's source or a clip of its splashes would be the reference.

### The fall splash drawn after the water (6.1, 2026-11-16)

The owner again: "water fall splash fx still not drawn in front of water"
(`docs/screenshots/handoff/fall-splash-behind-water.webp`). The cause is the
one measured above: the spray and mist are a Transparent3d mesh, and the
water composite runs after `EndMainPass` and paints over them, with no depth
of theirs to stop it. The sharp white panels in that shot are not the splash
but the lip froth on the pool's side faces (item 2 above), which is a look
change and waits for M31.

The fix, order and depth only:

- `animate_splash` keeps building exactly the same quads (same count,
  places, sizes, colours, alpha, nearest first) but hands them to the render
  world as a list (`SplashQuads`: the camera's right and up, then a centre,
  size and colour per quad) instead of rewriting a Bevy mesh; the Bevy mesh
  and its `StandardMaterial` go.
- The water's pass draws them last, after the cap and the flowing water, in
  `water.wgsl`'s `splash_vertex` / `splash_fragment`: the same 32-texel dot
  sampled nearest (as `ImagePlugin::default_nearest` sampled the image), the
  same unlit colour, the same straight alpha blend.
- Hidden by terrain in front by the cap's own test against the main pass's
  depth; hidden by water in front by the water's depth attachment, tested
  and not written. So from above a pool the splash stands on the surface,
  behind a fall's column it is hidden by the column, and from under the pool
  the surface overhead hides it.
- With the eye fully under water, what no sheet hides is murked by the
  compose's own terms and distance, as it was when the splash was part of
  the scene. (Measured on the captures: some spray and mist stand under a
  pool's surface or poke into the next column's water; from above they are
  now hidden, from under they read as before. Where they are placed is the
  splash's look, item 4 above, and not touched here.)
- Its pipeline is one of the water's, queued with the others when the view's
  water state is made, so the loading screen compiles it before `READY`.

### Mist puffs and bubbles, in a mockup first (6.5, owner, 2026-11-16)

The owner, on the 4-way sheets of the order fix: "design better mist instead
of just a gradient circle puff, use blender/krita or whatever, look at crab
breeze smoke puffs as a model", and "in the water there should be bubbles
particles, this should give it that teal color". The order fix stays (the
mist in front of the water). Both looks are shown first in
`docs/mockups/waterfall-mist.html`, beside the game's look today and the
order fix with the old dot, from above, from the side and from under, by day
and by night. Game code waits for the owner's approval.

- **Mist puffs.** A 4 x 4 sheet of 32-texel cells
  (`assets/textures/fx/mist_puffs/`, made by `tools/fx/mist_puffs_blender.py`
  and `mist_puffs_post.py`, provenance in its `SOURCE.md`): four lumpy
  billowing puffs, four frames of a puff's life each, a five-tone ramp lit
  from above, soft over the outer three texels, no outline. Each mist quad of
  `animate_splash` picks a row by its hash and a column by its age, in place
  of the dot. At an opacity of 0.80 a puff covers the same total as the dot.
  The sheet carries normals for lighting by the sun, as Crab Breeze's
  `smoke_lit` does.
- **Bubbles.** Under the surface at the fall's foot: 60 + 14 per metre fallen,
  born under the falling column and round it, down to 0.6 m + 0.22 m per
  metre fallen, rising at 0.35 to 0.7 m/s and gone at the surface. Each is an
  8-texel sprite (a bright rim, a clear middle, a glint). They are drawn
  before the water so the water's own transmission tints them from above, and
  are murked by distance from under the pool. In the game they would be
  points made on the GPU in the water's pass.
- **The aerated colour.** Over the bubbles the water's transmitted colour
  moves toward a lighter teal (the churn scattering light back up), fading
  out over 2.2 m + 0.15 m per metre fallen from the foot.

### Mist on every fall, the heights compared, and a faster scroll lower down (6.5, owner, 2026-11-16)

The owner asked three more things of the same mockup: "whats rules behind
waterfall? I feel like anything that exposes this even a tiny bit of the
water falling animation should have some mist", "show me a comparison of
different water fall heights", and "can you make the side scrolling texture
increase with the height of the fall?". All three are in
`docs/mockups/waterfall-mist.html`, on the New tab. Game code waits for the
owner's approval of the mockup. Screenshots:
`docs/screenshots/waterfall-mist-heights/`.

**Where the game shows the fall animation today.** The falling froth (the
streaks scrolled down the face, `water.wgsl`'s `in.kind == 2u` branch, and
the waves' radial scroll at `flow_uv_speed_falling`) is drawn on a face of
kind `KIND_FALLING`. `cell_pieces` (`water_flow.rs`) gives a side that kind
when `falling || above_water && flow.is_some()`:

| Where | Shown? | Splash |
| --- | --- | --- |
| A falling cell's side over air: the fall's sheet, every layer from the lip down | yes | at the foot only, none up the face |
| A foot cell's side over air: the flowing cell a fall lands on, until its water spreads to the cell beside it | yes, for a tick or two | its own foot, at the cell's centre |
| A falling cell's side over lower water ("a fall's side over lower water, down to it") | never: over water a cell with water above has all its corner levels at 1, so the face runs from its top to its top and has no height | none |
| A flowing top over a lower flowing top in the same layer (part of a riffle) | no: one sloped top, no face between, kind `KIND_TOP` | none |
| A far river's sheet (`kind == 4u`) | no: flat caps only | none |

A water cell is falling when the cell under it is air or flowing water
(`liquid::transform`). So a river stepping down a layer is a fall like any
other: one falling cell, then its foot, `h = 1`. Every fall that has landed
has a foot, so no place shows the animation with no splash at all. The gaps
are in the amount: a 1 or 2-layer fall gets one puff (alpha 0.16 or 0.20)
and 9 or 15 drops, and nothing marks the faces high up a tall fall. One more
case, by the code rather than seen: water over a source is not falling, so a
fall into a pool of sources counts one layer fewer, and a single-layer spill
onto one has no falling cell at all (a still face, no scroll, no splash).

**The proposed rule.** Every fall is measured by `D`, the metres from the
lip's water to the water it lands in (a part layer counts), and every part
of the splash gets a floor, so a riffle's metre reads as misty. The tall end
stays the owner's tuned 9-layer look.

| | Game today (`h` falling layers) | Proposed (`D` metres) |
| --- | --- | --- |
| Mist puffs | `clamp((h - 1) * 1.5, 1, 24)` | `clamp(round(7 + 0.75 D), 8, 24)` |
| Puff alpha | `0.12 + 0.33 min(h / 8, 1)` | `0.45`, every fall |
| Puff size | `min(0.5 + 0.18 h, 5)` | `max(0.35, min(0.25 + 0.19 D, 5))` |
| Puff rise | `0.25 + 0.04 h` m/s | `0.15 + 0.05 D` m/s, so a short fall's mist stays at its foot |
| Where the puffs rise | a ring all round the foot | the fall's open side, within 110 degrees of the way the water goes over |
| Spray drops | `min(3 + 6 h, 160)` | `max(12, min(3 + 6 D, 160))` |
| Fall droplets (6.5) | none | `max(6, min(160, 9 D))` |

The puffs' size is the mockup's Mist size (1.65) times this, and their alpha
the Mist opacity (1.50) times this, as tuned. In the game every run of
`KIND_FALLING` faces would carry a foot record at the bottom of its lowest
face, not only a foot cell, so a foot cell's face over air mists too; the
mesher already walks those faces. `D` is the falling run's height plus the
lip cell's level less the landing cell's, both of which the mesher has.

**The height comparison.** The mockup's one pit became five falls side by
side off one plateau, the same on all three tabs: a riffle (a river stepping
down one layer into a shallow channel, `D` 1.05 m), and falls of 1, 2, 4 and
9 layers into pools 3.35 m deep (`D` 1.4, 2.4, 4.4 and 9.4 m), each dug into
a shore one layer under the lip less than the fall, open to the front so the
side view sees every face. The 9 keeps the two-ring pit the owner tuned on.
**Fall** picks one to come close to; **All** frames them together.

| Fall | `h` | `D` | Puffs, alpha today | Proposed |
| --- | --- | --- | --- | --- |
| Riffle | 1 | 1.05 m | 1, 0.16 | 8, 0.45 |
| 1 layer | 1 | 1.4 m | 1, 0.16 | 8, 0.45 |
| 2 layers | 2 | 2.4 m | 1, 0.20 | 9, 0.45 |
| 4 layers | 4 | 4.4 m | 4, 0.29 | 10, 0.45 |
| 9 layers | 9 | 9.4 m | 12, 0.45 | 14, 0.45 |

**The scroll, faster down a taller fall.** Today a falling face scrolls at
one speed from lip to foot: its waves at 1 m/s (`flow_uv_speed_falling`) and
its froth streaks at 2.7 m/s (the same scroll, plus the 0.6 drift on noise
stretched 0.35 down the face: (0.35 + 0.6) / 0.35). On the New tab the water
speeds up as it falls: `s` metres under the lip it moves at
`v(s) = min(sqrt(v0^2 + 2 g s), 8 m/s)`, `g` 3 m/s^2 at Fall speed-up 1.00
(0 is the game's one speed). The froth leaves the lip at 2.7 m/s and reaches
about 4 m/s at the foot of the 1-layer fall, 5.8 at the 4 and 8 at the 9.

The pattern stays continuous. Reading it at `s - t v(s)` would move each
height at its own speed but shear the pattern: two heights a metre apart
slide `t (v(s+1) - v(s))` apart, more every second. Instead the pattern is
scrolled at `v0` as today and read at `c(s) = v0 tau(s)`, where `tau(s)` is
the time the water takes to fall `s` metres:

```text
tau(s) = (sqrt(v0^2 + 2 g s) - v0) / g                   s <= s_cap
       = (vmax - v0) / g + (s - s_cap) / vmax            s >  s_cap,  s_cap = (vmax^2 - v0^2) / (2 g)
```

A feature held at constant `c(s) - v0 t` has `d tau / dt = 1`, so it moves
at `v(s)`; `c` is smooth in `s` (including at the cap) and fixed in time, so
the texture never tears, and it stretches by `v(s) / v0` as the water speeds
up, as a falling blob of water does. With `g = 0`, `c(s) = s`: today's
scroll exactly. In the shader it is one function of the face's
metres-under-the-lip (`flow_uv.z` on a falling face, which the mesher
already writes and the shader does not yet read), applied to the waves'
advected position and to the froth's, each with its own `v0`. The game keeps
the one speed until the owner approves.

## The sound of a fall (owner, 2026-11-16)

The owner: "did you add sfx to the waterfall?" ... "which should have like a
light, medium, and roaring based on height". Measured (read off the code,
2026-11-16): the game has no falling-water sound. `assets/audio/ambience/`
has `amb_open_water`, `amb_surf` and `amb_underwater`, beds for the sea and
the shore; `crates/pbd-app/src/audio/hooks.rs` has no fall hook. The sound
only plays; it changes no look (mist, froth and the fall shaders stay as
they are, waiting on the owner's mockups).

**One source of truth for where falls are.** The mesher already finds
every fall's foot for the splash (`liquid_mesh_eased`, the `Foot` records
in `FallFeet`, read by `animate_splash`): its body-local point on the
landing cell's surface and `height`, the number of falling cells over it.
The sound reads that same `FallFeet` resource. No second fall finder.

**The drop, D.** D is metres from the lip's water to the landing water: the
falling run over the foot, `Foot::height` layers of one 1.000 m cell each
(CLAUDE.md, the gold-standard cell height), so D = `height` metres. It is
the same measure the splash sizes its spray and mist by.

**Three beds by the drop** (`pbd_core::audio::fall_band`, thresholds in
`mix.ron`):

| Band | Drop D | What it is | Cue | Heard full within | Silent past |
| --- | --- | --- | --- | ---: | ---: |
| Light | under 3 m (1 or 2 layers, a riffle or a step) | a bright trickle and splash | `fall_light` | 3 m | 40 m |
| Medium | 3 m to under 6 m (3 to 5 layers) | a steady full pour | `fall_medium` | 6 m | 90 m |
| Roaring | 6 m and over | a low, full roar | `fall_roar` | 12 m | 220 m |

`fall_medium_from_m: 3.0` and `fall_roar_from_m: 6.0`. A fall is as loud
as the cue's level and the distance law every placed sound keeps (full
within the near distance, then 6 dB a doubling, faded to nothing over the
last fifth before its reach; the air's low pass darkens it with distance and
behind the head). The roar carries furthest, as a big fall does; a trickle
is gone at 40 m. A fall is only heard while its foot is in the fine tier
(the near terrain), so on a flight the roar ends where the tier ends.

**Placed at its foot.** Each fall is a mono loop at its `Foot::at`, a
body-local point, given to the mixer as `Where::At` and placed against the
camera's eye in the camera's frame, as every placed sound is
(`audio-and-ambience` section 3). The cues are placed effects, so they are
mono files in `assets/audio/sfx/` (`fall_light.ogg`, `fall_medium.ogg`,
`fall_roar.ogg`), not stereo `amb_` beds, which are unplaced.

**A wide fall is one source.** A river over a cliff lands as a row of feet,
one a column. Feet within `fall_merge_m` (10 m) of a louder foot fold into
it: the fall sounds once, from its loudest foot, at that foot's band.

**How many play.** At most `fall_voices` (4) falls sound at once, the
loudest at the ears (the cue's level times its distance gain), so a roar at
100 m outranks a trickle at 30 m. A fall already playing is ranked 3 dB up,
so two falls of nearly the same loudness do not trade places every frame.
Each heard fall holds one voice. A source follows its fall as the loudest
foot of that fall moves (the same band, within the merge distance), rather
than restarting at the new foot. The cap is on voices, fading ones
included: a fall newly among the loudest takes a voice when one is free,
which is at most one fade after another fall left the list.

**Never a pop.** A fall fades in over `fall_fade_s` (1.5 s) from silence
when it is first heard (walking up to it, or a fall that has just landed),
and fades out over the same when it leaves the list or its foot is gone
(the water stopped, or the fall left the fine tier). Only at silence is its
voice let go. The distance law already takes a sound to nothing at its
reach, so walking away never cuts it either.

**Under water, sheltered, paused.** The falls play on the effects bus under
`world`, which already muffles every placed sound when the eye is under
water (600 Hz low pass, 9 dB down, more reverb), pauses with the world and
ducks under a menu. Nothing new is needed for it.

**Cost.** One pass over the feet a frame (the fine tier's few dozen to few
hundred), into a fixed list of at most four, with no allocation after the
first frame; at most four looping voices, streamed from disk as the other
long loops are (`stream: true`, as `fire_crackle` and `hull_water`), so the
decoded bank stays inside its 64 MB budget. No perf suite: no render or
simulation work changes.

**The sounds** (the `sound-effects` skill: recorded CC0 first; nothing
synthesised). Each is 8 s, mono, 48 kHz, looped over a 1 s crossfade, cut
from Freesound recordings whose pages say Creative Commons 0
(`tools/audio/cc0_cues.json`, sources in `assets/audio/sfx/SOURCES.md` and
`AUDIO_SOURCES.md`):

- `fall_light`: a small splashy stream falling into water (kyles, 454340),
  water poured onto stone 8 dB under (TheWoodlandNomad, 363119); a 120 Hz
  high pass keeps it light. Spectral centre 4.4 kHz, 2 % of its energy
  under 250 Hz and 52 % over 4 kHz: bright and splashy.
- `fall_medium`: a medium waterfall (Osiruswaltz, 530170) with a mid-sized
  one 5 dB under (ProductionNow, 352903); 60 Hz high pass. Centre 1.3 kHz,
  10 % under 250 Hz, 5 % over 4 kHz: a broad pour.
- `fall_roar`: Niagara's roar (KevinT1001, 379905) with a big waterfall 4 dB
  under (A.Deathy, 44880) and a huge one 10 dB under for its spray
  (deleted_user_7146007, 383732); 30 Hz high pass, 6 kHz low pass. Centre
  345 Hz, 73 % of its energy under 250 Hz, 1 % over 4 kHz: led by the low
  roar.

Every seam is under a typical sample step and its level across the seam is
within 1 dB of the whole; the contact sheet is
`docs/screenshots/audio/fall-sheet.jpg`. A Claude session cannot listen:
only the owner's ears approve them, on the Sounds page.

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

**Owner's tuning, 2026-11-16 (mockup M35):**
- "fall mist shouldnt be mist, should be like water droplets fx": the mist shed down the fall became droplets, flung off the sheet at the fall's speed and falling under gravity into the pool.
- "mist size = 1.65", "mist opac = 1.50": the mist puffs are drawn at 1.65x the game's puff size and 1.50 opacity on the new sheet. Mist size also scales the fall droplets.
- 2026-11-16: "opacity = 2", "fall speed up = 2", "also please do smooth": the mockup now opens at Mist opacity 2.00, Fall speed-up 2.00 and FX Smooth (the puffs, fall droplets and bubbles drawn smooth rather than pixelated). The owner, the same day: "smooth mist is better", so a port draws the mist smooth.
- 2026-11-16: "disable droplets streak by default, make a note that I dont want them in game yet right now": **droplet streaks are not to go into the game for now.** The mockup opens with Droplet streak at 0 (each fall droplet a single drop); the slider stays in the mockup to try them. A port of the fall droplets leaves the streak trail out until the owner asks for it.
- 2026-11-16: "mist seems to not be connecting with the bottom": a puff's centre now sits 0.2 of its size above the foot (it was its full size plus 5 cm, so its lower edge floated clear of the pool), and the wall blend fades it against the water's surface as well as the ground and walls, so the mist meets the pool softly. In the game the soft fade would read the depth after the water pass.

These are now the mockup's defaults. The other sliders (mist amount, fall droplets, wall blend, bubbles, teal churn) stay at 1.00 / 1.20 until the owner says otherwise.

The owner's tuning applies under the proposed mist rule too ("Mist on every fall" above): the New tab opens on the proposed rule, with a Mist rule switch back to the game's, and on Fall speed-up 1.00 (g = 3 m/s^2, at most 8 m/s); both are recommendations, not the owner's choices, until the owner reviews the mockup.

#### Mockup additions, 2026-11-16 (waterfall-mist, New tab; not in the game)

- **Foot ripples** (owner: "is there some rippling you can do here too? to show water being pushed outward"): rings run
  out from the falling column across the pool, 0.9 m apart at 1.3 m/s, dying over 2 + 0.35 h metres; they tilt the
  water's normal and whiten faintly on their crests near the column. Slider 0-3.
- **Pool flow away from the fall** (owner: "set some flow direction away from the source too (add toggle for flow
  arrows)"): the pool's waves and foam drift outward, 0.5 + 0.07 h m/s at the column, slowing over 2.5 + 0.3 h
  metres, by the two-phase flow-map method (two copies of the waves, each dragged 2 s and reset, cross-faded). A Flow
  arrows checkbox draws the field.
- **Glassy lip** (owner, with a photo of travertine falls: "check out the transition between smooth water and water
  fall, can we do something like that?"): the top's small waves die away over the last 2 m before the lip and the
  first ~0.5 m of the drop stays smooth and green, then thin strands appear and fill in to the full froth by 2.5 m
  down. The game's lip-whitening is off under it. A Lip switch (Froth / Glassy) compares it with the game's look.
- **Rounded lip** (owner: "still looks kinda sharp there"): under Glassy the falling cell's top edge, on every side
  but the one it is fed from, is a 0.45 m quarter-round: the top's vertices within 0.45 m of the rim move onto the
  arc and take its normal, and the sides start 0.45 m below the top where the arc ends. In the game this is a mesher
  change (the falling cell's cap and the top of its side quads), so it is the larger part of a port.
  **Taken back** (owner, on seeing it: "nope go back"): the lip stays square; Glassy keeps only its shading.
- **The owner's edge tuning** (2026-11-16, "here are the edge settings for now"): under Glassy, smooth for 0.70 m
  before the edge, glassy for 0.50 m over it, froth filled in by 2.25 m down, glassy green 0.25, no edge highlight.
  These are the mockup's defaults.

## Porting M35 (6.6; owner, 2026-11-16: "alright, I'm happy with this, send it yo")

The New tab of `docs/mockups/waterfall-mist.html` (version 22) goes into the
game with the mockup's defaults (`S` in `PBDMist`). `waterfall-mist/mist.js`
is the reference: every term below is ported from it, not re-derived. The
rounded lip (`LIP_R`) was taken back by the owner and is not ported: the edge
stays square. Droplet streaks are not ported (owner: "I dont want them in game
yet"): each fall droplet is one drop.

**Where each part goes.**

| Part | Where | Pass |
| --- | --- | --- |
| The proposed mist rule (puffs, spray, fall droplets, the open side) | CPU, `animate_splash` (`water_flow.rs`), as quads | the splash pass |
| Timed reach fade (140 m, 0.6 s in time, never by distance) | CPU, a per-foot visibility kept across frames, keyed by the foot's column | - |
| Puff fade-in at birth, frame cross-fade, the centre at 0.2 x size | CPU (alpha, a continuous frame number) and the splash fragment (two cells mixed) | splash |
| The puff sheet, smooth | `assets/textures/fx/mist_puffs/albedo.png`, sRGB, the water's linear sampler | splash |
| Soft particles against the depth after the water | the splash fragment reads the main pass's depth and the water's own depth | splash, now its own render pass |
| Bubbles | GPU only: camera-facing quads made in `bubble_vertex` from the feet uniform, a pure function of the time | splash pass, before the quads |
| Teal churn, foot ripples, pool outward flow | the water's fragment, on tops near a foot | cap and flowing |
| Fall speed-up `fallC`, the glassy lip | the water's fragment, falling faces and the tops by a lip | flowing |

**What the splash pass becomes.** Today the splash draws inside the water's
render pass against the water's depth attachment. Soft particles need that
depth as a texture, which a pass cannot read while it is its attachment. So
the water pass stores its depth (`StoreOp::Store`, only on frames with a
splash) and the splash draws in a second pass with no depth attachment: its
fragment tests the main pass's depth (terrain in front) and the water's depth
(water in front) itself, and fades by the gap to the nearer of the two over
the wall blend (1.2 m). The water depth texture gains `TEXTURE_BINDING`.

**What the quads carry.** Three lanes a quad instead of two: centre and half
size; colour; and the cell (-1 the old dot, -2 a fall droplet, 0..16 a
continuous puff frame, row x 4 + age x 4) with the flip. `SPLASH_LANES` in Rust
and the shader's stride are one constant, checked by a test.

**The uniform.** `WaterView` gains seven lanes and two arrays of eight:
- `fall_look`, `fall_flow`, `fall_lip`, `fall_more`, `fall_splash`, `aer_color`:
  the tuning below, read from `water.ron`'s `falls`;
- `feet[8]`: xyz the foot (body-local), w its drop `D`, m;
- `feet_more[8]`: x the pool's depth under it, m; y a seed; z its timed fade.

The feet are the nearest eight within the splash's reach, each faded in time
as it joins or leaves the eight, so a ninth fall coming nearer swaps in
without a pop. Every per-fragment loop over feet is bounded by that eight and
by `fall_look.x`, the count.

**What the mesher must supply that it does not today.**
- A foot's drop `D`: the falling run's lip water (the drawn top of the run's
  highest cell) less the landing cell's drawn top. Today a foot has only the
  whole layers fallen, `h`.
- The way the water goes over: the side the top of the run is fed from, as
  a tangent direction at the foot. The proposed rule puts the puffs on the
  open side, within 110 degrees of it.
- The pool's depth under the foot (water layers down to solid), which bounds
  how deep a bubble is born.
- The foot's column, which keys its timed fade.
- Metres under the lip on the face of a foot cell under a fall (a
  `KIND_FALLING` face of a cell that is not itself falling): today 0, which
  would read as glassy all the way down. It becomes the run above plus the
  fraction, as on a falling cell's face, so the face is one fall with the run
  over it.
- On a top, `flow_uv.z` stays the lip weight (1 at a corner on an edge that
  goes over, 0 away from it); the fragment reads it as metres before the lip,
  `-(1 - lip) x 1.42` (half a cell). The mockup's metres along the flow ray are
  sharper; the difference is one of the listed differences.

**The tuning, in `water.ron` (`falls`), validated, one source of defaults.**
Proposed rule; FX smooth; mist opacity 2, amount 1, size 1.65; wall blend
1.2 m; fall droplets 1; droplet streak 0; bubbles 1; teal churn 1; fall
speed-up 2 (g = 3 x 2 m/s^2, at most 8 m/s); ripples 0.5 high, 0.9 m apart,
6 m reach; pool flow 0.1, 6 m reach; Glassy, smooth 0.70 m before the edge,
0.50 m over it, froth filled by 2.25 m; glassy green 0.25; edge highlight 0;
splash reach 140 m and a 0.6 s fade; the aerated colour (0.16, 0.50, 0.46).

**Frame-cost risks.**
- **The pool flow's doubled noise.** Within reach of a foot a top evaluates
  its waves twice more (two copies, each the height and three gradient taps:
  eight more `fbm3`, 24 `gnoise3`). Bounded to fragments within 6 m of one of
  eight feet and at its level (0.3 m), and skipped where the summed flow is
  under 1e-3 m/s. Elsewhere the cost is the feet loop's distance tests.
- **Per-fragment loops over feet.** Three loops (flow, ripples, churn) over at
  most eight feet, each a level test and a distance before any work; on the
  sea and every flowing top. Cheap per foot, but it is on every water pixel;
  the eight is the bound.
- **The bubbles.** 60 + 14 D quads per foot (at most 256), eight feet: at most
  12 288 vertices, each a few hashes; small on screen, but blended and
  overlapping at the foot. Feet with a fade of zero emit nothing (degenerate
  quads cost the vertex stage only).
- **The soft-particle depth read.** Two depth loads and a reconstruction a
  splash fragment, and the water's depth now stored (a full-screen Depth32
  store) on frames with a splash. The puffs are large and overlap: overdraw
  is the real cost, as it was.
- **The second pass.** One more render pass begin on frames with a splash.

A `perf_suite.py` A/B (old exe against new, interleaved, release) over a fall
is the gate the owner set ("need to be a bit careful with perf"); if the
machine is not quiet in this sitting it is owed and said so.

**As built (6.6, 2026-11-16).** As above, with three notes from the stills
(`docs/screenshots/waterfall-look/`): the bubbles are drawn after the water
and take the water's transmission over them themselves (the colour is
`mix(water's lit albedo, bubble, T)` over the pool's depth, which blended over
the water gives exactly the mockup's water-over-bubble result); none of the
splash is drawn under its own pool's surface (the third lane carries that
surface's radius), so a puff's low centre no longer hangs in the pool seen
from under; and the frame cost (6.6.8) is owed.

**Second round (6.6, after the coordinator's review).** Three fixes: a
foot's fades are keyed by its column and layer (two feet stacked in one
column of a cascade reset each other's fade every frame); on a fall's top
cell the metres under the lip start at each corner's own level (a falling
cell's top slopes to its low corners, so most of the edge started past the
glassy band and Glassy looked off); and the stills are of the mockup's own
scene made in the game (`--pit` and `--pour` at the spawn), where the mist
rolls as the mockup's does. In a cascade of side-by-side falls the wall blend
fades each puff against the neighbouring sheets, so the mist reads thinner
than the mockup's single fall; narrowing the blend there would change the
approved look and is the owner's to ask for.
